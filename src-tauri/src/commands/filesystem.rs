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

        let permissions = parts[0].to_string();
        let size = parts[4].to_string();
        let modified = format!("{} {}", parts[5], parts[6]);
        let name = parts[7..].join(" ");

        // Skip . and .. entries
        if name == "." || name == ".." {
            continue;
        }

        let is_dir = permissions.starts_with('d');

        entries.push(DirEntry {
            name,
            is_dir,
            size,
            permissions,
            modified,
        });
    }

    Ok(entries)
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
