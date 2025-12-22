#[cfg(target_os = "windows")]
use async_ssh2_tokio::client::AuthKeyboardInteractive;
use async_ssh2_tokio::client::AuthMethod;
use std::path::Path;

pub fn build_auth_method(
    auth_method: &str,
    password: Option<&str>,
    key_file: Option<&str>,
    #[cfg(not(target_os = "windows"))]
    public_key_file: Option<&str>,
    #[cfg(target_os = "windows")]
    _public_key_file: Option<&str>,
) -> Result<AuthMethod, String> {
    match auth_method {
        "password" => {
            if let Some(pwd) = password {
                Ok(AuthMethod::with_password(pwd))
            } else {
                Err("Password not provided".to_string())
            }
        }
        "key" => {
            if let Some(key_path) = key_file {
                if !Path::new(key_path).exists() {
                    return Err(format!("Key file not found: {}", key_path));
                }
                Ok(AuthMethod::with_key_file(key_path, None))
            } else {
                Err("Key file path not provided".to_string())
            }
        }
        // if os is not windows, support public key auth
        #[cfg(not(target_os = "windows"))]
        "public_key" => {
            if let Some(pub_key_path) = public_key_file {
                if !Path::new(pub_key_path).exists() {
                    return Err(format!("Public key file not found: {}", pub_key_path));
                }
                Ok(AuthMethod::with_public_key_file(pub_key_path))
            } else {
                Err("Public key file path not provided".to_string())
            }
        }
        #[cfg(not(target_os = "windows"))]
        "agent" => Ok(AuthMethod::with_agent()),
        #[cfg(target_os = "windows")]
        "agent" => Ok(AuthMethod::KeyboardInteractive(AuthKeyboardInteractive::new())),
        _ => Err(format!("Unknown authentication method: {}", auth_method)),
    }
}
