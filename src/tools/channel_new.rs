use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use mcp_protocol::types::tool::{Tool, ToolCallResult, ToolContent};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::theater::client::TheaterClient;
use crate::tools::utils::register_async_tool;

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
        let result_json = json!({
            "channel_id": channel_id,
            "actor_id": actor_id
        });
        
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Text { 
                    text: serde_json::to_string(&result_json)? 
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
        let result_json = json!({
            "success": true,
            "channel_id": channel_id
        });
        
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Text { 
                    text: serde_json::to_string(&result_json)? 
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
        let result_json = json!({
            "success": true,
            "channel_id": channel_id
        });
        
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Text { 
                    text: serde_json::to_string(&result_json)? 
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
        let open_channel_tool = Tool {
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
        
        let tools_self = self.clone();
        register_async_tool(
            tool_manager,
            open_channel_tool,
            move |args| {
                let tools_self = tools_self.clone();
                async move {
                    tools_self.open_channel(args).await
                }
            },
        );
        
        // Register the send_on_channel tool
        let send_on_channel_tool = Tool {
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
        
        let tools_self = self.clone();
        register_async_tool(
            tool_manager,
            send_on_channel_tool,
            move |args| {
                let tools_self = tools_self.clone();
                async move {
                    tools_self.send_on_channel(args).await
                }
            },
        );
        
        // Register the close_channel tool
        let close_channel_tool = Tool {
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
        
        let tools_self = self.clone();
        register_async_tool(
            tool_manager,
            close_channel_tool,
            move |args| {
                let tools_self = tools_self.clone();
                async move {
                    tools_self.close_channel(args).await
                }
            },
        );
    }
}