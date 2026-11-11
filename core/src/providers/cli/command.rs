//! Runs a platform CLI with a deadline and its output, the way `os/exec` did
//! in Go, plus the PATH lookup the Go package used before every run.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::{Error, Output};

/// A moment after which a CLI call is killed.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Deadline(Instant);

impl Deadline {
    pub(crate) fn after(after: Duration) -> Self {
        Self(Instant::now() + after)
    }

    /// The time left before the deadline, never negative.
    fn remaining(self) -> Duration {
        self.0.saturating_duration_since(Instant::now())
    }
}

/// One finished run: both streams, and why it failed when it did.
struct Captured {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    failure: Option<String>,
    timed_out: bool,
}

/// A run that merges both streams, as Go's `CombinedOutput` did.
pub(crate) fn run_combined(name: &str, args: &[&str], deadline: Deadline) -> Output {
    let captured = execute(name, args, deadline);
    let mut out = captured.stdout;
    out.extend_from_slice(&captured.stderr);
    Output {
        out,
        err: captured.failure.map(Error::Message),
    }
}

/// A run that keeps stdout for the JSON and reads stderr only when it fails,
/// so progress lines a CLI writes to stderr never reach the JSON.
pub(crate) fn platform_runner(name: &str, args: &[&str], deadline: Deadline) -> Output {
    let captured = execute(name, args, deadline);
    let Some(failure) = captured.failure else {
        return Output {
            out: captured.stdout,
            err: None,
        };
    };
    if captured.timed_out {
        return Output {
            out: Vec::new(),
            err: Some(Error::Message(format!("{name} did not answer in time"))),
        };
    }
    let mut out = captured.stderr;
    out.extend_from_slice(&captured.stdout);
    Output {
        out,
        err: Some(Error::Message(failure)),
    }
}

fn execute(name: &str, args: &[&str], deadline: Deadline) -> Captured {
    let mut command = Command::new(name);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            return Captured {
                stdout: Vec::new(),
                stderr: Vec::new(),
                failure: Some(err.to_string()),
                timed_out: false,
            };
        }
    };
    let stdout_reader = child.stdout.take().map(|out| spawn_reader(Box::new(out)));
    let stderr_reader = child.stderr.take().map(|err| spawn_reader(Box::new(err)));

    let mut failure = None;
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if deadline.remaining().is_zero() {
                    let _ = child.kill();
                    let _ = child.wait();
                    timed_out = true;
                    failure = Some("signal: killed".to_string());
                    break None;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(err) => {
                let _ = child.kill();
                let _ = child.wait();
                failure = Some(err.to_string());
                break None;
            }
        }
    };
    if let Some(status) = status
        && let Some(message) = exit_message(&status)
    {
        failure = Some(message);
    }

    Captured {
        stdout: join_reader(stdout_reader),
        stderr: join_reader(stderr_reader),
        failure,
        timed_out,
    }
}

fn spawn_reader(mut reader: Box<dyn Read + Send>) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut out = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
        out
    })
}

fn join_reader(reader: Option<std::thread::JoinHandle<Vec<u8>>>) -> Vec<u8> {
    match reader {
        Some(handle) => handle.join().unwrap_or_default(),
        None => Vec::new(),
    }
}

/// Why a run failed, named the way Go's `ExitError` named it, or None when it
/// exited with status zero.
fn exit_message(status: &std::process::ExitStatus) -> Option<String> {
    if status.success() {
        return None;
    }
    if let Some(code) = status.code() {
        return Some(format!("exit status {code}"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return Some(match signal {
                9 => "signal: killed".to_string(),
                15 => "signal: terminated".to_string(),
                other => format!("signal: {other}"),
            });
        }
    }
    Some("signal: killed".to_string())
}

/// The absolute path of `binary` on PATH, the way `exec.LookPath` finds it.
pub(crate) fn look_path(binary: &str) -> Option<String> {
    if binary.contains('/') || (cfg!(windows) && binary.contains('\\')) {
        let candidate = PathBuf::from(binary);
        return is_executable(&candidate).then(|| candidate.display().to_string());
    }
    let path = std::env::var("PATH").ok()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_combined_merges_both_streams() {
        let out = run_combined(
            "sh",
            &["-c", "printf out; printf err >&2"],
            Deadline::after(Duration::from_secs(10)),
        );
        assert_eq!(out.err, None);
        let text = String::from_utf8_lossy(&out.out);
        assert!(text.contains("out") && text.contains("err"), "{text:?}");
    }

    #[test]
    fn platform_runner_keeps_stdout_alone_on_success() {
        let out = platform_runner(
            "sh",
            &["-c", "printf '{\"a\":1}'; printf progress >&2"],
            Deadline::after(Duration::from_secs(10)),
        );
        assert_eq!(out.err, None);
        assert_eq!(String::from_utf8_lossy(&out.out), "{\"a\":1}");
    }

    #[test]
    fn platform_runner_names_the_reason_on_failure() {
        let out = platform_runner(
            "sh",
            &["-c", "printf out; printf reason >&2; exit 1"],
            Deadline::after(Duration::from_secs(10)),
        );
        assert_eq!(out.err, Some(Error::Message("exit status 1".to_string())));
        let text = String::from_utf8_lossy(&out.out);
        assert!(text.starts_with("reason"), "{text:?}");
    }

    #[test]
    fn look_path_finds_a_binary_on_path() {
        assert!(look_path("sh").is_some());
        assert!(look_path("polterware-missing-binary").is_none());
    }
}
