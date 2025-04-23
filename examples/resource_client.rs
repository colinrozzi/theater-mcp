use anyhow::Result;
use mcp_client::transport::Transport;
use mcp_protocol::JsonRpcMessage;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;

/// Test client for Theater MCP resources
#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Theater MCP resource test client...");

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

    // Initialize the MCP connection
    println!("Initializing MCP connection...");
    let initialize_msg = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "1".into(),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "clientInfo": {
                "name": "resource-test-client",
                "version": "0.1.0",
            },
            "capabilities": {
                "resources": {
                    "enabled": true,
                },
            },
        })),
    };
    transport.send(initialize_msg).await?;

    // Read the initialize response
    let response = recv.recv().await;
    println!("Initialize response: {:?}", response);

    // Send initialized notification
    println!("Sending initialized notification...");
    let initialized_msg = JsonRpcMessage::Notification {
        jsonrpc: "2.0".to_string(),
        method: "notifications/initialized".to_string(),
        params: None,
    };
    transport.send(initialized_msg).await?;
    
    // Add a delay to ensure the notification is processed
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // List available resources
    println!("\nListing resources...");
    let list_resources_msg = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "2".into(),
        method: "resources/list".to_string(),
        params: None,
    };
    transport.send(list_resources_msg).await?;

    // Read the resources list response
    let response = recv.recv().await;
    println!("Resources list response: {:?}", response);

    // Test 1: Fetch the actors list resource
    println!("\nFetching actors list resource...");
    let get_actors_msg = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "3".into(),
        method: "resources/get".to_string(),
        params: Some(json!({
            "uri": "theater://actors"
        })),
    };
    transport.send(get_actors_msg).await?;

    // Read the actors resource response
    let response = recv.recv().await;
    println!("Actors resource response: {:?}", response);
    
    // Extract actor IDs from the response if available
    let mut actor_id = None;
    if let Some(JsonRpcMessage::Response { result, .. }) = response {
        if let Some(result_value) = result {
            if let Some(content) = result_value.get("content") {
                if let Some(content_array) = content.as_array() {
                    if let Some(first_content) = content_array.first() {
                        if let Some(text) = first_content.get("text") {
                            if let Ok(json_value) = serde_json::from_str::<Value>(text.as_str().unwrap_or("{}")) {
                                if let Some(actors) = json_value.get("actors") {
                                    if let Some(actors_array) = actors.as_array() {
                                        if let Some(first_actor) = actors_array.first() {
                                            actor_id = first_actor.get("id").and_then(|id| id.as_str()).map(String::from);
                                            println!("Found actor ID: {}", actor_id.as_ref().unwrap());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // If we found an actor, test the actor-specific resources
    if let Some(actor_id) = actor_id {
        // Test 2: Fetch actor details
        println!("\nFetching actor details...");
        let get_actor_details_msg = JsonRpcMessage::Request {
            jsonrpc: "2.0".to_string(),
            id: "4".into(),
            method: "resources/get".to_string(),
            params: Some(json!({
                "uri": format!("theater://actor/{}", actor_id),
            })),
        };
        transport.send(get_actor_details_msg).await?;

        // Read the actor details response
        let response = recv.recv().await;
        println!("Actor details response: {:?}", response);

        // Test 3: Fetch actor state
        println!("\nFetching actor state...");
        let get_actor_state_msg = JsonRpcMessage::Request {
            jsonrpc: "2.0".to_string(),
            id: "5".into(),
            method: "resources/get".to_string(),
            params: Some(json!({
                "uri": format!("theater://actor/{}/state", actor_id),
            })),
        };
        transport.send(get_actor_state_msg).await?;

        // Read the actor state response
        let response = recv.recv().await;
        println!("Actor state response: {:?}", response);

        // Test 4: Fetch actor events
        println!("\nFetching actor events...");
        let get_actor_events_msg = JsonRpcMessage::Request {
            jsonrpc: "2.0".to_string(),
            id: "6".into(),
            method: "resources/get".to_string(),
            params: Some(json!({
                "uri": format!("theater://events/{}", actor_id),
            })),
        };
        transport.send(get_actor_events_msg).await?;

        // Read the actor events response
        let response = recv.recv().await;
        println!("Actor events response: {:?}", response);
    } else {
        println!("No actors found to test actor-specific resources.");
    }

    // Test 5: Test template expansion
    println!("\nTesting template expansion...");
    let expand_template_msg = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "7".into(),
        method: "resources/expandTemplate".to_string(),
        params: Some(json!({
            "template": "theater://actor/{actor_id}",
            "parameters": {
                "actor_id": "test-actor-123",
            },
        })),
    };
    transport.send(expand_template_msg).await?;

    // Read the template expansion response
    let response = recv.recv().await;
    println!("Template expansion response: {:?}", response);

    println!("\nResource tests complete!");
    Ok(())
}
