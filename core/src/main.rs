//! The `polterware` binary: the CLI, `serve`, and `mcp` (port of `backend/runtimes/cli`).

fn main() {
    std::process::exit(polterware_core::runtimes::cli::main());
}
