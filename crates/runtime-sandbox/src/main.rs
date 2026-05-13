//! `runtime-sandbox` binary entry point.
//!
//! CLI: `runtime-sandbox --session-id <id> --ipc-socket <path>`.

use clap::Parser;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "runtime-sandbox", version)]
struct Args {
    #[arg(long)]
    session_id: String,
    #[arg(long)]
    ipc_socket: PathBuf,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    init_tracing();
    let args = Args::parse();
    info!(session_id = %args.session_id, "sandbox starting");
    if let Err(e) = runtime_sandbox::run(args.session_id, args.ipc_socket).await {
        error!(error = %e, "sandbox exited with error");
        std::process::exit(1);
    }
    info!("sandbox exited cleanly");
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let env =
        EnvFilter::try_from_env("RUNTIME_SANDBOX_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(env).with_target(false).json().init();
}
