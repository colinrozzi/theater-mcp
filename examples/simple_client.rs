use anyhow::Result;
use mcp_client::{client::ClientBuilder, transport::stdio::StdioTransport};
use mcp_protocol::types::tool::ToolContent;
use serde_json::json;
use std::process::{Command, Stdio};
use tokio::io::{stdin, stdout};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting MCP simple client");
    
    // Start the server process
    let server_process = Command::new("theater-mcp-server")
        .arg("--theater-address")
        .arg("127.0.0.1:9000")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .stdin(Stdio::piped())
        .spawn()?;
    
    println!("Server started with PID: {}", server_process.id());
    
    // Create client transport
    let (transport, receiver) = StdioTransport::from_child_process(server_process);
    
    // Create the client
    let client = ClientBuilder::new("simple-client", "0.1.0")
        .with_transport(transport)
        .build()?;
    
    // Start message handling
    let client_clone = client.clone();
    tokio::spawn(async move {
        while let Ok(message) = receiver.recv().await {
            if let Err(e) = client_clone.handle_message(message).await {
                eprintln!("Error handling message: {}", e);
            }
        }
    });
    
    // Initialize the client
    let init_result = client.initialize().await?;
    println!("Connected to: {} v{}", init_result.server_info.name, init_result.server_info.version);
    
    // List available tools
    let tools = client.list_tools().await?;
    println!("\nAvailable tools:");
    for tool in &tools.tools {
        println!("Tool: {} - {}", tool.name, tool.description.as_deref().unwrap_or(""));
    }
    
    // List available resources
    let resources = client.list_resources().await?;
    println!("\nAvailable resources:");
    for resource in &resources.resources {
        println!("Resource: {}", resource.uri);
    }
    
    // Example: Start an actor
    println!("\nStarting a new actor...");
    let result = client.call_tool("start_actor", &json!({
        "manifest": "/path/to/sample_actor.toml",
        "initial_state": { "counter": 0 }
    })).await?;
    
    // Process the result
    let actor_id = match &result.content[0] {
        ToolContent::Json { json } => {
            json.get("actor_id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string()
        },
        _ => "unknown".to_string(),
    };
    
    println!("Actor started with ID: {}", actor_id);
    
    // Example: Send a message to the actor
    println!("\nSending a message to the actor...");
    let message = "Hello, actor!".as_bytes();
    let base64_message = base64::encode(message);
    
    let result = client.call_tool("send_message", &json!({
        "actor_id": actor_id,
        "data": base64_message
    })).await?;
    
    // Process the result
    match &result.content[0] {
        ToolContent::Json { json } => {
            println!("Message sent: {}", json);
        },
        _ => println!("Unknown result format"),
    }
    
    // Example: Get actor state
    println!("\nGetting actor state...");
    let resource_uri = format!("theater://actor/{}/state", actor_id);
    let resource_result = client.read_resource(&resource_uri).await?;
    
    // Process the result
    match &resource_result.content[0] {
        ToolContent::Json { json } => {
            println!("Actor state: {}", json);
        },
        _ => println!("Unknown result format"),
    }
    
    // Example: Stop the actor
    println!("\nStopping the actor...");
    let result = client.call_tool("stop_actor", &json!({
        "actor_id": actor_id
    })).await?;
    
    // Process the result
    match &result.content[0] {
        ToolContent::Json { json } => {
            println!("Actor stopped: {}", json);
        },
        _ => println!("Unknown result format"),
    }
    
    println!("\nExample complete!");
    Ok(())
}
