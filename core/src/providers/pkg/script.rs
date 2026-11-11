//! Runs a package manager's script.

use crate::providers::exec;

/// Runs `pnpm run <script>` in `cwd`. A successful run answers with its stdout;
/// a failed one answers with its stderr and its error.
pub fn run_script(cwd: &str, script: &str) -> (String, Option<exec::Error>) {
    let res = exec::run("pnpm", &["run", script].map(String::from), cwd);
    match res.err {
        Some(err) => (res.stderr, Some(err)),
        None => (res.stdout, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_placeholder() {
        // The Go placeholder only names the runner; here the signature is checked.
        let _runner: fn(&str, &str) -> (String, Option<exec::Error>) = run_script;
    }
}
