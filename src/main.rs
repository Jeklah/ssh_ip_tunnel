// A CLI tool to create an IP tunnel to an ARM CPU and transfer SSH keys.
// Author: Arthur Bowers
// Optimized version with async operations, proper error handling, and connection validation.

use anyhow::Result;
use backoff::ExponentialBackoff;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;
use tokio::process::Command;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

#[derive(Error, Debug)]
pub enum TunnelError {
    #[error("SSH tunnel creation failed: {0}")]
    TunnelCreation(String),
    #[error("SSH key transfer failed: {0}")]
    KeyTransfer(String),
    #[error("Connection validation failed: {0}")]
    ConnectionValidation(String),
    #[error("Timeout waiting for tunnel to be ready")]
    TunnelTimeout,
    #[error("Invalid SSH key path: {0}")]
    InvalidKeyPath(PathBuf),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_key_path: String,
    pub default_port: u16,
    pub tunnel_timeout_secs: u64,
    pub max_retries: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_key_path: "~/.ssh/id_rsa.pub".to_string(),
            default_port: 2222,
            tunnel_timeout_secs: 30,
            max_retries: 3,
        }
    }
}

/// A CLI tool to create an IP tunnel to an ARM CPU and transfer SSH keys.
#[derive(Parser, Debug)]
#[command(name = "ssh-ip-tunnel")]
#[command(about = "CLI tool for tunneling SSH and SSH key transfer", long_about = None)]
struct Cli {
    /// The IP address of the ARM CPU
    #[arg(short = 'H', long)]
    host: String,

    /// The username for SSH
    #[arg(short, long)]
    user: String,

    /// Path to the SSH key file to transfer
    #[arg(short, long)]
    key: Option<String>,

    /// Local port to bind for tunnel
    #[arg(short, long)]
    port: Option<u16>,

    /// Skip SSH key transfer
    #[arg(long)]
    no_key_transfer: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(long)]
    config: Option<PathBuf>,
}

pub struct SSHTunnelManager {
    config: Config,
}

impl SSHTunnelManager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Validates that the SSH key file exists and is readable
    fn validate_key_path(&self, key_path: &str) -> Result<PathBuf, TunnelError> {
        let expanded_path = if key_path.starts_with("~/") {
            match dirs::home_dir() {
                Some(home) => home.join(&key_path[2..]),
                None => return Err(TunnelError::InvalidKeyPath(PathBuf::from(key_path))),
            }
        } else {
            PathBuf::from(key_path)
        };

        if !expanded_path.exists() {
            return Err(TunnelError::InvalidKeyPath(expanded_path));
        }

        Ok(expanded_path)
    }

    /// Creates an SSH tunnel with proper error handling and validation
    pub async fn create_tunnel(
        &self,
        host: &str,
        user: &str,
        port: u16,
    ) -> Result<(), TunnelError> {
        info!("Creating SSH tunnel to {}@{}...", user, host);

        let tunnel_args = [
            "-fN".to_string(),
            "-L".to_string(),
            format!("{}:localhost:22", port),
            format!("{}@{}", user, host),
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
            "-o".to_string(),
            "UserKnownHostsFile=/dev/null".to_string(),
            "-o".to_string(),
            "LogLevel=ERROR".to_string(),
        ];

        debug!("Running SSH with args: {:?}", tunnel_args);

        let backoff_strategy = ExponentialBackoff {
            max_elapsed_time: Some(Duration::from_secs(self.config.tunnel_timeout_secs)),
            ..Default::default()
        };

        let operation = || async {
            let output = Command::new("ssh")
                .args(&tunnel_args)
                .output()
                .await
                .map_err(|e| {
                    backoff::Error::permanent(TunnelError::TunnelCreation(format!(
                        "Failed to execute SSH: {}",
                        e
                    )))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("SSH tunnel creation attempt failed: {}", stderr);
                return Err(backoff::Error::transient(TunnelError::TunnelCreation(
                    stderr.to_string(),
                )));
            }

            Ok(())
        };

        backoff::future::retry(backoff_strategy, operation).await?;

        info!("SSH tunnel created successfully");
        Ok(())
    }

    /// Validates that the tunnel is working by attempting a connection
    pub async fn validate_tunnel(&self, user: &str, port: u16) -> Result<(), TunnelError> {
        info!("Validating tunnel connectivity...");

        let validation_timeout = Duration::from_secs(10);

        let result = timeout(
            validation_timeout,
            Command::new("ssh")
                .args([
                    "-p",
                    &port.to_string(),
                    &format!("{}@localhost", user),
                    "-o",
                    "ConnectTimeout=5",
                    "-o",
                    "StrictHostKeyChecking=no",
                    "-o",
                    "UserKnownHostsFile=/dev/null",
                    "-o",
                    "LogLevel=ERROR",
                    "echo 'tunnel_test'",
                ])
                .output(),
        )
        .await;

        match result {
            Ok(Ok(output)) if output.status.success() => {
                info!("Tunnel validation successful");
                Ok(())
            }
            Ok(Ok(output)) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(TunnelError::ConnectionValidation(format!(
                    "Tunnel validation failed: {}",
                    stderr
                )))
            }
            Ok(Err(e)) => Err(TunnelError::ConnectionValidation(format!(
                "Failed to execute validation command: {}",
                e
            ))),
            Err(_) => Err(TunnelError::TunnelTimeout),
        }
    }

    /// Transfers SSH key through the established tunnel
    pub async fn transfer_key(
        &self,
        key_path: &str,
        user: &str,
        port: u16,
    ) -> Result<(), TunnelError> {
        let validated_key_path = self.validate_key_path(key_path)?;
        info!("Transferring SSH key: {:?}", validated_key_path);

        let output = Command::new("ssh-copy-id")
            .args([
                "-i",
                validated_key_path.to_str().unwrap(),
                &format!("-p{}", port),
                &format!("{}@localhost", user),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
            ])
            .output()
            .await
            .map_err(|e| {
                TunnelError::KeyTransfer(format!("Failed to execute ssh-copy-id: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TunnelError::KeyTransfer(stderr.to_string()));
        }

        info!("SSH key transferred successfully");
        Ok(())
    }

    /// Main orchestration method
    pub async fn run(
        &self,
        host: &str,
        user: &str,
        key_path: &str,
        port: u16,
        skip_key_transfer: bool,
    ) -> Result<()> {
        // Create tunnel
        self.create_tunnel(host, user, port).await?;

        // Wait a bit for tunnel to stabilize
        sleep(Duration::from_millis(500)).await;

        // Validate tunnel
        self.validate_tunnel(user, port).await?;

        // Transfer key if requested
        if !skip_key_transfer {
            self.transfer_key(key_path, user, port).await?;
        }

        info!("Tunnel established on localhost:{}", port);
        if !skip_key_transfer {
            info!("SSH key deployment completed successfully!");
        }

        Ok(())
    }
}

/// Load configuration from file or use defaults
fn load_config(config_path: Option<PathBuf>) -> Result<Config> {
    if let Some(path) = config_path {
        let contents = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file {:?}: {}", path, e))?;
        let config: Config = toml::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;
        Ok(config)
    } else {
        // Try to load from default location
        if let Some(config_dir) = dirs::config_dir() {
            let default_config_path = config_dir.join("ssh_ip_tunnel").join("config.toml");
            if default_config_path.exists() {
                let contents = std::fs::read_to_string(&default_config_path)?;
                let config: Config = toml::from_str(&contents)?;
                return Ok(config);
            }
        }
        Ok(Config::default())
    }
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: bool) {
    let log_level = if verbose { "debug" } else { "info" };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("ssh_ip_tunnel={}", log_level).into()),
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    init_logging(cli.verbose);

    let config = load_config(cli.config)?;

    let key_path = cli.key.unwrap_or(config.default_key_path.clone());
    let port = cli.port.unwrap_or(config.default_port);

    let tunnel_manager = SSHTunnelManager::new(config);

    tunnel_manager
        .run(&cli.host, &cli.user, &key_path, port, cli.no_key_transfer)
        .await
        .map_err(|e| {
            error!("Operation failed: {}", e);
            e
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.default_port, 2222);
        assert_eq!(config.default_key_path, "~/.ssh/id_rsa.pub");
    }

    #[tokio::test]
    async fn test_key_path_validation() {
        let config = Config::default();
        let manager = SSHTunnelManager::new(config);

        // Test invalid path
        let result = manager.validate_key_path("/nonexistent/key.pub");
        assert!(result.is_err());
    }
}
