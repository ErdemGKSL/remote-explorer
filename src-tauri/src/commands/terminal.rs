use crate::auth::build_auth_method;
use crate::models::{TerminalConnection, TerminalExecution};
use crate::ssh::{connect_to_ssh, parse_host_port};
use crate::state::get_project_by_key;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

// Struct to track running executions
#[tauri::command]
pub async fn create_terminal(
    key: String,
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

    let user = project.user.clone();

    let client = connect_to_ssh(&host_port.hostname, host_port.port, &user, auth, 10).await?;

    // Verify the path exists
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
        content_lines: Arc::new(Mutex::new(Vec::new())),
        current_executions: Arc::new(Mutex::new(Vec::new())),
        path: path.clone(),
    };

    // Start an interactive shell session
    let connection = Arc::clone(&terminal_connection.connection);
    let content_lines = Arc::clone(&terminal_connection.content_lines);
    let current_executions = Arc::clone(&terminal_connection.current_executions);

    tauri::async_runtime::spawn(async move {
        let (stdout_tx, mut stdout_rx) = mpsc::channel::<Vec<u8>>(100);
        let (stdin_tx, stdin_rx) = mpsc::channel::<Vec<u8>>(100);

        // Store the stdin sender for this execution
        {
            let mut executions = current_executions.lock().await;
            executions.push(TerminalExecution {
                stdin_tx: stdin_tx.clone(),
                command: "shell".to_string(),
            });
        }

        let content_lines_clone = Arc::clone(&content_lines);
        let current_executions_clone = Arc::clone(&current_executions);

        // Spawn task to collect stdout
        tauri::async_runtime::spawn(async move {
            while let Some(data) = stdout_rx.recv().await {
                let text = String::from_utf8_lossy(&data).to_string();
                let mut lines = content_lines_clone.lock().await;
                
                // Split by newlines and add each line
                for line in text.split('\n') {
                    if !line.is_empty() || text.ends_with('\n') {
                        lines.push(line.to_string());
                    }
                }
            }
        });

        // Start an interactive shell with PTY
        let shell_command = format!("cd {} && exec bash -i", path);
        let result = connection
            .execute_io(
                &shell_command,
                stdout_tx,
                None, // No stderr channel - PTY sends everything to stdout
                Some(stdin_rx),
                true, // request_pty = true for interactive shell
                Some(0),
            )
            .await;

        // When shell exits, remove from executions
        let mut executions = current_executions_clone.lock().await;
        executions.clear();

        if let Err(e) = result {
            let mut lines = content_lines.lock().await;
            lines.push(format!("Shell exited with error: {}", e));
        }
    });

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
) -> Result<(), String> {
    let project = get_project_by_key(&key)?;

    let current_executions = {
        let terminals = project
            .terminal_connections
            .lock()
            .await;

        let terminal = terminals
            .iter()
            .find(|t| t.id == terminal_id)
            .ok_or_else(|| "Terminal not found".to_string())?;

        Arc::clone(&terminal.current_executions)
    };

    let executions = current_executions.lock().await;

    // Get the last execution (the interactive shell)
    if let Some(last_exec) = executions.last() {
        // Send command to stdin with newline
        let command_with_newline = format!("{}\n", command);
        last_exec
            .stdin_tx
            .send(command_with_newline.into_bytes())
            .await
            .map_err(|e| format!("Failed to send command to shell: {}", e))?;
        
        Ok(())
    } else {
        Err("No active shell session".to_string())
    }
}

#[tauri::command]
pub async fn send_terminal_input(
    key: String,
    terminal_id: String,
    input: String,
) -> Result<(), String> {
    let project = get_project_by_key(&key)?;

    let current_executions = {
        let terminals = project
            .terminal_connections
            .lock()
            .await;

        let terminal = terminals
            .iter()
            .find(|t| t.id == terminal_id)
            .ok_or_else(|| "Terminal not found".to_string())?;

        Arc::clone(&terminal.current_executions)
    };

    let executions = current_executions.lock().await;

    // Get the last execution
    if let Some(last_exec) = executions.last() {
        // Send raw input (useful for Ctrl+C, etc.)
        last_exec
            .stdin_tx
            .send(input.into_bytes())
            .await
            .map_err(|e| format!("Failed to send input to shell: {}", e))?;
        
        Ok(())
    } else {
        Err("No active shell session".to_string())
    }
}

#[tauri::command]
pub async fn close_terminal(key: String, terminal_id: String) -> Result<(), String> {
    let project = get_project_by_key(&key)?;

    let mut terminals = project
        .terminal_connections
        .lock()
        .await;

    let initial_len = terminals.len();

    // Before removing, try to close the shell gracefully
    if let Some(terminal) = terminals.iter().find(|t| t.id == terminal_id) {
        let executions = terminal.current_executions.lock().await;
        if let Some(last_exec) = executions.last() {
            // Send exit command
            let _ = last_exec.stdin_tx.send(b"exit\n".to_vec()).await;
        }
    }

    terminals.retain(|t| t.id != terminal_id);

    if terminals.len() == initial_len {
        return Err("Terminal not found".to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn list_terminals(key: String) -> Result<Vec<(String, String)>, String> {
    let project = get_project_by_key(&key)?;

    let terminals = project
        .terminal_connections
        .lock()
        .await;

    let result: Vec<(String, String)> = terminals
        .iter()
        .map(|t| (t.id.clone(), t.path.clone()))
        .collect();

    Ok(result)
}

#[tauri::command]
pub async fn get_terminal_pwd(key: String, terminal_id: String) -> Result<String, String> {
    let project = get_project_by_key(&key)?;

    let current_executions = {
        let terminals = project
            .terminal_connections
            .lock()
            .await;

        let terminal = terminals
            .iter()
            .find(|t| t.id == terminal_id)
            .ok_or_else(|| "Terminal not found".to_string())?;

        Arc::clone(&terminal.current_executions)
    };

    let executions = current_executions.lock().await;

    if let Some(last_exec) = executions.last() {
        // Send pwd command
        last_exec
            .stdin_tx
            .send(b"pwd\n".to_vec())
            .await
            .map_err(|e| format!("Failed to send pwd command: {}", e))?;
        
        // Note: The output will appear in content_lines
        // You might want to implement a more sophisticated way to capture this
        Ok("Check content_lines for result".to_string())
    } else {
        Err("No active shell session".to_string())
    }
}

#[tauri::command]
pub async fn get_terminal_content(key: String, terminal_id: String) -> Result<Vec<String>, String> {
    let project = get_project_by_key(&key)?;

    let content_lines = {
        let terminals = project
            .terminal_connections
            .lock()
            .await;

        let terminal = terminals
            .iter()
            .find(|t| t.id == terminal_id)
            .ok_or_else(|| "Terminal not found".to_string())?;

        Arc::clone(&terminal.content_lines)
    };

    let lines = content_lines.lock().await;
    Ok(lines.clone())
}

#[tauri::command]
pub async fn clear_terminal_content(key: String, terminal_id: String) -> Result<(), String> {
    let project = get_project_by_key(&key)?;

    let content_lines = {
        let terminals = project
            .terminal_connections
            .lock()
            .await;

        let terminal = terminals
            .iter()
            .find(|t| t.id == terminal_id)
            .ok_or_else(|| "Terminal not found".to_string())?;

        Arc::clone(&terminal.content_lines)
    };

    let mut lines = content_lines.lock().await;
    lines.clear();
    Ok(())
}