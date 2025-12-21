use crate::auth::build_auth_method;
use crate::models::{Project, ProjectInfo};
use crate::ssh::{connect_to_ssh, parse_host_port};
use crate::state::{add_project, get_project_by_key, PROJECTS};
use std::sync::Arc;

#[tauri::command]
pub async fn validate_ssh_connection(
    host: &str,
    user: &str,
    password: Option<&str>,
    key_file: Option<&str>,
    public_key_file: Option<&str>,
    auth_method: &str,
) -> Result<bool, String> {
    let host_port = parse_host_port(host);
    let auth = build_auth_method(auth_method, password, key_file, public_key_file)?;

    let client = connect_to_ssh(&host_port.hostname, host_port.port, user, auth, 5).await?;

    let result = client
        .execute("echo connection_test")
        .await
        .map_err(|e| format!("SSH command failed - {}", e))?;

    if result.exit_status == 0 {
        Ok(true)
    } else {
        Err(format!(
            "Command failed with exit status: {}",
            result.exit_status
        ))
    }
}

#[tauri::command]
pub async fn start_project(
    app: tauri::AppHandle,
    key: String,
    name: String,
    host: String,
    user: String,
    password: Option<String>,
    key_file: Option<String>,
    public_key_file: Option<String>,
    auth_method: String,
) -> Result<(), String> {
    let host_port = parse_host_port(&host);
    let auth = build_auth_method(
        &auth_method,
        password.as_deref(),
        key_file.as_deref(),
        public_key_file.as_deref(),
    )?;

    let client = connect_to_ssh(&host_port.hostname, host_port.port, &user, auth, 10).await?;

    // Test the connection with a simple command
    let result = client
        .execute("echo start_project_test")
        .await
        .map_err(|e| format!("SSH command failed - {}", e))?;

    if result.exit_status != 0 {
        return Err(format!(
            "Test command failed with exit status: {}",
            result.exit_status
        ));
    }

    // Create the project and store it
    let project = Project {
        key: key.clone(),
        name: name.clone(),
        host,
        password,
        key_file,
        public_key_file,
        auth_method,
        main_connection: Arc::new(client),
        terminal_connections: vec![],
    };

    add_project(project)?;

    // Create a new window for the project (desktop) or navigate in current window (mobile)
    #[cfg(desktop)]
    {
        use tauri::WebviewUrl;
        use tauri::WebviewWindowBuilder;

        let window_label = format!("remote-{}", key);
        let url = format!("/remote?key={}", key);

        let _ = WebviewWindowBuilder::new(&app, &window_label, WebviewUrl::App(url.into()))
            .inner_size(1200.0, 800.0)
            .title(&format!("{} - Remote Explorer", name))
            .decorations(false)
            .build()
            .map_err(|e| format!("Failed to create window: {}", e))?;
    }

    #[cfg(mobile)]
    {
        use tauri::Manager;
        let main_window = app
            .get_webview_window("main")
            .ok_or_else(|| "Main window not found".to_string())?;

        let url = format!("/remote?key={}", key);
        main_window
            .eval(&format!("window.location.href = '{}';", url))
            .map_err(|e| format!("Failed to navigate: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_project(key: String) -> Result<ProjectInfo, String> {
    get_project_by_key(&key).map(|project| ProjectInfo {
        key: project.key.clone(),
        auth_method: project.auth_method.clone(),
        host: project.host.clone(),
        name: project.name.clone(),
    })
}

#[tauri::command]
pub async fn get_current_pwd(key: String) -> Result<String, String> {
    let connection = {
        let projects = PROJECTS
            .lock()
            .map_err(|e| format!("Failed to lock projects: {}", e))?;

        let project = projects
            .iter()
            .find(|p| p.key == key)
            .ok_or_else(|| "Project not found".to_string())?;

        std::sync::Arc::clone(&project.main_connection)
    };

    let result = connection
        .execute("pwd")
        .await
        .map_err(|e| format!("Failed to execute pwd: {}", e))?;

    if result.exit_status == 0 {
        Ok(result.stdout.trim().to_string())
    } else {
        Err(format!("Command failed: {}", result.stderr))
    }
}

#[tauri::command]
pub async fn close_project(app: tauri::AppHandle, key: String) -> Result<(), String> {
    // Remove the project from the global state
    {
        let mut projects = PROJECTS
            .lock()
            .map_err(|e| format!("Failed to lock projects: {}", e))?;
        let initial_len = projects.len();
        projects.retain(|p| p.key != key);
        if projects.len() == initial_len {
            return Err("Project not found".to_string());
        }
    }

    // Handle platform-specific closing
    #[cfg(desktop)]
    {
        use tauri::Manager;

        let window_label = format!("remote-{}", key);
        if let Some(window) = app.get_webview_window(&window_label) {
            window
                .close()
                .map_err(|e| format!("Failed to close window: {}", e))?;
        }
    }

    #[cfg(mobile)]
    {
        use tauri::Manager;
        let main_window = app
            .get_webview_window("main")
            .ok_or_else(|| "Main window not found".to_string())?;
        main_window
            .eval("window.location.href = '/';")
            .map_err(|e| format!("Failed to navigate: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_desktop_environment() -> Result<(Option<String>, Option<String>), String> {
    let xdg_current_desktop = std::env::var("XDG_CURRENT_DESKTOP").ok();
    let desktop_session = std::env::var("DESKTOP_SESSION").ok();
    Ok((xdg_current_desktop, desktop_session))
}
