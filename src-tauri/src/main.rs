//! Agent Runtime — Tauri desktop shell for agentic AI workflows.

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
