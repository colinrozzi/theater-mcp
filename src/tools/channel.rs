use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::warn;

use crate::theater::client::TheaterClient;

pub struct ChannelTools {
    theater_client: Arc<TheaterClient>,
}
impl ChannelTools {
    pub fn new(theater_client: Arc<TheaterClient>) -> Self {
        Self { theater_client }
    }
    
    /// Helper method to handle Theater connection errors
    fn handle_connection_error<T>(&self, result: Result<T>, context: &str) -> Result<T> {
        match result {
            Ok(val) => Ok(val),
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("connect") || error_msg.contains("connection") || 
                   error_msg.contains("read") || error_msg.contains("write") {
                    // This is likely a connection issue
                    warn!("Theater connection issue during {}: {}. Will attempt reconnection on next request.", context, error_msg);
                    Err(anyhow::anyhow!("Theater server connection issue: {}. The server will attempt to reconnect on the next request.", error_msg))
                } else {
                    // Other type of error
                    Err(e)
                }
            }
        }
    }
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
        
        // Open the channel with connection error handling
        let channel_id = match initial_message {
            Some(msg) => self.handle_connection_error(
                self.theater_client.open_channel(actor_id, Some(&msg)).await,
                &format!("channel open to {}", actor_id)
            )?,
            None => self.handle_connection_error(
                self.theater_client.open_channel(actor_id, None).await,
                &format!("channel open to {}", actor_id)
            )?,
        };
        
        // Create result
        let response_json = json!({
            "channel_id": channel_id,
            "actor_id": actor_id
        });
        
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Text { 
                    text: format!("{{\"json\":{}}}", serde_json::to_string(&response_json)?) 
                }
            ],
            is_error: Some(false),
        })
    }
    
    pub async fn send_on_channel(&self, args: Value) -> Result<ToolCallResult> {
        // Extract channel ID
        let channel_id = args["channel_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing channel_id parameter"))?;
            
        // Extract message data
        let message_b64 = args["message"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing message parameter"))?;
            
        // Decode message data
        let message = BASE64.decode(message_b64)?;
        
        // Send on the channel with connection error handling
        self.handle_connection_error(
            self.theater_client.send_on_channel(channel_id, &message).await,
            &format!("channel send on {}", channel_id)
        )?;
        
        // Create result
        let response_json = json!({
            "success": true,
            "channel_id": channel_id
        });
        
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Text { 
                    text: format!("{{\"json\":{}}}", serde_json::to_string(&response_json)?) 
                }
            ],
            is_error: Some(false),
        })
    }
    
    pub async fn close_channel(&self, args: Value) -> Result<ToolCallResult> {
        // Extract channel ID
        let channel_id = args["channel_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing channel_id parameter"))?;
            
        // Close the channel with connection error handling
        self.handle_connection_error(
            self.theater_client.close_channel(channel_id).await,
            &format!("channel close {}", channel_id)
        )?;
        
        // Create result
        let response_json = json!({
            "success": true,
            "channel_id": channel_id
        });
        
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Text { 
                    text: format!("{{\"json\":{}}}", serde_json::to_string(&response_json)?) 
                }
            ],
            is_error: Some(false),
        })
    }
    
    pub fn register_tools(self: Arc<Self>, tool_manager: &Arc<mcp_server::tools::ToolManager>) {
        use crate::tools::utils::register_async_tool;
        
        // Register the open_channel tool
        let open_channel_tool = mcp_protocol::types::tool::Tool {
            name: "open_channel".to_string(),
            description: Some("Open a communication channel to an actor".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "actor_id": {
                        "type": "string",
                        "description": "ID of the actor to open a channel with"
                    },
                    "initial_message": {
                        "type": "string",
                        "description": "Initial message data (base64 encoded)"
                    }
                },
                "required": ["actor_id"]
            }),
            annotations: None,
        };
        
        let channel_self = self.clone();
        register_async_tool(tool_manager, open_channel_tool, move |args| {
            let channel_self = channel_self.clone();
            async move {
                channel_self.open_channel(args).await
            }
        });
        
        // Register the send_on_channel tool
        let send_on_channel_tool = mcp_protocol::types::tool::Tool {
            name: "send_on_channel".to_string(),
            description: Some("Send a message on an open channel".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "string",
                        "description": "ID of the channel"
                    },
                    "message": {
                        "type": "string",
                        "description": "Message data (base64 encoded)"
                    }
                },
                "required": ["channel_id", "message"]
            }),
            annotations: None,
        };
        
        let channel_self = self.clone();
        register_async_tool(tool_manager, send_on_channel_tool, move |args| {
            let channel_self = channel_self.clone();
            async move {
                channel_self.send_on_channel(args).await
            }
        });
        
        // Register the close_channel tool
        let close_channel_tool = mcp_protocol::types::tool::Tool {
            name: "close_channel".to_string(),
            description: Some("Close an open channel".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "string",
                        "description": "ID of the channel to close"
                    }
                },
                "required": ["channel_id"]
            }),
            annotations: None,
        };
        
        let channel_self = self.clone();
        register_async_tool(tool_manager, close_channel_tool, move |args| {
            let channel_self = channel_self.clone();
            async move {
                channel_self.close_channel(args).await
            }
        });
    }
}