//! Runs an agent client CLI with a deadline and its combined output, and
//! answers the two system questions the setup asks: where a binary lives and
//! where the user's home directory is.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::{Error, Result};

/// Why a client CLI run did not succeed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Failure {
    /// The deadline passed and the process was killed.
    TimedOut,
    /// The process ran and exited with a status.
    Exit { message: String },
    /// The process could not be started.
    Spawn(String),
}

impl Failure {
    /// The text `err.Error()` would carry in the Go package.
    pub(crate) fn message(&self) -> String {
        match self {
            Failure::TimedOut => "signal: killed".to_string(),
            Failure::Exit { message } => message.clone(),
            Failure::Spawn(message) => message.clone(),
        }
    }
}

fn env_value(key: &'static str) -> Option<String> {
    #[cfg(test)]
    if let Some(value) = overrides::get(key) {
        return Some(value);
    }
    std::env::var(key).ok()
}

/// Runs `binary` with `args`, merging stdout and stderr the way
/// `CombinedOutput` does.
pub(crate) fn run<S: AsRef<str>>(
    binary: &str,
    args: &[S],
    timeout: Duration,
) -> (Vec<u8>, Option<Failure>) {
    // `os/exec` resolves the binary on PATH up front; do the same so the
    // client CLI that runs is the one the lookup found.
    let program = look_path(binary).unwrap_or_else(|| binary.to_string());
    let mut command = Command::new(program);
    command
        .args(args.iter().map(AsRef::as_ref))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => return (Vec::new(), Some(Failure::Spawn(err.to_string()))),
    };
    let combined = Arc::new(Mutex::new(Vec::new()));
    let mut readers: Vec<std::thread::JoinHandle<()>> = Vec::new();
    let streams: [Option<Box<dyn Read + Send>>; 2] = [
        child
            .stdout
            .take()
            .map(|stdout| Box::new(stdout) as Box<dyn Read + Send>),
        child
            .stderr
            .take()
            .map(|stderr| Box::new(stderr) as Box<dyn Read + Send>),
    ];
    for stream in streams.into_iter().flatten() {
        let combined = Arc::clone(&combined);
        readers.push(std::thread::spawn(move || {
            let mut stream = stream;
            let mut buf = [0u8; 4096];
            loop {
                match stream.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let mut output = combined
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        output.extend_from_slice(&buf[..n]);
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
        }));
    }

    let deadline = Instant::now() + timeout;
    let failure = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    break None;
                }
                break Some(Failure::Exit {
                    message: exit_message(&status),
                });
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Some(Failure::TimedOut);
                }
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(err) => {
                let _ = child.kill();
                let _ = child.wait();
                break Some(Failure::Spawn(err.to_string()));
            }
        }
    };
    for reader in readers {
        let _ = reader.join();
    }
    let output = combined
        .lock()
        .map(|output| output.clone())
        .unwrap_or_default();
    (output, failure)
}

fn exit_message(status: &std::process::ExitStatus) -> String {
    if let Some(code) = status.code() {
        return format!("exit status {code}");
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return match signal {
                9 => "signal: killed".to_string(),
                15 => "signal: terminated".to_string(),
                other => format!("signal: {other}"),
            };
        }
    }
    "signal: killed".to_string()
}

/// The absolute path of `binary` on PATH, the way `exec.LookPath` finds it.
pub(crate) fn look_path(binary: &str) -> Option<String> {
    if binary.contains('/') || (cfg!(windows) && binary.contains('\\')) {
        let candidate = PathBuf::from(binary);
        return is_executable(&candidate).then(|| candidate.display().to_string());
    }
    let path = env_value("PATH")?;
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

/// The user's home directory, as `os.UserHomeDir` reads it.
pub(crate) fn home_dir() -> Result<String> {
    #[cfg(windows)]
    const KEY: &str = "USERPROFILE";
    #[cfg(not(windows))]
    const KEY: &str = "HOME";
    match env_value(KEY) {
        Some(value) => Ok(value),
        None => Err(Error(format!("${KEY} is not defined"))),
    }
}

/// Sets the values tests read instead of the process environment. Tests run in
/// parallel threads, and HOME and PATH are process-wide, so the ported Go tests
/// that install agents or fake a client CLI override them per thread.
#[cfg(test)]
pub(crate) mod overrides {
    use std::cell::RefCell;
    use std::collections::HashMap;

    thread_local! {
        static VALUES: RefCell<HashMap<&'static str, String>> = RefCell::new(HashMap::new());
    }

    pub(crate) fn set(key: &'static str, value: String) {
        VALUES.with(|values| {
            values.borrow_mut().insert(key, value);
        });
    }

    pub(crate) fn remove(key: &'static str) {
        VALUES.with(|values| {
            values.borrow_mut().remove(key);
        });
    }

    pub(crate) fn get(key: &'static str) -> Option<String> {
        VALUES.with(|values| values.borrow().get(key).cloned())
    }
}
