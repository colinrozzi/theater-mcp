use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use serde_json::{json, Value};
use std::sync::Arc;

use theater::id::TheaterId;
use theater::messages::ChannelParticipant;
use crate::theater::client::TheaterClient;
use crate::theater::types::TheaterIdExt;

pub struct ChannelTools {
    theater_client: Arc<TheaterClient>,
}

impl ChannelTools {
    pub fn new(theater_client: Arc<TheaterClient>) -> Self {
        Self { theater_client }
    }
    
    pub async fn open_channel(&self, args: Value) -> Result<ToolCallResult> {
        // Extract actor ID
        let actor_id_str = args["actor_id"].as_str()
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
            Some(msg) => self.theater_client.open_channel(actor_id_str, Some(&msg)).await?,
            None => self.theater_client.open_channel(actor_id_str, None).await?,
        };
        
        // Create result
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Json {
                    json: json!({
                        "channel_id": channel_id,
                        "actor_id": actor_id_str
                    })
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
        let message_data = BASE64.decode(message_b64)?;
        
        // Send on the channel
        self.theater_client.send_on_channel(channel_id, &message_data).await?;
        
        // Create result
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Json {
                    json: json!({
                        "success": true,
                        "channel_id": channel_id
                    })
                }
            ],
            is_error: Some(false),
        })
    }
    
    pub async fn close_channel(&self, args: Value) -> Result<ToolCallResult> {
        // Extract channel ID
        let channel_id = args["channel_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing channel_id parameter"))?;
            
        // Close the channel
        self.theater_client.close_channel(channel_id).await?;
        
        // Create result
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Json {
                    json: json!({
                        "success": true,
                        "channel_id": channel_id
                    })
                }
            ],
            is_error: Some(false),
        })
    }
    
    /// Register the tools with the MCP tool manager
    pub fn register_tools(
        self: Arc<Self>,
        tool_manager: &Arc<mcp_server::tools::ToolManager>,
    ) {
        // Register the open_channel tool
        tool_manager.register_tool(
            "open_channel",
            "Open a communication channel to an actor",
            json!({
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
            move |args| {
                let tools_self = self.clone();
                Box::pin(async move {
                    tools_self.open_channel(args).await
                })
            },
        );
        
        // Register the send_on_channel tool
        tool_manager.register_tool(
            "send_on_channel",
            "Send a message on an open channel",
            json!({
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
            move |args| {
                let tools_self = self.clone();
                Box::pin(async move {
                    tools_self.send_on_channel(args).await
                })
            },
        );
        
        // Register the close_channel tool
        tool_manager.register_tool(
            "close_channel",
            "Close an open channel",
            json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "string",
                        "description": "ID of the channel to close"
                    }
                },
                "required": ["channel_id"]
            }),
            move |args| {
                let tools_self = self.clone();
                Box::pin(async move {
                    tools_self.close_channel(args).await
                })
            },
        );
    }
}