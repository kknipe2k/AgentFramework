//! Agent Runtime — Tauri desktop shell for agentic AI workflows.

mod commands;

fn main() {
    init_tracing();
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "agent-runtime starting"
    );
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::set_api_key,
            commands::run_smoke_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Initialize the global `tracing` subscriber for the Tauri main process.
///
/// Default level is `info` for everything, `debug` for project crates
/// (`runtime_core`, `runtime_main`, `runtime_drone`, `agent_runtime`).
/// Override via `RUST_LOG` env (`RUST_LOG=trace` for verbose, etc.) per
/// `tracing_subscriber::EnvFilter` syntax.
///
/// Per spec §13.5 "Dev Logging" — logs are dev-only, sink to stdout/stderr,
/// never phoned home (CLAUDE.md §4 hard rule #4 zero-telemetry remains in
/// force). Secrets MUST NOT be logged: `tracing` calls in this codebase wrap
/// API keys in `secrecy::SecretString`, which suppresses Debug output.
fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    let default = "info,runtime_core=debug,runtime_main=debug,runtime_drone=debug,agent_runtime=debug";
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .with_thread_names(false)
        .compact()
        .init();
}
