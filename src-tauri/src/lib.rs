use std::time::Duration;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};
use std::path::Path;
use tauri_plugin_store::StoreExt;
use tokio::time::timeout;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn validate_ssh_connection(
    host: &str,
    user: &str,
    password: Option<&str>,
    key_file: Option<&str>,
    public_key_file: Option<&str>,
    auth_method: &str,
) -> Result<bool, String> {
    // Parse host and port (default SSH port is 22)
    let (hostname, port) = if host.contains(':') {
        let parts: Vec<&str> = host.splitn(2, ':').collect();
        let port = parts[1].parse::<u16>().unwrap_or(22);
        (parts[0], port)
    } else {
        (host, 22)
    };

    // Prepare authentication method based on selected method
    let auth_method = match auth_method {
        "password" => {
            if let Some(pwd) = password {
                AuthMethod::with_password(pwd)
            } else {
                return Err("Password not provided".to_string());
            }
        }
        "key" => {
            if let Some(key_path) = key_file {
                if !Path::new(key_path).exists() {
                    return Err(format!("Key file not found: {}", key_path));
                }
                AuthMethod::with_key_file(key_path, None)
            } else {
                return Err("Key file path not provided".to_string());
            }
        }
        "public_key" => {
            if let Some(pub_key_path) = public_key_file {
                if !Path::new(pub_key_path).exists() {
                    return Err(format!("Public key file not found: {}", pub_key_path));
                }
                AuthMethod::with_public_key_file(pub_key_path)
            } else {
                return Err("Public key file path not provided".to_string());
            }
        }
        "agent" => AuthMethod::with_agent(),
        _ => return Err(format!("Unknown authentication method: {}", auth_method)),
    };

    // Connect to SSH server
    let client = Client::connect(
        (hostname, port),
        user,
        auth_method,
        ServerCheckMethod::NoCheck,
    );

    let client = timeout(Duration::from_secs(5), client)
        .await
        .map_err(|e| format!("SSH connection failed - {}", e))?
        .map_err(|e| format!("SSH connection failed - {}", e))?;

    // Execute a simple test command
    let result = client
        .execute("echo connection_test")
        .await
        .map_err(|e| format!("SSH command failed - {}", e))?;

    // Check if command executed successfully
    if result.exit_status == 0 {
        Ok(true)
    } else {
        Err(format!(
            "Command failed with exit status: {}",
            result.exit_status
        ))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    let mut builder = tauri::Builder::default();
    
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            use tauri::Manager;

            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }));
    }

    builder
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let _store = app.store("store.json")?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, validate_ssh_connection])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
