use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use mcp_client::transport::Transport;
use mcp_protocol::messages::ClientCapabilities;
use mcp_protocol::JsonRpcMessage;
use serde_json::json;
use tracing::{error, info, trace};

// Simple client to demonstrate using the theater-mcp-server
#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Theater MCP client test...");

    // Create a transport connection to the server
    let (transport, mut recv) = mcp_client::transport::stdio::StdioTransport::new(
        "/Users/colinrozzi/work/mcp-servers/theater-mcp-server/target/debug/theater-mcp-server",
        vec![
            "--theater-address".to_string(),
            "127.0.0.1:9000".to_string(),
        ],
    );

    transport.start().await?;

    // Wait a moment for the server to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Initialize the connection
    println!("Initializing MCP connection...");
    let initialize_msg: JsonRpcMessage = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "1".into(),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "clientInfo": {
                "name": "test-client",
                "version": "0.1.0",
            },
            "capabilities": {
                "tools": {
                    "enabled": true,
                },
                "resources": {
                    "enabled": true,
                },
            },
        })),
    };
    transport.send(initialize_msg).await?;

    // Read the response
    let response = recv.recv().await;
    println!("Initialize response: {:?}", response);

    // List tools
    println!("\nListing tools...");
    let list_tools_msg: JsonRpcMessage = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "2".into(),
        method: "tools/list".to_string(),
        params: None,
    };
    transport.send(list_tools_msg).await?;

    // Read the response
    let response = recv.recv().await;
    println!("Tools response: {:?}", response);

    // List resources
    println!("\nListing resources...");
    let list_resources_msg: JsonRpcMessage = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "3".into(),
        method: "resources/list".to_string(),
        params: None,
    };
    transport.send(list_resources_msg).await?;

    // Read the response
    let response = recv.recv().await;
    println!("Resources response: {:?}", response);

    // Try to use a tool (if a Theater server is running with an actor)
    if let Ok(actor_id) = std::env::var("THEATER_ACTOR_ID") {
        println!("\nSending message to actor: {}", actor_id);

        // Create a simple message
        let message = "Hello from MCP client!";
        let base64_message = BASE64.encode(message.as_bytes());

        // Send the message
        let send_message_msg: JsonRpcMessage = JsonRpcMessage::Request {
            jsonrpc: "2.0".to_string(),
            id: "4".into(),
            method: "callTool".to_string(),
            params: Some(json!({
                "name": "send_message",
                "arguments": {
                    "actor_id": actor_id,
                    "data": base64_message,
                },
            })),
        };
        transport.send(send_message_msg).await?;

        // Read the response
        let response = recv.recv().await;
        println!("Send message response: {:?}", response);
    } else {
        println!("\nSkipping actor message test (set THEATER_ACTOR_ID env var to test)");
    }

    println!("\nTest complete!");
    Ok(())
}
