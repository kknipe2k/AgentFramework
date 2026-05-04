//! Agent Runtime — Tauri desktop shell for agentic AI workflows.

mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::set_api_key,
            commands::run_smoke_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
