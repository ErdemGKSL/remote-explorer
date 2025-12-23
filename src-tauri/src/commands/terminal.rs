use crate::auth::build_auth_method;
use crate::models::TerminalConnection;
use crate::ssh::{connect_to_ssh, parse_host_port};
use crate::state::get_project_by_key;
use std::sync::Arc;
use uuid::Uuid;

#[tauri::command]
pub async fn create_terminal(
    key: String,
    name: String,
    path: String,
) -> Result<String, String> {
    let project = get_project_by_key(&key)?;

    let host_port = parse_host_port(&project.host);
    let auth = build_auth_method(
        &project.auth_method,
        project.password.as_deref(),
        project.key_file.as_deref(),
        project.public_key_file.as_deref(),
    )?;

    // Extract user from host (e.g., "user@hostname") or default to "root"
    let user = if project.host.contains('@') {
        project.host.split('@').collect::<Vec<_>>()[0].to_string()
    } else {
        "root".to_string()
    };

    let client = connect_to_ssh(&host_port.hostname, host_port.port, &user, auth, 10).await?;

    // Change to the specified directory
    let cd_result = client
        .execute(&format!("cd {} && pwd", path))
        .await
        .map_err(|e| format!("Failed to change directory: {}", e))?;

    if cd_result.exit_status != 0 {
        return Err(format!("Failed to navigate to path: {}", path));
    }

    let terminal_id = Uuid::new_v4().to_string();

    let terminal_connection = TerminalConnection {
        id: terminal_id.clone(),
        connection: Arc::new(client),
        name,
        path,
    };

    // Add terminal connection to the project
    {
        let mut terminals = project
            .terminal_connections
            .lock()
            .await;

        terminals.push(terminal_connection);
    }

    Ok(terminal_id)
}

#[tauri::command]
pub async fn execute_terminal_command(
    key: String,
    terminal_id: String,
    command: String,
) -> Result<(String, String, i32), String> {
    let project = get_project_by_key(&key)?;

    let connection = {
        let terminals = project
            .terminal_connections
            .lock()
            .await;

        let terminal = terminals
            .iter()
            .find(|t| t.id == terminal_id)
            .ok_or_else(|| "Terminal not found".to_string())?;

        Arc::clone(&terminal.connection)
    };

    let result = connection
        .execute(&command)
        .await
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    Ok((result.stdout, result.stderr, result.exit_status as i32))
}

#[tauri::command]
pub async fn close_terminal(key: String, terminal_id: String) -> Result<(), String> {
    let project = get_project_by_key(&key)?;

    let mut terminals = project
        .terminal_connections
        .lock()
        .await;

    let initial_len = terminals.len();

    terminals.retain(|t| t.id != terminal_id);

    if terminals.len() == initial_len {
        return Err("Terminal not found".to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn list_terminals(key: String) -> Result<Vec<(String, String, String)>, String> {
    let project = get_project_by_key(&key)?;

    let terminals = project
        .terminal_connections
        .lock()
        .await;

    let result: Vec<(String, String, String)> = terminals
        .iter()
        .map(|t| (t.id.clone(), t.name.clone(), t.path.clone()))
        .collect();

    Ok(result)
}

#[tauri::command]
pub async fn get_terminal_pwd(key: String, terminal_id: String) -> Result<String, String> {
    let project = get_project_by_key(&key)?;

    let connection = {
        let terminals = project
            .terminal_connections
            .lock()
            .await;

        let terminal = terminals
            .iter()
            .find(|t| t.id == terminal_id)
            .ok_or_else(|| "Terminal not found".to_string())?;

        Arc::clone(&terminal.connection)
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