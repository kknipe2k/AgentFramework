//! Agent Runtime — Tauri desktop shell for agentic AI workflows.

mod commands;
mod drone_lifecycle;

use std::path::PathBuf;
use std::sync::Arc;

use commands::GlobalBudgetState;
use drone_lifecycle::DroneLifecycle;
use runtime_main::drone_ipc::DroneClient;
use runtime_main::hitl::HitlSeam;
use runtime_main::sdk::ApprovalSeam;
use tauri::{Manager, RunEvent};
use tokio::sync::Mutex;

/// Tauri-managed type alias for the drone subprocess handle. Held so the
/// `RunEvent::ExitRequested` handler can `.take()` and call
/// [`DroneLifecycle::shutdown`] before propagating exit.
type ManagedLifecycle = Mutex<Option<DroneLifecycle>>;

fn main() {
    init_tracing();
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "agent-runtime starting"
    );

    let app = tauri::Builder::default()
        // M04 Stage E (spec §6a) — Tauri notification plugin powers the
        // `desktop` HITL notifier. Registered alongside the existing
        // invoke handlers; permission granted via
        // src-tauri/capabilities/default.json `notification:default`.
        // Verified against https://v2.tauri.app/plugin/notification/ at
        // 2026-05-10 (gotcha #32).
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::set_api_key,
            commands::run_smoke_session,
            commands::query_session_db,
            commands::replay_session,
            commands::approve_plan,
            commands::revise_plan,
            commands::abort_plan,
            commands::respond_hitl,
            commands::request_resume,
            commands::respond_uncertainty,
            commands::set_global_budget,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            // M04 Stage C: register the in-process ApprovalSeam as
            // managed-state ahead of drone spawn. The seam has no I/O so
            // construction is infallible — registering early keeps the
            // Tauri command layer wired even if drone spawn fails (the
            // approve/revise/abort commands no-op gracefully when no
            // SDK awaiter is registered, per commands.rs::resolve_or_log).
            let seam: Arc<ApprovalSeam> = Arc::new(ApprovalSeam::new());
            app_handle.manage(seam);
            // M04 Stage E: register the in-process HitlSeam as
            // managed-state. Same rationale as ApprovalSeam — no I/O at
            // construction; the renderer's respond_hitl command resolves
            // pending awaits via this seam.
            let hitl_seam: Arc<HitlSeam> = Arc::new(HitlSeam::new());
            app_handle.manage(hitl_seam);
            // M04 Stage F: in-memory global budget cap. Persistent
            // settings storage is M10 first-run UX; v0.1 keeps it
            // process-local so the BudgetHeaderBar's settings panel
            // round-trips without a new dependency.
            let global_budget: GlobalBudgetState = Mutex::new(None);
            app_handle.manage(global_budget);
            // The setup hook runs on the Tauri main thread; we need an
            // async block for the drone spawn + connect. block_on uses
            // the Tauri runtime that's already configured.
            tauri::async_runtime::block_on(async move {
                let db_path = resolve_db_path(&app_handle)?;
                tracing::info!(db_path = %db_path.display(), "spawning runtime-drone");
                match DroneLifecycle::spawn(db_path).await {
                    Ok(lifecycle) => {
                        let client: Arc<DroneClient> = Arc::clone(&lifecycle.client);
                        app_handle.manage(client);
                        let managed: ManagedLifecycle = Mutex::new(Some(lifecycle));
                        app_handle.manage(managed);
                        Ok::<(), Box<dyn std::error::Error>>(())
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "drone spawn failed at setup");
                        // Without a drone, query_session_db + replay_session
                        // would fail at every invocation. Propagating the
                        // setup error here aborts startup with a visible
                        // error rather than producing a half-broken app.
                        Err(Box::<dyn std::error::Error>::from(e.to_string()))
                    }
                }
            })?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let RunEvent::ExitRequested { .. } = event {
            // Drain the lifecycle from managed state and run shutdown
            // synchronously inside the Tauri runtime so the drone gets
            // its graceful-shutdown handshake before the host exits.
            let managed = app_handle.state::<ManagedLifecycle>();
            let lifecycle = tauri::async_runtime::block_on(async { managed.lock().await.take() });
            if let Some(lc) = lifecycle {
                tauri::async_runtime::block_on(async move {
                    if let Err(e) = lc.shutdown().await {
                        tracing::warn!(error = %e, "drone shutdown failed at exit");
                    }
                });
            }
        }
    });
}

/// Resolve the `SQLite` database path for v0.1.
///
/// v0.1 single-session per `agent-runtime-spec.md` §0d; one db per
/// installation. Lives under the app's local data directory (created
/// on first run).
///
/// # Errors
///
/// Returns the underlying [`tauri::Error`] if the platform's app-data
/// directory cannot be resolved (rare; would indicate a misconfigured
/// `tauri.conf.json` `identifier`).
fn resolve_db_path<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<PathBuf> {
    let dir = app.path().app_local_data_dir()?;
    std::fs::create_dir_all(&dir).map_err(tauri::Error::Io)?;
    Ok(dir.join("session.sqlite"))
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

    let default =
        "info,runtime_core=debug,runtime_main=debug,runtime_drone=debug,agent_runtime=debug";
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .with_thread_names(false)
        .compact()
        .init();
}
