use crate::models::DirEntry;
use crate::state::PROJECTS;
use std::sync::Arc;

#[tauri::command]
pub async fn get_dir_contents(key: String, path: String) -> Result<Vec<DirEntry>, String> {
    let connection = {
        let projects = PROJECTS
            .lock()
            .map_err(|e| format!("Failed to lock projects: {}", e))?;

        let project = projects
            .iter()
            .find(|p| p.key == key)
            .ok_or_else(|| "Project not found".to_string())?;

        Arc::clone(&project.main_connection)
    };

    // Use ls -la to get detailed directory listing
    let command = format!("cd '{}' && ls -la --time-style=long-iso", path);
    let result = connection
        .execute(&command)
        .await
        .map_err(|e| format!("Failed to execute ls: {}", e))?;

    if result.exit_status != 0 {
        return Err(format!("Command failed: {}", result.stderr));
    }

    let mut entries = Vec::new();

    for line in result.stdout.lines().skip(1) {
        // Skip "total" line
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.len() < 8 {
            continue;
        }

        let permissions = parts[0].to_string();
        let name_and_target = parts[7..].join(" ");

        // Skip . and .. entries
        if name_and_target == "." || name_and_target == ".." {
            continue;
        }

        // Check if this is a symlink
        if permissions.starts_with('l') {
            // Parse symlink format: "name -> target"
            if let Some(arrow_pos) = name_and_target.find(" -> ") {
                let name = name_and_target[..arrow_pos].trim().to_string();
                let target = name_and_target[arrow_pos + 4..].trim().to_string();
                
                // Resolve the target path (handle relative paths)
                let resolved_target = if target.starts_with('/') {
                    target.clone()
                } else {
                    format!("{}/{}", path, target)
                };

                // Get the stats of the target file/directory
                match get_target_stats(&connection, &resolved_target).await {
                    Ok((is_dir, size, modified)) => {
                        entries.push(DirEntry {
                            name,
                            is_dir,
                            size,
                            permissions,
                            modified,
                        });
                    }
                    Err(_) => {
                        // If we can't stat the target (broken symlink), skip it or handle as needed
                        // For now, we'll add it as a file with unknown stats
                        entries.push(DirEntry {
                            name,
                            is_dir: false,
                            size: "?".to_string(),
                            permissions,
                            modified: "?".to_string(),
                        });
                    }
                }
            }
        } else {
            // Regular file or directory
            let size = parts[4].to_string();
            let modified = format!("{} {}", parts[5], parts[6]);
            let name = name_and_target;
            let is_dir = permissions.starts_with('d');

            entries.push(DirEntry {
                name,
                is_dir,
                size,
                permissions,
                modified,
            });
        }
    }

    Ok(entries)
}

// Helper function to get stats of the symlink target
async fn get_target_stats(
    connection: &Arc<async_ssh2_tokio::client::Client>,
    target_path: &str,
) -> Result<(bool, String, String), String> {
    let command = format!("stat -L --format='%F|%s|%y' '{}'", target_path);
    let result = connection
        .execute(&command)
        .await
        .map_err(|e| format!("Failed to stat target: {}", e))?;

    if result.exit_status != 0 {
        return Err(format!("Failed to stat target: {}", result.stderr));
    }

    let output = result.stdout.trim();
    let parts: Vec<&str> = output.split('|').collect();
    
    if parts.len() < 3 {
        return Err("Invalid stat output".to_string());
    }

    let file_type = parts[0];
    let size = parts[1].to_string();
    let modified = parts[2].split('.').next().unwrap_or("").to_string();
    
    let is_dir = file_type.contains("directory");

    Ok((is_dir, size, modified))
}

#[tauri::command]
pub async fn create_file(key: String, path: String) -> Result<(), String> {
    let connection = {
        let projects = PROJECTS
            .lock()
            .map_err(|e| format!("Failed to lock projects: {}", e))?;

        let project = projects
            .iter()
            .find(|p| p.key == key)
            .ok_or_else(|| "Project not found".to_string())?;

        Arc::clone(&project.main_connection)
    };

    let command = format!("touch '{}'", path);
    let result = connection
        .execute(&command)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    if result.exit_status == 0 {
        Ok(())
    } else {
        Err(format!("Command failed: {}", result.stderr))
    }
}

#[tauri::command]
pub async fn create_folder(key: String, path: String) -> Result<(), String> {
    let connection = {
        let projects = PROJECTS
            .lock()
            .map_err(|e| format!("Failed to lock projects: {}", e))?;

        let project = projects
            .iter()
            .find(|p| p.key == key)
            .ok_or_else(|| "Project not found".to_string())?;

        Arc::clone(&project.main_connection)
    };

    let command = format!("mkdir -p '{}'", path);
    let result = connection
        .execute(&command)
        .await
        .map_err(|e| format!("Failed to create folder: {}", e))?;

    if result.exit_status == 0 {
        Ok(())
    } else {
        Err(format!("Command failed: {}", result.stderr))
    }
}

#[tauri::command]
pub async fn delete_item(key: String, path: String, is_dir: bool) -> Result<(), String> {
    let connection = {
        let projects = PROJECTS
            .lock()
            .map_err(|e| format!("Failed to lock projects: {}", e))?;

        let project = projects
            .iter()
            .find(|p| p.key == key)
            .ok_or_else(|| "Project not found".to_string())?;

        Arc::clone(&project.main_connection)
    };

    let command = if is_dir {
        format!("rm -rf '{}'", path)
    } else {
        format!("rm '{}'", path)
    };

    let result = connection
        .execute(&command)
        .await
        .map_err(|e| format!("Failed to delete item: {}", e))?;

    if result.exit_status == 0 {
        Ok(())
    } else {
        Err(format!("Command failed: {}", result.stderr))
    }
}