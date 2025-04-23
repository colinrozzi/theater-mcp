use anyhow::{anyhow, Result};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::time::Duration;
use std::thread;
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Helper script for testing Theater MCP resources
#[tokio::main]
async fn main() -> Result<()> {
    println!("Theater MCP Resources Test Helper");
    println!("=================================");

    // Step 1: Check if Theater server is running
    println!("\nStep 1: Checking if Theater server is running...");
    
    let theater_running = match check_theater_server().await {
        Ok(true) => {
            println!("✅ Theater server is running on localhost:9000");
            true
        },
        Ok(false) => {
            println!("❌ Theater server is not running on localhost:9000");
            println!("   Please start the Theater server and try again.");
            println!("   Typically: cargo run --bin theater-server -- --port 9000");
            false
        },
        Err(e) => {
            println!("❌ Error checking Theater server: {}", e);
            false
        }
    };

    if !theater_running {
        return Err(anyhow!("Theater server must be running to test resources"));
    }

    // Step 2: Check if we have an actor running or start one
    println!("\nStep 2: Checking for running actors...");
    
    // Build our mcp-server if it's not already built
    println!("Building Theater MCP server...");
    let build_output = Command::new("cargo")
        .args(["build", "--bin", "theater-mcp-server"])
        .current_dir("/Users/colinrozzi/work/mcp-servers/theater-mcp-server")
        .output()?;
    
    if !build_output.status.success() {
        return Err(anyhow!("Failed to