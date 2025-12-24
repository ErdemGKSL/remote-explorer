use async_ssh2_tokio::client::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, mpsc};
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone)]
pub struct TerminalConnection {
    pub id: String,
    pub connection: Arc<Client>,
    pub content_lines: Arc<Mutex<String>>, // Changed from Vec<String> to String
    pub current_executions: Arc<Mutex<Vec<TerminalExecution>>>,
    pub path: String,
}

#[allow(dead_code)]
pub struct TerminalExecution {
    pub stdin_tx: mpsc::Sender<Vec<u8>>,
    pub command: String,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Project {
    pub key: String,
    pub name: String,
    pub user: String,
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
    pub user: String,
    pub host: String,
    pub auth_method: String,
}

#[derive(Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_linked: bool,
    pub size: String,
    pub permissions: String,
    pub modified: String,
}