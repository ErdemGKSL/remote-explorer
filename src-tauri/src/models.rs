use async_ssh2_tokio::client::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone)]
pub struct TerminalConnection {
    pub id: String,
    pub connection: Arc<Client>,
    pub name: String,
    pub path: String, // this is a start path, if user changes it by cd command, this path is not updated
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Project {
    pub key: String,
    pub name: String,
    pub host: String,
    pub password: Option<String>,
    pub key_file: Option<String>,
    pub public_key_file: Option<String>,
    pub auth_method: String,
    pub main_connection: Arc<Client>,
    pub terminal_connections: Arc<Mutex<Vec<TerminalConnection>>>,
}

#[derive(Serialize)]
pub struct ProjectInfo {
    pub key: String,
    pub name: String,
    pub host: String,
    pub auth_method: String,
}

#[derive(Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: String,
    pub permissions: String,
    pub modified: String,
}
