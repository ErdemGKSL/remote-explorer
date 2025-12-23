use async_ssh2_tokio::client::{Client, ServerCheckMethod};
use std::time::Duration;
use tokio::time::timeout;

pub struct HostPort {
    pub hostname: String,
    pub port: u16,
}
pub fn parse_host_port(host: &str) -> HostPort {
    if host.contains(':') {
        let parts: Vec<&str> = host.splitn(2, ':').collect();
        let port = parts[1].parse::<u16>().unwrap_or(22);
        HostPort {
            hostname: parts[0].to_string(),
            port,
        }
    } else {
        HostPort {
            hostname: host.to_string(),
            port: 22,
        }
    }
}

pub async fn connect_to_ssh(
    hostname: &str,
    port: u16,
    user: &str,
    auth_method: async_ssh2_tokio::client::AuthMethod,
    timeout_secs: u64,
) -> Result<Client, String> {
    let client = Client::connect(
        (hostname, port),
        user,
        auth_method,
        ServerCheckMethod::NoCheck,
    );

    timeout(Duration::from_secs(timeout_secs), client)
        .await
        .map_err(|e| format!("SSH connection failed - {}", e))?
        .map_err(|e| format!("SSH connection failed - {}", e))
}
