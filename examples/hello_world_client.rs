use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use mcp_client::transport::Transport;
use mcp_protocol::JsonRpcMessage;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

// Enhanced client to demonstrate using the theater-mcp-server with hello-world actor
#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Hello World Actor MCP client test...");

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
    sleep(Duration::from_secs(1)).await;

    // Initialize the connection
    println!("Initializing MCP connection...");
    let initialize_msg: JsonRpcMessage = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "1".into(),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "clientInfo": {
                "name": "hello-world-client",
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

    // Send initialized notification
    println!("Sending initialized notification...");
    let initialized_msg: JsonRpcMessage = JsonRpcMessage::Notification {
        jsonrpc: "2.0".to_string(),
        method: "notifications/initialized".to_string(),
        params: None,
    };
    transport.send(initialized_msg).await?;

    // Add a longer delay to ensure the notification is processed
    println!("Waiting for notification to be processed...");
    sleep(Duration::from_secs(2)).await;

    // Step 1: List available tools
    println!("\nListing available tools...");
    let list_tools_msg: JsonRpcMessage = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "2".into(),
        method: "tools/list".to_string(),
        params: None,
    };
    transport.send(list_tools_msg).await?;
    let tools_response = recv.recv().await;
    println!("Tools list response: {:?}", tools_response);

    // Step 2: List available resources
    println!("\nListing available resources...");
    let list_resources_msg: JsonRpcMessage = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "3".into(),
        method: "resources/list".to_string(),
        params: None,
    };
    transport.send(list_resources_msg).await?;
    let resources_response = recv.recv().await;
    println!("Resources list response: {:?}", resources_response);

    // Step 3: Start our Hello World actor - use tools/call as specified in the protocol
    println!("\nStarting Hello World Actor...");
    let manifest_path = "/Users/colinrozzi/work/actors/hello-world-actor/manifest.toml";
    let start_actor_msg: JsonRpcMessage = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "4".into(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "start_actor",
            "arguments": {
                "manifest": manifest_path,
                "initial_state": {"greeting": "Hello from MCP client!"} // Initial state as a simple JSON object
            }
        })),
    };
    transport.send(start_actor_msg).await?;
    println!("\nWaiting for actor to start...");
    let start_response = recv.recv().await;
    println!("Start actor response: {:?}", start_response);

    // Extract actor ID from the start response
    let actor_id = match &start_response {
        Some(JsonRpcMessage::Response { result, .. }) => {
            if let Some(result_obj) = result {
                if let Some(content) = result_obj.get("content") {
                    if let Some(content_arr) = content.as_array() {
                        if !content_arr.is_empty() {
                            // Check for the text field which contains a JSON string
                            if let Some(text_content) = content_arr[0].get("text") {
                                if let Some(text) = text_content.as_str() {
                                    // Parse the JSON string inside the text field
                                    match serde_json::from_str::<serde_json::Value>(text) {
                                        Ok(parsed_json) => {
                                            // Navigate through the nested structure: {"json": {"actor_id": "..."}}
                                            if let Some(json_obj) = parsed_json.get("json") {
                                                if let Some(actor_id) = json_obj.get("actor_id") {
                                                    actor_id.as_str().unwrap_or("").to_string()
                                                } else {
                                                    println!("Failed to find actor_id in json object: {:?}", json_obj);
                                                    "".to_string()
                                                }
                                            } else {
                                                println!("Failed to find json field in parsed text: {:?}", parsed_json);
                                                "".to_string()
                                            }
                                        },
                                        Err(e) => {
                                            println!("Failed to parse text as JSON: {} - Text content: {}", e, text);
                                            "".to_string()
                                        }
                                    }
                                } else {
                                    println!("Text content is not a string");
                                    "".to_string()
                                }
                            } else if let Some(json_content) = content_arr[0].get("json") {
                                // Try the old format just in case
                                if let Some(actor_id) = json_content.get("actor_id") {
                                    actor_id.as_str().unwrap_or("").to_string()
                                } else {
                                    println!("Failed to find actor_id in json content");
                                    "".to_string()
                                }
                            } else {
                                println!("Failed to find text or json in content");
                                "".to_string()
                            }
                        } else {
                            println!("Content array is empty");
                            "".to_string()
                        }
                    } else {
                        println!("Content is not an array");
                        "".to_string()
                    }
                } else {
                    println!("No content field in result");
                    "".to_string()
                }
            } else {
                println!("No result object");
                "".to_string()
            }
        },
        Some(JsonRpcMessage::Response { error, .. }) => {
            println!("Error in start response: {:?}", error);
            "".to_string()
        },
        _ => {
            println!("Failed to extract actor ID from start response: unexpected message type");
            "".to_string()
        },
    };

    if actor_id.is_empty() {
        println!("Failed to extract actor ID from start response");
        return Ok(());
    }
    
    println!("Successfully started actor with ID: {}", actor_id);

    println!("Successfully started actor with ID: {}", actor_id);
    sleep(Duration::from_secs(1)).await;

    // Step 4: Get actor resources/details
    println!("\nGetting actors list...");
    let actors_list_msg: JsonRpcMessage = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "5".into(),
        method: "resources/get".to_string(),
        params: Some(json!({
            "uri": "theater://actors"
        })),
    };
    transport.send(actors_list_msg).await?;
    let actors_list_response = recv.recv().await;
    println!("Actors list response: {:?}", actors_list_response);

    // Step 5: Send a one-way message to the actor
    println!("\nSending message to actor...");
    let message = "Hello from MCP client!";
    let base64_message = BASE64.encode(message.as_bytes());
    let send_message_msg: JsonRpcMessage = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "6".into(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "send_message",
            "arguments": {
                "actor_id": actor_id,
                "data": base64_message
            }
        })),
    };
    transport.send(send_message_msg).await?;
    let send_response = recv.recv().await;
    println!("Send message response: {:?}", send_response);

    // Step 6: Make a request to the actor
    println!("\nMaking request to actor...");
    let request = "What is your state?";
    let base64_request = BASE64.encode(request.as_bytes());
    let request_msg: JsonRpcMessage = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "7".into(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "request_message",
            "arguments": {
                "actor_id": actor_id,
                "data": base64_request
            }
        })),
    };
    transport.send(request_msg).await?;
    let request_response = recv.recv().await;
    println!("Request response: {:?}", request_response);

    // Try to decode the response if we received one
    if let Some(JsonRpcMessage::Response { result, .. }) = &request_response {
        if let Some(result_obj) = result {
            if let Some(content) = result_obj.get("content") {
                if let Some(content_arr) = content.as_array() {
                    if !content_arr.is_empty() {
                        if let Some(json_content) = content_arr[0].get("json") {
                            if let Some(response_b64) =
                                json_content.get("response").and_then(|r| r.as_str())
                            {
                                if let Ok(response_bytes) = BASE64.decode(response_b64) {
                                    if let Ok(response_text) = String::from_utf8(response_bytes) {
                                        println!("Decoded response: {}", response_text);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Step 7: Open a channel to the actor
    println!("\nOpening channel to actor...");
    let channel_message = "Opening a channel";
    let base64_channel_message = BASE64.encode(channel_message.as_bytes());
    let open_channel_msg: JsonRpcMessage = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "8".into(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "open_channel",
            "arguments": {
                "actor_id": actor_id,
                "initial_message": base64_channel_message
            }
        })),
    };
    transport.send(open_channel_msg).await?;
    let channel_response = recv.recv().await;
    println!("Open channel response: {:?}", channel_response);

    // Extract channel ID from the channel response
    let channel_id = match &channel_response {
        Some(JsonRpcMessage::Response { result, .. }) => {
            if let Some(result_obj) = result {
                if let Some(content) = result_obj.get("content") {
                    if let Some(content_arr) = content.as_array() {
                        if !content_arr.is_empty() {
                            if let Some(json_content) = content_arr[0].get("json") {
                                if let Some(channel_id) = json_content.get("channel_id") {
                                    channel_id.as_str().unwrap_or("").to_string()
                                } else {
                                    "".to_string()
                                }
                            } else {
                                "".to_string()
                            }
                        } else {
                            "".to_string()
                        }
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }
        }
        _ => "".to_string(),
    };

    if !channel_id.is_empty() {
        // Step 8: Send message on the channel
        println!("\nSending message on channel...");
        let channel_msg = "Message via channel";
        let base64_channel_msg = BASE64.encode(channel_msg.as_bytes());
        let send_channel_msg: JsonRpcMessage = JsonRpcMessage::Request {
            jsonrpc: "2.0".to_string(),
            id: "9".into(),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "send_on_channel",
                "arguments": {
                    "channel_id": channel_id,
                    "message": base64_channel_msg
                }
            })),
        };
        transport.send(send_channel_msg).await?;
        let send_channel_response = recv.recv().await;
        println!("Send on channel response: {:?}", send_channel_response);

        // Step 9: Close the channel
        println!("\nClosing channel...");
        let close_channel_msg: JsonRpcMessage = JsonRpcMessage::Request {
            jsonrpc: "2.0".to_string(),
            id: "10".into(),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "close_channel",
                "arguments": {
                    "channel_id": channel_id
                }
            })),
        };
        transport.send(close_channel_msg).await?;
        let close_channel_response = recv.recv().await;
        println!("Close channel response: {:?}", close_channel_response);
    }

    // Step 10: Get actor events
    println!("\nGetting actor events...");
    let events_msg: JsonRpcMessage = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "11".into(),
        method: "resources/get".to_string(),
        params: Some(json!({
            "uri": format!("theater://events/{}", actor_id)
        })),
    };
    transport.send(events_msg).await?;
    let events_response = recv.recv().await;
    println!("Actor events response: {:?}", events_response);

    // Step 11: Clean up - stop the actor
    println!("\nStopping actor...");
    let stop_actor_msg: JsonRpcMessage = JsonRpcMessage::Request {
        jsonrpc: "2.0".to_string(),
        id: "12".into(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "stop_actor",
            "arguments": {
                "actor_id": actor_id
            }
        })),
    };
    transport.send(stop_actor_msg).await?;
    let stop_response = recv.recv().await;
    println!("Stop actor response: {:?}", stop_response);

    println!("\nHello World Actor test complete!");
    Ok(())
}
