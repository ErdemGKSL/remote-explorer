use crate::models::Project;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref PROJECTS: Mutex<Vec<Arc<Project>>> = Mutex::new(Vec::new());
}

pub fn get_project_by_key(key: &str) -> Result<Arc<Project>, String> {
    let projects = PROJECTS
        .lock()
        .map_err(|e| format!("Failed to lock projects: {}", e))?;

    projects
        .iter()
        .find(|p| p.key == key)
        .cloned()
        .ok_or_else(|| "Project not found".to_string())
}

pub fn add_project(project: Project) -> Result<(), String> {
    PROJECTS
        .lock()
        .map_err(|e| format!("Failed to lock projects: {}", e))?
        .push(Arc::new(project));
    Ok(())
}

pub fn remove_project_by_key(key: &str) -> Result<(), String> {
    let mut projects = PROJECTS
        .lock()
        .map_err(|e| format!("Failed to lock projects: {}", e))?;

    projects.retain(|project| project.key != key);
    Ok(())
}