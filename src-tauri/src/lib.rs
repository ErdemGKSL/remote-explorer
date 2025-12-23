mod auth;
mod commands;
mod models;
mod ssh;
mod state;

use tauri_plugin_store::StoreExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init());

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                use tauri::Manager;

                let _ = app
                    .get_webview_window("main")
                    .expect("no main window")
                    .set_focus();
            }))
            .plugin(tauri_plugin_shell::init());
    }

    builder
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let _store = app.store("store.json")?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::connection::validate_ssh_connection,
            commands::connection::start_project,
            commands::connection::close_project,
            commands::connection::get_project,
            commands::connection::get_current_pwd,
            commands::connection::get_desktop_environment,
            commands::terminal::create_terminal,
            commands::terminal::execute_terminal_command,
            commands::terminal::send_terminal_input,
            commands::terminal::close_terminal,
            commands::terminal::list_terminals,
            commands::terminal::get_terminal_pwd,
            commands::terminal::clear_terminal_content,
            commands::terminal::get_terminal_content,
            commands::filesystem::get_dir_contents,
            commands::filesystem::create_file,
            commands::filesystem::create_folder,
            commands::filesystem::delete_item
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
