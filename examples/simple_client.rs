use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use mcp_protocol::types::tool::ToolContent;
use serde_json::json;
use std::process::{Command, Stdio};
use tokio::io::{stdin, stdout};

// Simple client to demonstrate using the theater-mcp-server
#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Theater MCP client test...");
    
    // Start the server process
    let server_process = Command::new("../target/debug/theater-mcp-server")
        .arg("--theater-address")
        .arg("127.0.0.1:9000")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .stdin(Stdio::piped())
        .spawn()?;
    
    println!("Server started with PID: {}", server_process.id());
    
    // Wait a moment for the server to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Create a transport connection to the server
    let mut transport = mcp_client::transport::stdio::StdioProcess::new(server_process);
    
    // Initialize the connection
    println!("Initializing MCP connection...");
    let initialize_msg = r#"{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocolVersion":"2024-11-05","clientInfo":{"name":"test-client","version":"0.1.0"}}}"#;
    transport.write(initialize_msg.as_bytes()).await?;
    
    // Read the response
    let mut buf = [0u8; 4096];
    let n = transport.read(&mut buf).await?;
    let response = std::str::from_utf8(&buf[..n])?;
    println!("Initialize response: {}", response);
    
    // List tools
    println!("\nListing tools...");
    let list_tools_msg = r#"{"jsonrpc":"2.0","id":"2","method":"listTools","params":{}}"#;
    transport.write(list_tools_msg.as_bytes()).await?;
    
    // Read the response
    let n = transport.read(&mut buf).await?;
    let response = std::str::from_utf8(&buf[..n])?;
    println!("Tools response: {}", response);
    
    // List resources
    println!("\nListing resources...");
    let list_resources_msg = r#"{"jsonrpc":"2.0","id":"3","method":"listResources","params":{}}"#;
    transport.write(list_resources_msg.as_bytes()).await?;
    
    // Read the response
    let n = transport.read(&mut buf).await?;
    let response = std::str::from_utf8(&buf[..n])?;
    println!("Resources response: {}", response);
    
    // Try to use a tool (if a Theater server is running with an actor)
    if let Ok(actor_id) = std::env::var("THEATER_ACTOR_ID") {
        println!("\nSending message to actor: {}", actor_id);
        
        // Create a simple message
        let message = "Hello from MCP client!";
        let base64_message = BASE64.encode(message.as_bytes());
        
        // Send the message
        let send_message_msg = format!(
            r#"{{"jsonrpc":"2.0","id":"4","method":"callTool","params":{{"name":"send_message","arguments":{{"actor_id":"{}","data":"{}"}}}}}}"#,
            actor_id, base64_message
        );
        transport.write(send_message_msg.as_bytes()).await?;
        
        // Read the response
        let n = transport.read(&mut buf).await?;
        let response = std::str::from_utf8(&buf[..n])?;
        println!("Send message response: {}", response);
    } else {
        println!("\nSkipping actor message test (set THEATER_ACTOR_ID env var to test)");
    }
    
    println!("\nTest complete!");
    Ok(())
}
