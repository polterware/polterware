//! The PTY sessions behind the Desktop terminal and Connect's provider sign-in
//! flow: one manager per core, one session per shell.

use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use time::OffsetDateTime;

/// The error a write returns once the session closed.
pub const CLOSED_PIPE_MESSAGE: &str = "io: read/write on closed pipe";

/// A terminal operation failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct Error(pub String);

impl Error {
    fn session_not_found() -> Error {
        Error("terminal session not found".to_string())
    }
}

/// SSHProfile is the metadata needed to spawn system ssh inside a PTY.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SshProfile {
    pub user: String,
    pub host: String,
    pub port: i32,
    pub identity_file: String,
}

struct Inner {
    status: String,
    cols: u16,
    rows: u16,
    buffer: Vec<u8>,
    closed: bool,
    master: Option<Box<dyn MasterPty + Send>>,
    child: Option<Box<dyn Child + Send + Sync>>,
}

/// One PTY-backed shell session.
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub target_id: String,
    pub shell: String,
    pub cwd: String,
    pub started_at: OffsetDateTime,
    inner: Arc<Mutex<Inner>>,
    // Kept outside `inner` so a blocked write never holds the read lock.
    writer: Mutex<Option<Box<dyn Write + Send>>>,
}

impl Session {
    /// The session status: `active` until it closes, then `closed`.
    pub fn status(&self) -> String {
        lock(&self.inner).status.clone()
    }

    pub fn cols(&self) -> u16 {
        lock(&self.inner).cols
    }

    pub fn rows(&self) -> u16 {
        lock(&self.inner).rows
    }

    pub fn is_closed(&self) -> bool {
        lock(&self.inner).closed
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("project_id", &self.project_id)
            .field("target_id", &self.target_id)
            .field("status", &self.status())
            .field("shell", &self.shell)
            .field("cwd", &self.cwd)
            .field("started_at", &self.started_at)
            .field("cols", &self.cols())
            .field("rows", &self.rows())
            .finish()
    }
}

/// Owns every terminal session of the core.
pub struct Manager {
    sessions: Mutex<HashMap<String, Arc<Session>>>,
}

impl Default for Manager {
    fn default() -> Self {
        Manager::new()
    }
}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Creates a local shell session.
    // The Go signature takes every field the session records; keep it one call.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        &self,
        id: &str,
        project_id: &str,
        target_id: &str,
        shell: &str,
        cwd: &str,
        cols: u16,
        rows: u16,
    ) -> Result<Arc<Session>, Error> {
        let shell = if shell.is_empty() {
            default_shell()
        } else {
            shell.to_string()
        };
        let command = CommandBuilder::new(&shell);
        self.create_cmd(id, project_id, target_id, &shell, cwd, cols, rows, command)
    }

    /// Opens an interactive remote session via system ssh inside a PTY.
    #[allow(clippy::too_many_arguments)]
    pub fn create_ssh(
        &self,
        id: &str,
        project_id: &str,
        target_id: &str,
        cwd: &str,
        cols: u16,
        rows: u16,
        profile: &SshProfile,
    ) -> Result<Arc<Session>, Error> {
        let (name, args) = build_ssh_command(profile)?;
        let mut command = CommandBuilder::new(&name);
        command.args(args);
        self.create_cmd(id, project_id, target_id, "ssh", cwd, cols, rows, command)
    }

    #[allow(clippy::too_many_arguments)]
    fn create_cmd(
        &self,
        id: &str,
        project_id: &str,
        target_id: &str,
        shell_label: &str,
        cwd: &str,
        cols: u16,
        rows: u16,
        mut command: CommandBuilder,
    ) -> Result<Arc<Session>, Error> {
        let cols = if cols == 0 { 120 } else { cols };
        let rows = if rows == 0 { 32 } else { rows };
        if !cwd.is_empty() {
            command.cwd(cwd);
        }
        let pair = native_pty_system()
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|err| Error(err.to_string()))?;
        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|err| Error(err.to_string()))?;
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|err| Error(err.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|err| Error(err.to_string()))?;
        let master = pair.master;
        // The parent must not hold the slave open, or the reader never sees EOF
        // when the shell exits (Go's pty.Start closes it the same way).
        drop(pair.slave);

        let session = Arc::new(Session {
            id: id.to_string(),
            project_id: project_id.to_string(),
            target_id: target_id.to_string(),
            shell: shell_label.to_string(),
            cwd: cwd.to_string(),
            started_at: OffsetDateTime::now_utc(),
            inner: Arc::new(Mutex::new(Inner {
                status: "active".to_string(),
                cols,
                rows,
                buffer: Vec::new(),
                closed: false,
                master: Some(master),
                child: Some(child),
            })),
            writer: Mutex::new(Some(writer)),
        });
        let inner = Arc::clone(&session.inner);
        std::thread::spawn(move || read_loop(reader, inner));
        lock(&self.sessions).insert(id.to_string(), Arc::clone(&session));
        Ok(session)
    }

    pub fn get(&self, id: &str) -> Result<Arc<Session>, Error> {
        match lock(&self.sessions).get(id) {
            Some(session) => Ok(Arc::clone(session)),
            None => Err(Error::session_not_found()),
        }
    }

    pub fn list(&self, project_id: &str) -> Vec<Arc<Session>> {
        lock(&self.sessions)
            .values()
            .filter(|session| project_id.is_empty() || session.project_id == project_id)
            .cloned()
            .collect()
    }

    pub fn write(&self, id: &str, data: &[u8]) -> Result<(), Error> {
        let session = self.get(id)?;
        if lock(&session.inner).closed {
            return Err(Error(CLOSED_PIPE_MESSAGE.to_string()));
        }
        match lock(&session.writer).as_mut() {
            Some(writer) => writer.write_all(data).map_err(|err| Error(err.to_string())),
            None => Err(Error(CLOSED_PIPE_MESSAGE.to_string())),
        }
    }

    pub fn resize(&self, id: &str, cols: u16, rows: u16) -> Result<Arc<Session>, Error> {
        let session = self.get(id)?;
        if cols == 0 || rows == 0 {
            return Ok(session);
        }
        {
            let inner = lock(&session.inner);
            match inner.master.as_ref() {
                Some(master) => master
                    .resize(PtySize {
                        rows,
                        cols,
                        pixel_width: 0,
                        pixel_height: 0,
                    })
                    .map_err(|err| Error(err.to_string()))?,
                // The pty is closed: Go's Setsize on a closed file says so.
                None => return Err(Error("file already closed".to_string())),
            }
        }
        {
            let mut inner = lock(&session.inner);
            inner.cols = cols;
            inner.rows = rows;
        }
        Ok(session)
    }

    /// Reads the output written since `offset`, with the next offset.
    pub fn read(&self, id: &str, offset: i64) -> Result<(Vec<u8>, i64, bool), Error> {
        let session = self.get(id)?;
        let inner = lock(&session.inner);
        let raw = &inner.buffer;
        let offset = offset.max(0) as usize;
        let offset = offset.min(raw.len());
        Ok((raw[offset..].to_vec(), raw.len() as i64, inner.closed))
    }

    pub fn close(&self, id: &str) -> Result<Arc<Session>, Error> {
        let session = self.get(id)?;
        let (master, mut child) = {
            let mut inner = lock(&session.inner);
            inner.closed = true;
            inner.status = "closed".to_string();
            (inner.master.take(), inner.child.take())
        };
        // Drop both handles so the pty really closes and the reader ends.
        drop(master);
        {
            let mut writer = lock(&session.writer);
            writer.take();
        }
        if let Some(child) = child.as_mut() {
            let _ = child.kill();
        }
        Ok(session)
    }
}

fn read_loop(mut reader: Box<dyn Read + Send>, inner: Arc<Mutex<Inner>>) {
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                lock(&inner).buffer.extend_from_slice(&buf[..n]);
            }
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    let mut inner = lock(&inner);
    inner.closed = true;
    inner.status = "closed".to_string();
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn default_shell() -> String {
    if cfg!(windows) {
        return "powershell.exe".to_string();
    }
    match std::env::var("SHELL") {
        Ok(shell) if !shell.is_empty() => shell,
        _ => "/bin/zsh".to_string(),
    }
}

fn ssh_binary() -> &'static str {
    if cfg!(windows) { "ssh.exe" } else { "ssh" }
}

/// Returns argv for an interactive SSH session.
pub fn build_ssh_command(profile: &SshProfile) -> Result<(String, Vec<String>), Error> {
    let user = profile.user.trim();
    let host = profile.host.trim();
    if user.is_empty() || host.is_empty() {
        return Err(Error("ssh user and host are required".to_string()));
    }
    let port = if profile.port == 0 { 22 } else { profile.port };
    if !(1..=65535).contains(&port) {
        return Err(Error("ssh port must be between 1 and 65535".to_string()));
    }
    let binary = ssh_binary();
    let binary = match look_path(binary) {
        Some(path) => path,
        None => {
            return Err(Error(format!(
                "ssh binary not found ({binary}): install OpenSSH client"
            )));
        }
    };
    let mut args = vec!["-tt".to_string(), "-p".to_string(), port.to_string()];
    let identity = profile.identity_file.trim();
    if !identity.is_empty() {
        args.push("-i".to_string());
        args.push(identity.to_string());
    }
    args.push(format!("{user}@{host}"));
    Ok((binary, args))
}

/// The absolute path of `binary` on PATH, the way `exec.LookPath` finds it.
fn look_path(binary: &str) -> Option<String> {
    if binary.contains('/') || (cfg!(windows) && binary.contains('\\')) {
        let candidate = PathBuf::from(binary);
        return is_executable(&candidate).then(|| candidate.display().to_string());
    }
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let dir = if dir.as_os_str().is_empty() {
            PathBuf::from(".")
        } else {
            dir
        };
        let candidate = dir.join(binary);
        if is_executable(&candidate) {
            return Some(candidate.display().to_string());
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}
