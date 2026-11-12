//! The `polterware` binary: the CLI, `serve`, and `mcp`.

fn main() {
    std::process::exit(polterware_core::runtimes::cli::main());
}
