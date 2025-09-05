# SSH IP Tunnel v2.0

A high-performance CLI tool to create SSH tunnels to ARM CPUs and transfer SSH keys automatically. Built with modern Rust for speed, reliability, and safety. This tool simplifies the process of setting up SSH tunnels and deploying SSH keys to remote ARM devices with enterprise-grade features.

## ✨ Features

### **Core Functionality**
- **SSH Tunnel Creation**: Establishes secure SSH tunnels with local port forwarding
- **Automatic Key Transfer**: Deploys SSH public keys using `ssh-copy-id`
- **Connection Validation**: Actively tests tunnel connectivity before proceeding
- **Flexible Configuration**: Support for custom ports, keys, and SSH options

### **Advanced Features**
- **Async Operations**: Non-blocking I/O using Tokio for better performance
- **Retry Logic**: Exponential backoff for robust tunnel establishment
- **Structured Logging**: Configurable logging with debug/info/warn/error levels
- **Configuration Files**: TOML-based configuration with intelligent defaults
- **Path Validation**: Secure handling of SSH key paths with expansion
- **Error Handling**: Comprehensive error types with detailed diagnostic context
- **Cross-platform**: Tested on Linux, macOS, and Windows

### **Enterprise Ready**
- **Production Logging**: Structured logs suitable for monitoring systems
- **Timeout Management**: Configurable timeouts for different operations
- **Security Hardening**: Secure SSH options and input validation
- **Integration Friendly**: Clean exit codes and structured output for automation

## Installation

### From Source

```bash
# Clone the repository
git clone <repository-url>
cd ssh_ip_tunnel

# Build the project
cargo build --release

# The binary will be available at target/release/ssh_ip_tunnel
```

### Prerequisites

Make sure you have the following installed on your system:
- `ssh` client
- `ssh-copy-id` utility (usually comes with OpenSSH)
- Rust toolchain (for building from source)

## Usage

### Basic Command

```bash
ssh_ip_tunnel --host <ARM_IP> --user <USERNAME> [OPTIONS]
```

### Options

#### **Required Arguments**
- `-H, --host <HOST>` - IP address or hostname of the target device
- `-u, --user <USER>` - SSH username for authentication

#### **Optional Arguments**
- `-k, --key <KEY>` - Path to SSH public key file (default: from config or `~/.ssh/id_rsa.pub`)
- `-p, --port <PORT>` - Local port for tunnel (default: from config or `2222`)

#### **Feature Flags**
- `--no-key-transfer` - Create tunnel only, skip SSH key deployment
- `-v, --verbose` - Enable detailed logging output for debugging

#### **Configuration**
- `--config <CONFIG>` - Path to custom configuration file
- `-h, --help` - Display help information and exit

#### **Environment Variables**
- `RUST_LOG` - Set log level (debug, info, warn, error)

### Examples

```bash
# Basic usage with Raspberry Pi
ssh_ip_tunnel --host 192.168.1.42 --user pi

# Custom SSH key and port
ssh_ip_tunnel --host 192.168.1.100 --user ubuntu --key ~/.ssh/my_key.pub --port 3333

# Tunnel only (no key transfer)
ssh_ip_tunnel --host 10.0.0.50 --user root --no-key-transfer

# Verbose logging for debugging
ssh_ip_tunnel --host 192.168.1.42 --user pi --verbose

# Using custom configuration file
ssh_ip_tunnel --host 192.168.1.42 --user pi --config /path/to/config.toml

# Short form with all options
ssh_ip_tunnel -H 10.0.0.50 -u root -k ~/.ssh/id_ed25519.pub -p 2200 -v
```

## How It Works

1. **Configuration Loading**: Loads settings from config file or uses intelligent defaults
2. **Path Validation**: Validates and expands SSH key paths (handles `~` notation)
3. **SSH Tunnel Creation**: Establishes tunnel using secure SSH options with exponential backoff retry
4. **Connection Validation**: Actively tests tunnel connectivity before proceeding (replaces fixed delays)
5. **Key Transfer**: Transfers SSH public key through the validated tunnel using `ssh-copy-id`
6. **Error Handling**: Provides comprehensive error diagnostics with structured logging

### **Technical Flow**
- **Async Runtime**: All operations run on Tokio async runtime for non-blocking I/O
- **Retry Logic**: Failed operations automatically retry with exponential backoff
- **Timeout Management**: Each operation has appropriate timeout limits
- **Security**: Uses hardened SSH options and validates all file paths

## Configuration

### **Configuration File Locations**
The tool looks for configuration files in the following order:
1. Path specified with `--config` flag
2. `~/.config/ssh_ip_tunnel/config.toml` (user config)
3. Built-in defaults

### **Configuration Format**
Create a configuration file using TOML format:

```toml
# Default SSH key path when none is specified
default_key_path = "~/.ssh/id_rsa.pub"

# Default local port for SSH tunnels
default_port = 2222

# Timeout in seconds to wait for tunnel establishment
tunnel_timeout_secs = 30

# Maximum number of retry attempts for tunnel creation
max_retries = 3
```

### **Configuration Schema**
| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `default_key_path` | String | `"~/.ssh/id_rsa.pub"` | Default SSH public key path |
| `default_port` | Integer | `2222` | Default local tunnel port |
| `tunnel_timeout_secs` | Integer | `30` | Tunnel establishment timeout |
| `max_retries` | Integer | `3` | Maximum retry attempts |

### **Example Configuration**
Copy `config.toml.example` to your config directory:
```bash
mkdir -p ~/.config/ssh_ip_tunnel
cp config.toml.example ~/.config/ssh_ip_tunnel/config.toml
```

## Integration Examples

### Python Script Example

Simple example of using the SSH tunnel tool from Python:

```python
import subprocess

# Basic usage with error handling
try:
    result = subprocess.run([
        "ssh_ip_tunnel",
        "--host", "192.168.1.42",
        "--user", "pi",
        "--verbose"  # Enable detailed logging
    ], check=True, capture_output=True, text=True)
    print("SSH tunnel and key transfer successful!")
    print(result.stdout)
except subprocess.CalledProcessError as e:
    print(f"Failed to set up tunnel: {e.stderr}")

# Advanced usage with custom config
subprocess.run([
    "ssh_ip_tunnel",
    "--host", "192.168.1.100",
    "--user", "ubuntu",
    "--key", "~/.ssh/id_ed25519.pub",
    "--port", "3333",
    "--config", "/path/to/custom/config.toml"
], check=True)
```

### Bash Script Example

```bash
#!/bin/bash
# deploy_to_farm.sh - Deploy to multiple ARM devices

set -e

DEVICES=(
    "192.168.1.42:pi"
    "192.168.1.43:pi"
    "192.168.1.100:ubuntu"
)

echo "Deploying to ${#DEVICES[@]} devices..."

for i in "${!DEVICES[@]}"; do
    IFS=':' read -r HOST USER <<< "${DEVICES[$i]}"
    PORT=$((2222 + i))

    echo "[$((i+1))/${#DEVICES[@]}] Setting up tunnel to $USER@$HOST..."

    if ssh_ip_tunnel --host "$HOST" --user "$USER" --port "$PORT"; then
        echo "✓ Tunnel established, deploying code..."

        # Deploy your code through the tunnel
        scp -P "$PORT" -r my_project/ "$USER@localhost:/home/$USER/"
        ssh -p "$PORT" "$USER@localhost" "cd my_project && ./install.sh"

        echo "✓ Deployment to $HOST complete"
    else
        echo "✗ Failed to set up tunnel to $HOST"
    fi
done

echo "🎉 All deployments complete!"
```

## Technical Architecture

### **Built with Modern Rust**
This tool showcases advanced Rust features and best practices:

#### **Async/Await Operations**
```rust
#[tokio::main]
async fn main() -> Result<()> {
    tunnel_manager.run(&host, &user, &key_path, port, skip_transfer).await
}
```
- Non-blocking I/O operations using Tokio runtime
- Concurrent operations where beneficial
- Better resource utilization and responsiveness

#### **Advanced Error Handling**
```rust
#[derive(Error, Debug)]
pub enum TunnelError {
    #[error("SSH tunnel creation failed: {0}")]
    TunnelCreation(String),
    #[error("Connection validation failed: {0}")]
    ConnectionValidation(String),
    // ... more specific error types
}
```
- Custom error types with `thiserror` for rich error context
- Type-safe error handling throughout the application
- Detailed diagnostic information for debugging

#### **Retry Logic with Exponential Backoff**
```rust
let backoff_strategy = ExponentialBackoff {
    max_elapsed_time: Some(Duration::from_secs(timeout)),
    ..Default::default()
};
backoff::future::retry(backoff_strategy, operation).await
```
- Automatic retry for transient failures
- Exponential backoff prevents overwhelming remote systems
- Configurable timeout and retry limits

#### **Structured Logging**
```rust
use tracing::{debug, info, warn, error};
info!("Creating SSH tunnel to {}@{}", user, host);
debug!("Running SSH with args: {:?}", args);
```
- Production-ready logging with `tracing` crate
- Configurable log levels (debug/info/warn/error)
- Structured output suitable for log aggregation systems

#### **Type-Safe Configuration**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_key_path: String,
    pub default_port: u16,
    pub tunnel_timeout_secs: u64,
}
```
- TOML configuration with `serde` for type safety
- Hierarchical configuration (CLI > config file > defaults)
- Cross-platform path handling with proper expansion

## Troubleshooting

### **Common Issues & Solutions**

#### **1. SSH Tunnel Creation Failed**
**Error**: `SSH tunnel creation failed: <details>`

**Solutions**:
- Verify the host IP address is correct and reachable: `ping <host>`
- Ensure SSH is running on target: `nc -zv <host> 22`
- Check SSH access: `ssh <user>@<host>`
- Review firewall settings on both local and remote machines
- Try with verbose logging: `--verbose`

#### **2. SSH Key Transfer Failed** 
**Error**: `SSH key transfer failed: <details>`

**Solutions**:
- Verify SSH key file exists: `ls -la ~/.ssh/id_rsa.pub`
- Ensure target user account exists
- Check if password authentication is enabled on target
- Verify key file permissions: `chmod 644 ~/.ssh/id_rsa.pub`

#### **3. Connection Validation Failed**
**Error**: `Connection validation failed: <details>`

**Solutions**:
- Check network connectivity: `telnet localhost <port>`
- Verify no other service is using the local port
- Ensure target SSH service accepts connections
- Check for intermediate firewalls or NAT issues

#### **4. Tunnel Timeout**
**Error**: `Timeout waiting for tunnel to be ready`

**Solutions**:
- Increase timeout in config: `tunnel_timeout_secs = 60`
- Check network latency: `ping <host>`
- Verify stable network connection
- Try a different local port: `--port <different_port>`

#### **5. Invalid SSH Key Path**
**Error**: `Invalid SSH key path: <path>`

**Solutions**:
- Check file exists: `ls -la <path>`
- Use absolute path instead of relative
- Verify file permissions are readable
- Generate key if missing: `ssh-keygen -t rsa`

### **Debugging Tools**

#### **Verbose Logging**
```bash
# Enable detailed logging
ssh_ip_tunnel --host 192.168.1.42 --user pi --verbose

# Or use environment variable for even more detail
RUST_LOG=debug ssh_ip_tunnel --host 192.168.1.42 --user pi
```

#### **Configuration Testing**
```bash
# Test with custom config
ssh_ip_tunnel --config ./debug.toml --host 192.168.1.42 --user pi

# Skip key transfer for tunnel testing
ssh_ip_tunnel --host 192.168.1.42 --user pi --no-key-transfer
```

#### **Manual Validation**
```bash
# Test SSH connectivity manually
ssh <user>@<host>

# Test tunnel manually
ssh -L 2222:localhost:22 <user>@<host>

# Test through tunnel
ssh -p 2222 <user>@localhost
```

### **Log Analysis**
The tool provides structured logs that can help identify issues:

```
INFO  Creating SSH tunnel to pi@192.168.1.42...
DEBUG Running SSH with args: ["-fN", "-L", "2222:localhost:22", "pi@192.168.1.42"]
INFO  SSH tunnel created successfully
INFO  Validating tunnel connectivity...
WARN  Tunnel validation attempt failed, retrying...
INFO  Tunnel validation successful
INFO  Transferring SSH key: "/home/user/.ssh/id_rsa.pub"
INFO  SSH key transferred successfully
```
