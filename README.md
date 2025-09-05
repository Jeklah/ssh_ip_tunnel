# SSH IP Tunnel

A high-performance CLI tool to create an IP tunnel to an ARM CPU and transfer SSH keys automatically. Built with Rust for speed, reliability, and safety. This tool simplifies the process of setting up SSH tunnels and deploying SSH keys to remote ARM devices with advanced features like async operations, retry logic, and connection validation.

## Features

- **Async Operations**: Non-blocking tunnel creation and validation for better performance
- **Connection Validation**: Actively verifies tunnel connectivity before proceeding
- **Retry Logic**: Exponential backoff retry for robust tunnel establishment
- **Structured Logging**: Configurable logging with different verbosity levels
- **Configuration Files**: TOML-based configuration with sensible defaults
- **Error Handling**: Comprehensive error types with detailed context
- **Security**: Path validation and secure SSH option handling
- **Cross-platform**: Works on Linux, macOS, and Windows

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

- `-H, --host <HOST>` - IP address of the ARM CPU (required)
- `-u, --user <USER>` - SSH username (required)
- `-k, --key <KEY>` - Path to SSH key file (configurable default)
- `-p, --port <PORT>` - Local port for tunnel (configurable default)
- `--no-key-transfer` - Skip SSH key transfer, only create tunnel
- `-v, --verbose` - Enable verbose logging for debugging
- `--config <CONFIG>` - Path to configuration file
- `-h, --help` - Show help information

### Examples

```bash
# Basic usage with Raspberry Pi
ssh_ip_tunnel --host 192.168.1.42 --user pi

# Using custom SSH key and port
ssh_ip_tunnel --host 192.168.1.100 --user ubuntu --key ~/.ssh/my_key.pub --port 3333

# Short form
ssh_ip_tunnel -H 10.0.0.50 -u root -k ~/.ssh/id_ed25519.pub -p 2200
```

## How It Works

1. **Configuration Loading**: Loads settings from config file or uses sensible defaults
2. **SSH Tunnel Creation**: Creates an SSH tunnel using secure SSH options with retry logic
3. **Connection Validation**: Actively tests tunnel connectivity with timeout handling
4. **Key Transfer**: Validates SSH key path and transfers it through the established tunnel
5. **Error Handling**: Provides detailed error messages with proper error propagation

## Configuration

Create a configuration file at `~/.config/ssh_ip_tunnel/config.toml`:

```toml
# Default SSH key path to use when none is specified
default_key_path = "~/.ssh/id_rsa.pub"

# Default local port for SSH tunnels
default_port = 2222

# Timeout in seconds to wait for tunnel establishment
tunnel_timeout_secs = 30

# Maximum number of retry attempts for tunnel creation
max_retries = 3
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

## Rust Optimizations

This tool leverages several advanced Rust features for optimal performance and reliability:

### **Async/Await Operations**
- Non-blocking SSH operations using Tokio
- Concurrent tunnel validation and setup
- Better resource utilization

### **Advanced Error Handling**
- Custom error types with `thiserror` for detailed error context
- Proper error propagation and chaining
- Structured error messages for debugging

### **Retry Logic with Backoff**
- Exponential backoff for tunnel creation failures
- Configurable timeout and retry limits
- Resilient against temporary network issues

### **Structured Logging**
- Configurable log levels with `tracing`
- JSON-structured logs for automation
- Performance-oriented logging with minimal overhead

### **Configuration Management**
- TOML-based configuration with `serde`
- Environment-aware defaults
- Type-safe configuration parsing

## Troubleshooting

### Common Issues

1. **"Failed to create SSH tunnel"**
   - Verify the host IP address is correct and reachable
   - Ensure SSH is running on the target device
   - Check if you have SSH access to the target device

2. **"Failed to transfer SSH key"**
   - Verify the SSH key file exists at the specified path
   - Ensure the target user account exists
   - Check if the target device allows password authentication

3. **"Connection validation failed"**
   - Network connectivity issues between local and remote host
   - Firewall blocking SSH connections
   - SSH service not running on target

4. **"Tunnel timeout"**
   - Increase `tunnel_timeout_secs` in config file
   - Check network latency and stability

### Debugging

Enable verbose logging for detailed diagnostic information:

```bash
ssh_ip_tunnel --host 192.168.1.42 --user pi --verbose
```

Or set the log level via environment variable:
```bash
RUST_LOG=debug ssh_ip_tunnel --host 192.168.1.42 --user pi
```
