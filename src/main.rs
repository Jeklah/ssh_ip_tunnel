// A CLI tool to create an IP tunnel to an ARM CPU and transfer SSH keys.
// Author: Arthur Bowers
// Usage: ssh-ip-tunnel --host <ARM_IP> --user <USERNAME> [--key <KEY_PATH>] [--port <LOCAL_PORT>]
// Example: ssh-ip-tunnel --host "192.168.1.42" --user "pi" --key "~/.ssh/id_rsa.pub" --port 2222

use anyhow::Result;
use clap::Parser;
use std::process::Command;

/// A simple CLI tool to create an IP tunnel to an ARM CPU and transfer SSH keys.
#[derive(Parser, Debug)]
#[command(name = "ssh-ip-tunnel")]
#[command(about = "CLI tool for tunneling SSH and SSH key transfer", long_about = None)]
struct Cli {
    // The IP address of the ARM CPU
    #[arg(short, long)]
    host: String,

    // The username for SSH
    user: String,

    // Path to the SSH key file to transfer
    #[arg(short, long, default_value = "~/.ssh/id_rsa.pub")]
    key: String,

    // Optional: local port to bind for tunnel
    #[arg(shoft, long, default_value_t = 2222)]
    port: u16,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!(
        "Creating SSH tunnel to {}@{} on port {}...",
        cli.user, cli.host, cli.port
    );

    // Create the SSH tunnel
    // This will forward local port to the ARM CPU's SSH port
    let tunnel = Command::new("ssh")
        .args([
            "-fN", // Run in background
            "-L",
            &format!("{}:localhost:22", cli.port),
            &format!("{}@{}", cli.user, cli.host),
        ])
        .status()?;

    if !tunnel.success() {
        eprintln!("Failed to create SSH tunnel");
        return Ok(());
    }

    println!("Tunnel established on localhost:{}", cli.port);

    // Transfer the SSH key
    let copy_key = Command::new("ssh-copy-id")
        .args([
            "-i",
            &cli.key,
            &format!("-p{}", cli.port),
            &format!("{}@localhost", cli.user),
        ])
        .status()?;

    if copy_key.success() {
        println!("SSH key transferred successfully!");
    } else {
        eprintln!("Failed to transfer SSH key");
    }

    Ok(())
}

