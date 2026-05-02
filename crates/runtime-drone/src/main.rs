//! `runtime-drone` binary entry point.
//!
//! CLI: `runtime-drone --session-id <id> --db-path <path> --ipc-socket <path>`.

use clap::Parser;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "runtime-drone", version)]
struct Args {
    #[arg(long)]
    session_id: String,
    #[arg(long)]
    db_path: PathBuf,
    #[arg(long)]
    ipc_socket: PathBuf,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    init_tracing();
    let args = Args::parse();
    info!(session_id = %args.session_id, "drone starting");
    if let Err(e) = runtime_drone::run(args.session_id, args.db_path, args.ipc_socket).await {
        error!(error = %e, "drone exited with error");
        std::process::exit(1);
    }
    info!("drone exited cleanly");
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let env =
        EnvFilter::try_from_env("RUNTIME_DRONE_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(env).with_target(false).json().init();
}
