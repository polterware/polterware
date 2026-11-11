//! Runs one short-lived command and reports its exit code and output.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The timeout a run without its own deadline gets.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10 * 60);
/// The most stdout a result carries.
pub const MAX_STDOUT_BYTES: usize = 64 * 1024;

/// Why a run failed, named the way Go's `os/exec` names it.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct Error(pub String);

/// The outcome of one command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Result {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub err: Option<Error>,
}

/// Executes a short-lived command with a default timeout when none is set.
pub fn run(name: &str, args: &[String], cwd: &str) -> Result {
    run_with_timeout(name, args, cwd, None)
}

/// Executes a short-lived command, killing it at the deadline.
pub fn run_with_timeout(
    name: &str,
    args: &[String],
    cwd: &str,
    timeout: Option<Duration>,
) -> Result {
    let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT);
    let mut command = Command::new(name);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if !cwd.is_empty() {
        command.current_dir(cwd);
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            return Result {
                exit_code: -1,
                stdout: String::new(),
                stderr: String::new(),
                err: Some(Error(err.to_string())),
            };
        }
    };
    let stdout_reader = child.stdout.take().map(|out| spawn_reader(Box::new(out)));
    let stderr_reader = child.stderr.take().map(|err| spawn_reader(Box::new(err)));

    let deadline = Instant::now() + timeout;
    let mut failure = None;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    failure = Some(Error("signal: killed".to_string()));
                    break None;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(err) => {
                let _ = child.kill();
                let _ = child.wait();
                failure = Some(Error(err.to_string()));
                break None;
            }
        }
    };

    let stdout = join_reader(stdout_reader);
    let stderr = join_reader(stderr_reader);
    let mut exit_code = 0;
    if let Some(status) = status {
        if let Some(code) = status.code() {
            exit_code = code;
            if code != 0 {
                failure = Some(Error(format!("exit status {code}")));
            }
        } else {
            exit_code = -1;
            failure = Some(Error(signal_message(&status)));
        }
    }

    let stdout = truncate(stdout);
    Result {
        exit_code,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        err: failure,
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

fn truncate(mut out: Vec<u8>) -> Vec<u8> {
    if out.len() > MAX_STDOUT_BYTES {
        out.truncate(MAX_STDOUT_BYTES);
    }
    out
}

/// The `ExitError` text of a process a signal ended.
fn signal_message(status: &std::process::ExitStatus) -> String {
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
    let _ = status;
    "signal: killed".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn run_reports_stdout_and_success() {
        let result = run("sh", &args(&["-c", "printf hello"]), "");
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "hello");
        assert_eq!(result.err, None, "a zero exit is not an error");
    }

    #[test]
    fn run_names_the_exit_status() {
        let result = run("sh", &args(&["-c", "exit 3"]), "");
        assert_eq!(result.exit_code, 3);
        assert_eq!(
            result.err.map(|err| err.to_string()),
            Some("exit status 3".to_string())
        );
    }

    #[test]
    fn run_runs_in_the_working_directory() {
        let dir = tempfile::tempdir().expect("temp dir");
        let result = run("pwd", &[], &dir.path().display().to_string());
        assert!(!result.stdout.trim().is_empty());
        assert!(
            result.stdout.trim().ends_with(
                dir.path()
                    .file_name()
                    .expect("dir name")
                    .to_string_lossy()
                    .as_ref()
            )
        );
    }

    #[test]
    fn run_truncates_stdout() {
        let result = run(
            "sh",
            &args(&["-c", "head -c 70000 /dev/zero | tr '\\0' 'a'"]),
            "",
        );
        assert_eq!(result.stdout.len(), MAX_STDOUT_BYTES);
    }

    #[test]
    fn run_reports_a_missing_binary_without_a_code() {
        let result = run("polterware-missing-binary", &[], "");
        assert_eq!(result.exit_code, -1);
        assert!(result.err.is_some(), "a missing binary is an error");
    }
}
