use crate::models::Project;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref PROJECTS: Mutex<Vec<Project>> = Mutex::new(Vec::new());
}

pub fn get_project_by_key(key: &str) -> Result<std::sync::Arc<crate::models::Project>, String> {
    let projects = PROJECTS
        .lock()
        .map_err(|e| format!("Failed to lock projects: {}", e))?;

    for project in projects.iter() {
        if project.key == key {
            return Ok(std::sync::Arc::new(project.clone()));
        }
    }
    Err("Project not found".to_string())
}

pub fn add_project(project: Project) -> Result<(), String> {
    PROJECTS
        .lock()
        .map_err(|e| format!("Failed to lock projects: {}", e))?
        .push(project);
    Ok(())
}

pub fn remove_project_by_key(key: &str) -> Result<(), String> {
    let mut projects = PROJECTS
        .lock()
        .map_err(|e| format!("Failed to lock projects: {}", e))?;

    projects.retain(|project| project.key != key);
    Ok(())
}