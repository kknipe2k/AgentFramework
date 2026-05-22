//! Agent Runtime — Tauri desktop shell for agentic AI workflows.

mod commands;
mod drone_lifecycle;
mod sandbox_lifecycle;
mod session_db;

use std::path::PathBuf;
use std::sync::Arc;

use commands::{CurrentTierState, GlobalBudgetState};
use drone_lifecycle::DroneLifecycle;
use runtime_main::audit::{audit_path, AuditWriter};
use runtime_main::drone_ipc::DroneClient;
use runtime_main::hitl::HitlSeam;
use runtime_main::sandbox_ipc::SandboxClient;
use runtime_main::sdk::{ApprovalSeam, SessionId};
use runtime_main::tier::{load_tier, Tier};
use runtime_mcp::client::{
    InMemorySecretStore, KeyringSecretStore, McpClient, Registry, SecretStore,
};
use sandbox_lifecycle::SandboxLifecycle;
use tauri::{Manager, RunEvent};
use tokio::sync::Mutex;

/// Tauri-managed type alias for the drone subprocess handle. Held so the
/// `RunEvent::ExitRequested` handler can `.take()` and call
/// [`DroneLifecycle::shutdown`] before propagating exit.
type ManagedLifecycle = Mutex<Option<DroneLifecycle>>;
/// Tauri-managed type alias for the sandbox subprocess handle (M05 C1).
/// Same `Mutex<Option<_>>` shape as `ManagedLifecycle` so the exit
/// handler can drain + shutdown identically.
type ManagedSandbox = Mutex<Option<SandboxLifecycle>>;

#[allow(
    clippy::too_many_lines,
    reason = "Tauri shell startup is a linear sequence of registrations + spawns; splitting hides the order-of-events that's load-bearing for diagnosis."
)]
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
        // M08 Stage C (spec §M7) — dialog plugin for the local-file
        // picker. Permission granted via src-tauri/capabilities/default.json
        // `dialog:allow-open`. Verified against
        // https://v2.tauri.app/plugin/dialog/ at 2026-05-21 (gotcha #32).
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::set_api_key,
            commands::has_api_key,
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
            commands::get_current_tier,
            commands::request_tier_transition,
            // M06.C — MCP server lifecycle commands
            commands::mcp_add_server,
            commands::mcp_remove_server,
            commands::mcp_test_connection,
            commands::mcp_list_servers,
            // M07.C — import pipeline
            commands::import_artifact,
            // M07.5 — import validate/commit lifecycle (ADR-0017)
            commands::complete_import_artifact,
            commands::cancel_pending_import,
            // M08 Stage B — Builder backend
            commands::validate_framework,
            commands::save_framework,
            commands::load_framework,
            commands::list_installed_artifacts,
            // M08 Stage F1 — the Tester backend
            commands::test_framework,
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
            // M07.5 §8.security L4 / ADR-0017: Novice imports held at the
            // tier-gate review live here between import_artifact
            // returning Pending and the renderer's complete_/cancel_
            // call. Process-local in-memory state (v0.1 single-session);
            // managed unconditionally so the import command trio is
            // always wired even when MCP setup is unavailable.
            app_handle.manage(commands::PendingImportState::default());
            // M05 Stage D §8.security L4: load the persisted tier from
            // `<app_data_dir>/tier.json` (first-run default is Novice).
            // The CurrentTierState seam is the single source of truth
            // for the get_current_tier + request_tier_transition
            // commands; tier_transition events drive the renderer's
            // currentTier state.
            let tier_from_disk = match app_handle.path().app_local_data_dir() {
                Ok(dir) => load_tier(&dir).unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "tier load failed; defaulting to Novice");
                    Tier::default()
                }),
                Err(e) => {
                    tracing::warn!(error = %e, "app_local_data_dir unavailable; defaulting to Novice");
                    Tier::default()
                }
            };
            let tier_state: CurrentTierState = Mutex::new(tier_from_disk);
            app_handle.manage(tier_state);
            // M05 Stage E §8.security L5: best-effort audit log open.
            let audit_writer_opt = open_audit_writer(&app_handle);
            if let Some(ref w) = audit_writer_opt {
                app_handle.manage(Arc::clone(w));
            }
            // M06.C — McpClient: SQLite-backed registry + KeyringSecretStore +
            // shared AuditWriter (when present). v0.1 single-session uses a
            // synthetic session id at app startup; multi-session per §0d
            // post-v0.1 will derive from the active SessionId surface.
            let mcp_client_opt = open_mcp_client(&app_handle, audit_writer_opt.as_ref());
            if let Some(c) = mcp_client_opt {
                app_handle.manage(c);
            }
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
                        // M06.5 🔴-2: the SDK MUST write signals under
                        // the drone's seeded session id or the
                        // signals→sessions FK rejects every row. Manage
                        // it so run_smoke_session builds the AgentSdk
                        // with the matching SessionId (single
                        // source-of-truth, parallel to 🔴-1/ADR-0012).
                        let sdk_session: SessionId = lifecycle.sdk_session_id();
                        app_handle.manage(sdk_session);
                        let managed: ManagedLifecycle = Mutex::new(Some(lifecycle));
                        app_handle.manage(managed);
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "drone spawn failed at setup");
                        // Without a drone, query_session_db + replay_session
                        // would fail at every invocation. Propagating the
                        // setup error here aborts startup with a visible
                        // error rather than producing a half-broken app.
                        return Err(Box::<dyn std::error::Error>::from(e.to_string()));
                    }
                }
                // M05 Stage C1: spawn the sandbox subprocess alongside the
                // drone. v0.1 has no production caller for the sandbox
                // (M09 generators wires the first one); the boundary
                // stays callable-but-unwired here so the L3 surface is
                // ready when M09 lands. Failure to spawn the sandbox at
                // app startup currently aborts the app — same policy as
                // the drone — because the L3 boundary is part of the
                // §8.security contract and a half-broken sandbox is
                // worse than no app at all.
                tracing::info!("spawning runtime-sandbox");
                match SandboxLifecycle::spawn().await {
                    Ok(lifecycle) => {
                        let client: Arc<SandboxClient> = Arc::clone(&lifecycle.client);
                        app_handle.manage(client);
                        let managed: ManagedSandbox = Mutex::new(Some(lifecycle));
                        app_handle.manage(managed);
                        Ok::<(), Box<dyn std::error::Error>>(())
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "sandbox spawn failed at setup");
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
            // Sandbox shutdown mirrors drone's: drain managed-state +
            // graceful-then-kill.
            let managed_sb = app_handle.state::<ManagedSandbox>();
            let sandbox = tauri::async_runtime::block_on(async { managed_sb.lock().await.take() });
            if let Some(lc) = sandbox {
                tauri::async_runtime::block_on(async move {
                    if let Err(e) = lc.shutdown().await {
                        tracing::warn!(error = %e, "sandbox shutdown failed at exit");
                    }
                });
            }
        }
    });
}

/// Open the M05 Stage E §8.security L5 audit log at app startup.
///
/// Returns `Some(Arc<AuditWriter>)` when the file opens; `None` (with a
/// `tracing::warn!`) when the data-directory resolve / `create_dir_all`
/// / `AuditWriter::open` fails. Per phase doc E.3.4 + spec §13.5, audit
/// availability is best-effort observability — the runtime continues
/// without an audit trail rather than abort startup.
fn open_audit_writer<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Option<Arc<AuditWriter>> {
    let dir = match app.path().app_local_data_dir() {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(error = %e, "app_local_data_dir unavailable; audit disabled");
            return None;
        }
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        tracing::warn!(error = %e, "audit log dir create failed");
    }
    let path = audit_path(&dir);
    match tauri::async_runtime::block_on(AuditWriter::open(&path)) {
        Ok(w) => Some(Arc::new(w)),
        Err(e) => {
            tracing::warn!(
                error = %e,
                path = %path.display(),
                "audit log open failed; continuing without audit"
            );
            None
        }
    }
}

/// Open the M06.C `McpClient` at app startup.
///
/// Resolves the registry path through the single source-of-truth
/// [`session_db::session_db_path`] (ADR-0012) — the **same**
/// `<app_local_data_dir>/session.sqlite` the drone uses via
/// [`resolve_db_path`], so an added MCP server is visible to the runtime
/// (closes `docs/M06-irl-findings.md` 🔴-1). Opens a
/// [`KeyringSecretStore`] for per-server auth secrets, and (optionally)
/// wires the existing M05.E `AuditWriter` so MCP install/uninstall/auth
/// events land in the same `skills.audit.jsonl` as capability + tier
/// audit lines. Returns `Some(Arc<McpClient>)` on success; `None` (with
/// `tracing::warn!`) on failure — same best-effort posture as the audit
/// log open per spec §13.5. The renderer's MCP commands will fail with
/// "internal: `McpClient` not initialized" when this returns `None`.
fn open_mcp_client<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    audit: Option<&Arc<AuditWriter>>,
) -> Option<Arc<McpClient>> {
    let dir = match app.path().app_local_data_dir() {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(error = %e, "app_local_data_dir unavailable; MCP disabled");
            return None;
        }
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        tracing::warn!(error = %e, "MCP registry dir create failed");
    }
    let path = session_db::session_db_path(&dir);
    let registry = match Registry::open(&path) {
        Ok(r) => Arc::new(r),
        Err(e) => {
            tracing::warn!(error = %e, path = %path.display(), "MCP registry open failed; MCP disabled");
            return None;
        }
    };
    let secret_store: Arc<dyn SecretStore> = if std::env::var("AGENT_RUNTIME_MCP_IN_MEMORY").is_ok()
    {
        // Test-only override — never set in production. Lets headless
        // CI / smoke runs avoid touching the OS keychain.
        Arc::new(InMemorySecretStore::new())
    } else {
        Arc::new(KeyringSecretStore::new())
    };
    // Manage the registry Arc standalone too — the M07 Stage C
    // `import_artifact` command upserts MCP-server-config imports
    // through the same M06 registry (reuse, not a second DB).
    app.manage(Arc::clone(&registry));
    let session_id = format!("session-{}", uuid::Uuid::new_v4());
    let client = if let Some(w) = audit {
        McpClient::new_with_audit(registry, secret_store, Arc::clone(w), session_id)
    } else {
        McpClient::new(registry, secret_store, session_id)
    };
    Some(Arc::new(client))
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
    Ok(session_db::session_db_path(&dir))
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

    let default = "info,runtime_core=debug,runtime_main=debug,runtime_drone=debug,runtime_sandbox=debug,agent_runtime=debug";
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .with_thread_names(false)
        .compact()
        .init();
}
