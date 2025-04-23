# Theater MCP Server Implementation Plan

This document outlines the implementation strategy for creating an MCP server that provides access to the Theater WebAssembly actor system.

## 1. Project Setup

### Initial Directory Structure
```
theater-mcp-server/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Entry point with command-line interface
│   ├── lib.rs                  # Library exports
│   ├── server.rs               # Main server implementation
│   ├── theater/
│   │   ├── mod.rs              # Theater client module exports
│   │   ├── client.rs           # Client for Theater server TCP connection
│   │   └── types.rs            # Conversions between Theater and MCP types
│   ├── resources/
│   │   ├── mod.rs              # Resource definitions
│   │   ├── actors.rs           # Actor resource implementation
│   │   └── events.rs           # Event resource implementation
│   └── tools/
│       ├── mod.rs              # Tool definitions
│       ├── actor.rs            # Actor management tools
│       ├── message.rs          # Message tools
│       └── channel.rs          # Channel tools
└── examples/
    └── simple_client.rs        # Example MCP client usage
```

### Cargo.toml Setup
```toml
[package]
name = "theater-mcp-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# MCP dependencies
mcp-protocol = { path = "../rust-mcp/mcp-protocol" }
mcp-server = { path = "../rust-mcp/mcp-server" }

# Common dependencies
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.28", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
async-trait = "0.1"
futures = "0.3"
base64 = "0.21"
uuid = { version = "1.6", features = ["v4", "serde"] }
```

## 2. Core Components Implementation

### 2.1 Theater Client

The Theater client will be responsible for communicating with the Theater server:

```rust
// src/theater/client.rs
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct TheaterClient {
    connection: Arc<Mutex<TcpStream>>,
}

impl TheaterClient {
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self {
            connection: Arc::new(Mutex::new(stream)),
        })
    }
    
    // General method to send commands to the Theater server
    async fn send_command(&self, command: Value) -> Result<Value> {
        let mut connection = self.connection.lock().await;
        
        // Create message frame
        let message = serde_json::to_vec(&command)?;
        let len = message.len() as u32;
        let len_bytes = len.to_be_bytes();
        
        // Send length prefix and message
        connection.write_all(&len_bytes).await?;
        connection.write_all(&message).await?;
        
        // Read response length
        let mut len_buf = [0u8; 4];
        connection.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;
        
        // Read response
        let mut response_buf = vec![0u8; len];
        connection.read_exact(&mut response_buf).await?;
        
        // Parse response
        let response = serde_json::from_slice(&response_buf)?;
        Ok(response)
    }
    
    // Actor management methods
    pub async fn list_actors(&self) -> Result<Vec<String>> {
        let command = json!({
            "method": "GetActors",
            "id": uuid::Uuid::new_v4().to_string(),
        });
        
        let response = self.send_command(command).await?;
        
        // Extract actor IDs from response
        let actors = response
            .get("result")
            .and_then(|r| r.as_object())
            .and_then(|o| o.get("actors"))
            .and_then(|a| a.as_array())
            .ok_or_else(|| anyhow!("Invalid response format"))?
            .iter()
            .filter_map(|a| a.as_str().map(|s| s.to_string()))
            .collect();
            
        Ok(actors)
    }
    
    pub async fn start_actor(&self, manifest: &str, initial_state: Option<Vec<u8>>) -> Result<String> {
        let state_value = if let Some(state) = &initial_state {
            Value::String(base64::encode(state))
        } else {
            Value::Null
        };
        
        let command = json!({
            "method": "SpawnActor",
            "id": uuid::Uuid::new_v4().to_string(),
            "params": {
                "manifest_path": manifest,
                "initial_state": state_value
            }
        });
        
        let response = self.send_command(command).await?;
        
        // Extract actor ID from response
        let actor_id = response
            .get("result")
            .and_then(|r| r.as_object())
            .and_then(|o| o.get("actor_id"))
            .and_then(|id| id.as_str())
            .ok_or_else(|| anyhow!("Invalid response format"))?
            .to_string();
            
        Ok(actor_id)
    }
    
    pub async fn stop_actor(&self, actor_id: &str) -> Result<()> {
        let command = json!({
            "method": "StopActor",
            "id": uuid::Uuid::new_v4().to_string(),
            "params": {
                "actor_id": actor_id
            }
        });
        
        let _response = self.send_command(command).await?;
        Ok(())
    }
    
    pub async fn restart_actor(&self, actor_id: &str) -> Result<()> {
        let command = json!({
            "method": "RestartActor",
            "id": uuid::Uuid::new_v4().to_string(),
            "params": {
                "actor_id": actor_id
            }
        });
        
        let _response = self.send_command(command).await?;
        Ok(())
    }
    
    pub async fn get_actor_state(&self, actor_id: &str) -> Result<Option<Value>> {
        let command = json!({
            "method": "GetActorState",
            "id": uuid::Uuid::new_v4().to_string(),
            "params": {
                "actor_id": actor_id
            }
        });
        
        let response = self.send_command(command).await?;
        
        // Extract state from response
        let state = response
            .get("result")
            .and_then(|r| r.as_object())
            .and_then(|o| o.get("state"));
            
        if let Some(state) = state {
            if state.is_null() {
                return Ok(None);
            } else {
                return Ok(Some(state.clone()));
            }
        }
        
        Ok(None)
    }
    
    pub async fn get_actor_events(&self, actor_id: &str) -> Result<Vec<Value>> {
        let command = json!({
            "method": "GetActorEvents",
            "id": uuid::Uuid::new_v4().to_string(),
            "params": {
                "actor_id": actor_id
            }
        });
        
        let response = self.send_command(command).await?;
        
        // Extract events from response
        let events = response
            .get("result")
            .and_then(|r| r.as_object())
            .and_then(|o| o.get("events"))
            .and_then(|e| e.as_array())
            .ok_or_else(|| anyhow!("Invalid response format"))?
            .clone();
            
        Ok(events)
    }
    
    // Actor message methods
    pub async fn send_message(&self, actor_id: &str, data: &[u8]) -> Result<()> {
        let command = json!({
            "method": "SendMessage",
            "id": uuid::Uuid::new_v4().to_string(),
            "params": {
                "actor_id": actor_id,
                "data": base64::encode(data)
            }
        });
        
        let _response = self.send_command(command).await?;
        Ok(())
    }
    
    pub async fn request_message(&self, actor_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        let command = json!({
            "method": "RequestMessage",
            "id": uuid::Uuid::new_v4().to_string(),
            "params": {
                "actor_id": actor_id,
                "data": base64::encode(data)
            }
        });
        
        let response = self.send_command(command).await?;
        
        // Extract response data
        let response_data = response
            .get("result")
            .and_then(|r| r.as_object())
            .and_then(|o| o.get("data"))
            .and_then(|d| d.as_str())
            .ok_or_else(|| anyhow!("Invalid response format"))?;
            
        let data = base64::decode(response_data)?;
        Ok(data)
    }
    
    // Channel methods
    pub async fn open_channel(&self, actor_id: &str, initial_message: Option<&[u8]>) -> Result<String> {
        let initial_data = if let Some(data) = initial_message {
            base64::encode(data)
        } else {
            "".to_string()
        };
        
        let command = json!({
            "method": "OpenChannel",
            "id": uuid::Uuid::new_v4().to_string(),
            "params": {
                "actor_id": actor_id,
                "initial_message": initial_data
            }
        });
        
        let response = self.send_command(command).await?;
        
        // Extract channel ID
        let channel_id = response
            .get("result")
            .and_then(|r| r.as_object())
            .and_then(|o| o.get("channel_id"))
            .and_then(|id| id.as_str())
            .ok_or_else(|| anyhow!("Invalid response format"))?
            .to_string();
            
        Ok(channel_id)
    }
    
    pub async fn send_on_channel(&self, channel_id: &str, message: &[u8]) -> Result<()> {
        let command = json!({
            "method": "SendOnChannel",
            "id": uuid::Uuid::new_v4().to_string(),
            "params": {
                "channel_id": channel_id,
                "message": base64::encode(message)
            }
        });
        
        let _response = self.send_command(command).await?;
        Ok(())
    }
    
    pub async fn close_channel(&self, channel_id: &str) -> Result<()> {
        let command = json!({
            "method": "CloseChannel",
            "id": uuid::Uuid::new_v4().to_string(),
            "params": {
                "channel_id": channel_id
            }
        });
        
        let _response = self.send_command(command).await?;
        Ok(())
    }
}
```

### 2.2 Resource Implementation

Resources will provide structured access to Theater actors and their state:

```rust
// src/resources/actors.rs
use anyhow::Result;
use mcp_protocol::types::resource::{Resource, ResourceContent};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::theater::client::TheaterClient;

pub struct ActorResources {
    theater_client: Arc<TheaterClient>,
}

impl ActorResources {
    pub fn new(theater_client: Arc<TheaterClient>) -> Self {
        Self { theater_client }
    }
    
    pub async fn list_actors_content(&self) -> Result<ResourceContent> {
        let actor_ids = self.theater_client.list_actors().await?;
        
        let actors = actor_ids.iter().map(|id| {
            json!({
                "id": id,
                "name": format!("Actor {}", id),
                "status": "RUNNING",
                "uri": format!("theater://actor/{}", id)
            })
        }).collect::<Vec<_>>();
        
        let content = json!({
            "actors": actors,
            "total": actors.len()
        });
        
        Ok(ResourceContent::Json { json: content })
    }
    
    pub async fn get_actor_details(&self, actor_id: &str) -> Result<ResourceContent> {
        // We don't have a direct "get actor details" in Theater client
        // so we piece together information from other calls
        
        // Get actor state to verify actor exists
        let state_result = self.theater_client.get_actor_state(actor_id).await;
        if state_result.is_err() {
            return Err(anyhow::anyhow!("Actor not found: {}", actor_id));
        }
        
        let content = json!({
            "id": actor_id,
            "status": "RUNNING", // We're simplifying for now
            "events_uri": format!("theater://events/{}", actor_id),
            "state_uri": format!("theater://actor/{}/state", actor_id)
        });
        
        Ok(ResourceContent::Json { json: content })
    }
    
    pub async fn get_actor_state(&self, actor_id: &str) -> Result<ResourceContent> {
        let state = self.theater_client.get_actor_state(actor_id).await?;
        
        if let Some(state_value) = state {
            return Ok(ResourceContent::Json { json: state_value });
        }
        
        // Return empty JSON if no state
        Ok(ResourceContent::Json { json: json!({}) })
    }
    
    pub async fn get_actor_events(&self, actor_id: &str) -> Result<ResourceContent> {
        let events = self.theater_client.get_actor_events(actor_id).await?;
        Ok(ResourceContent::Json { json: json!(events) })
    }
}
```

### 2.3 Tools Implementation

Tools will allow interaction with the Theater system:

```rust
// src/tools/actor.rs
use anyhow::Result;
use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::theater::client::TheaterClient;

pub struct ActorTools {
    theater_client: Arc<TheaterClient>,
}

impl ActorTools {
    pub fn new(theater_client: Arc<TheaterClient>) -> Self {
        Self { theater_client }
    }
    
    pub async fn start_actor(&self, args: Value) -> Result<ToolCallResult> {
        // Extract manifest path
        let manifest = args["manifest"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing manifest parameter"))?;
            
        // Extract optional initial state
        let initial_state = if let Some(state) = args.get("initial_state") {
            Some(serde_json::to_vec(state)?)
        } else {
            None
        };
        
        // Start the actor
        let actor_id = self.theater_client.start_actor(manifest, initial_state).await?;
        
        // Create result
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Json {
                    json: json!({
                        "actor_id": actor_id,
                        "status": "RUNNING"
                    })
                }
            ],
            is_error: Some(false),
        })
    }
    
    pub async fn stop_actor(&self, args: Value) -> Result<ToolCallResult> {
        // Extract actor ID
        let actor_id = args["actor_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
            
        // Stop the actor
        self.theater_client.stop_actor(actor_id).await?;
        
        // Create result
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Json {
                    json: json!({
                        "actor_id": actor_id,
                        "status": "STOPPED"
                    })
                }
            ],
            is_error: Some(false),
        })
    }
    
    pub async fn restart_actor(&self, args: Value) -> Result<ToolCallResult> {
        // Extract actor ID
        let actor_id = args["actor_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
            
        // Restart the actor
        self.theater_client.restart_actor(actor_id).await?;
        
        // Create result
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Json {
                    json: json!({
                        "actor_id": actor_id,
                        "status": "RUNNING"
                    })
                }
            ],
            is_error: Some(false),
        })
    }
}

// src/tools/message.rs
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::theater::client::TheaterClient;

pub struct MessageTools {
    theater_client: Arc<TheaterClient>,
}

impl MessageTools {
    pub fn new(theater_client: Arc<TheaterClient>) -> Self {
        Self { theater_client }
    }
    
    pub async fn send_message(&self, args: Value) -> Result<ToolCallResult> {
        // Extract actor ID
        let actor_id = args["actor_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
            
        // Extract message data
        let data_b64 = args["data"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing data parameter"))?;
            
        // Decode message data
        let data = BASE64.decode(data_b64)?;
        
        // Send the message
        self.theater_client.send_message(actor_id, &data).await?;
        
        // Create result
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Json {
                    json: json!({
                        "success": true,
                        "actor_id": actor_id
                    })
                }
            ],
            is_error: Some(false),
        })
    }
    
    pub async fn request_message(&self, args: Value) -> Result<ToolCallResult> {
        // Extract actor ID
        let actor_id = args["actor_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
            
        // Extract request data
        let data_b64 = args["data"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing data parameter"))?;
            
        // Decode request data
        let data = BASE64.decode(data_b64)?;
        
        // Send the request and get response
        let response_data = self.theater_client.request_message(actor_id, &data).await?;
        
        // Encode response data
        let response_b64 = BASE64.encode(&response_data);
        
        // Create result
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Json {
                    json: json!({
                        "actor_id": actor_id,
                        "response": response_b64
                    })
                }
            ],
            is_error: Some(false),
        })
    }
}

// src/tools/channel.rs
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::theater::client::TheaterClient;

pub struct ChannelTools {
    theater_client: Arc<TheaterClient>,
}

impl ChannelTools {
    pub fn new(theater_client: Arc<TheaterClient>) -> Self {
        Self { theater_client }
    }
    
    pub async fn open_channel(&self, args: Value) -> Result<ToolCallResult> {
        // Extract actor ID
        let actor_id = args["actor_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
            
        // Extract optional initial message
        let initial_message = if let Some(msg) = args.get("initial_message") {
            if let Some(msg_str) = msg.as_str() {
                let msg_data = BASE64.decode(msg_str)?;
                Some(msg_data)
            } else {
                None
            }
        } else {
            None
        };
        
        // Open the channel
        let channel_id = match initial_message {
            Some(msg) => self.theater_client.open_channel(actor_id, Some(&msg)).await?,
            None => self.theater_client.open_channel(actor_id, None).await?,
        };
        
        // Create result
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Json {
                    json: json!({
                        "channel_id": channel_
