//! Port of `backend/platform/runstate`.
//!
//! Owns the small files that let local Polterware processes find the core
//! (Decision 0007): core.json (endpoint, token, build), core.lock (one core per
//! database), and core.log. Files live in `<db dir>/run` and are private to the
//! user.

use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use rand::Rng;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

const CORE_FILE: &str = "core.json";

/// Describes the running core (`polterware serve`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Core {
    pub pid: i32,
    pub endpoint: String,
    pub token: String,
    #[serde(rename = "dbPath")]
    pub db_path: String,
    /// Identifies the core binary (see `coreclient.BuildID`).
    pub build: String,
    #[serde(rename = "startedAt", with = "time::serde::rfc3339")]
    pub started_at: OffsetDateTime,
}

/// The run-state directory for a database path.
pub fn dir(db_path: impl AsRef<Path>) -> PathBuf {
    match db_path.as_ref().parent() {
        Some(parent) if !parent.as_os_str().is_empty() && parent != Path::new(".") => {
            parent.join("run")
        }
        _ => PathBuf::from("run"),
    }
}

pub fn core_path(db_path: impl AsRef<Path>) -> PathBuf {
    dir(db_path).join(CORE_FILE)
}

pub fn log_path(db_path: impl AsRef<Path>) -> PathBuf {
    dir(db_path).join("core.log")
}

pub fn lock_path(db_path: impl AsRef<Path>) -> PathBuf {
    dir(db_path).join("core.lock")
}

/// Another core already owns this database.
pub const ERR_CORE_LOCKED: &str = "another polterware core owns this database";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Another core already owns this database.
    #[error("another polterware core owns this database")]
    CoreLocked,
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Takes the exclusive per-database core lock (Decision 0007: one core per
/// database). The lock lasts until the returned file is closed or the process
/// exits.
pub fn lock_core(db_path: impl AsRef<Path>) -> Result<File> {
    ensure_dir(&db_path)?;
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .mode(0o600)
        .open(lock_path(&db_path))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(std::fs::TryLockError::WouldBlock) => Err(Error::CoreLocked),
        Err(std::fs::TryLockError::Error(err)) => Err(Error::Io(err)),
    }
}

/// Creates the run-state directory with user-only permissions.
pub fn ensure_dir(db_path: impl AsRef<Path>) -> Result<()> {
    let run_dir = dir(db_path);
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&run_dir)?;
    Ok(())
}

/// Writes `v` atomically with 0600 permissions.
pub fn write_json<T: Serialize>(path: impl AsRef<Path>, v: &T) -> Result<()> {
    let body = serde_json::to_vec_pretty(v)?;
    let path = path.as_ref();
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(parent)?;
    }
    let tmp = path
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
        .join(format!(".tmp-{:016x}", rand::rng().random::<u64>()));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&tmp)?;
    let written = (|| -> io::Result<()> {
        file.write_all(&body)?;
        file.write_all(b"\n")
    })();
    drop(file);
    if let Err(err) = written {
        let _ = fs::remove_file(&tmp);
        return Err(Error::Io(err));
    }
    if let Err(err) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(Error::Io(err));
    }
    Ok(())
}

/// Reports whether a process with `pid` exists.
pub fn pid_alive(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }
    // EPERM means the process exists but belongs to another user.
    const EPERM: i32 = 1;
    match kill_zero(pid) {
        Ok(()) => true,
        Err(err) => err.raw_os_error() == Some(EPERM),
    }
}

fn kill_zero(pid: i32) -> io::Result<()> {
    let rc = unsafe { kill(pid, 0) };
    if rc == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

/// Returns the live core for `db_path`. A stale file is removed and reported as
/// not found.
pub fn read_core(db_path: impl AsRef<Path>) -> Option<Core> {
    let path = core_path(db_path);
    let body = fs::read(&path).ok()?;
    let Ok(core) = serde_json::from_slice::<Core>(&body) else {
        let _ = fs::remove_file(&path);
        return None;
    };
    if !pid_alive(core.pid) {
        let _ = fs::remove_file(&path);
        return None;
    }
    Some(core)
}

/// Deletes core.json only when it still belongs to `pid`.
pub fn remove_core(db_path: impl AsRef<Path>, pid: i32) {
    let path = core_path(db_path);
    let Ok(body) = fs::read(&path) else {
        return;
    };
    if let Ok(core) = serde_json::from_slice::<Core>(&body)
        && core.pid != pid
    {
        return;
    }
    let _ = fs::remove_file(&path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_core_is_exclusive() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let db = tmp.path().join("local.db");
        let first = lock_core(&db).expect("first lock");
        match lock_core(&db) {
            Err(Error::CoreLocked) => {}
            other => panic!("second lock = {other:?}, want ErrCoreLocked"),
        }
        drop(first);
        let again = lock_core(&db).expect("lock after release");
        drop(again);
    }

    #[test]
    fn paths_live_in_a_private_run_directory() {
        let db = Path::new("/tmp/data/local.db");
        assert_eq!(dir(db), Path::new("/tmp/data/run"));
        assert_eq!(core_path(db), Path::new("/tmp/data/run/core.json"));
        assert_eq!(log_path(db), Path::new("/tmp/data/run/core.log"));
        assert_eq!(lock_path(db), Path::new("/tmp/data/run/core.lock"));
        assert_eq!(dir("local.db"), Path::new("run"));
    }

    #[test]
    fn write_json_round_trips_and_read_core_drops_stale_files() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let db_path = tmp.path().join("local.db");
        let started_at = OffsetDateTime::from_unix_timestamp(1_767_225_600).expect("timestamp");
        let core = Core {
            pid: std::process::id() as i32,
            endpoint: "http://127.0.0.1:1234".to_string(),
            token: "token".to_string(),
            db_path: db_path.display().to_string(),
            build: "build".to_string(),
            started_at,
        };
        let path = core_path(&db_path);
        write_json(&path, &core).expect("write core");
        let body = fs::read_to_string(&path).expect("read core");
        assert!(body.contains("\"dbPath\""), "{body}");
        assert!(body.contains("\"startedAt\""), "{body}");
        assert!(body.ends_with("}\n"), "{body}");
        assert_eq!(read_core(&db_path), Some(core.clone()));

        let stale = Core {
            pid: 2_147_483_647,
            ..core
        };
        write_json(&path, &stale).expect("write stale core");
        assert_eq!(read_core(&db_path), None);
        assert!(!path.exists());
    }
}
