use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use mcp_protocol::types::tool::{Tool, ToolCallResult, ToolContent};
use serde_json::{json, Value};
use std::sync::Arc;

use theater::id::TheaterId;
use crate::theater::client::TheaterClient;
use crate::theater::TheaterIdExt;
use crate::tools::utils::register_async_tool;

pub struct MessageTools {
    theater_client: Arc<TheaterClient>,
}

impl MessageTools {
    pub fn new(theater_client: Arc<TheaterClient>) -> Self {
        Self { theater_client }
    }
    
    pub async fn send_message(&self, args: Value) -> Result<ToolCallResult> {
        // Extract actor ID
        let actor_id_str = args["actor_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
            
        // Convert to TheaterId
        let theater_id = TheaterId::from_str(actor_id_str)?;
            
        // Extract message data
        let data_b64 = args["data"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing data parameter"))?;
            
        // Decode message data
        let data = BASE64.decode(data_b64)?;
        
        // Send the message
        self.theater_client.send_message(&theater_id, &data).await?;
        
        // Create result
        let result_json = json!({
            "success": true,
            "actor_id": actor_id_str
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
    
    pub async fn request_message(&self, args: Value) -> Result<ToolCallResult> {
        // Extract actor ID
        let actor_id_str = args["actor_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
            
        // Convert to TheaterId
        let theater_id = TheaterId::from_str(actor_id_str)?;
            
        // Extract request data
        let data_b64 = args["data"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing data parameter"))?;
            
        // Decode request data
        let data = BASE64.decode(data_b64)?;
        
        // Send the request and get response
        let response_data = self.theater_client.request_message(&theater_id, &data).await?;
        
        // Encode response data
        let response_b64 = BASE64.encode(&response_data);
        
        // Create result
        let result_json = json!({
            "actor_id": actor_id_str,
            "response": response_b64
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
        // Register the send_message tool
        let send_message_tool = Tool {
            name: "send_message".to_string(),
            description: Some("Send a message to an actor".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "actor_id": {
                        "type": "string",
                        "description": "ID of the actor to send the message to"
                    },
                    "data": {
                        "type": "string",
                        "description": "Message data (base64 encoded)"
                    }
                },
                "required": ["actor_id", "data"]
            }),
            annotations: None,
        };
        
        let tools_self = self.clone();
        register_async_tool(
            tool_manager,
            send_message_tool,
            move |args| {
                let tools_self = tools_self.clone();
                async move {
                    tools_self.send_message(args).await
                }
            },
        );
        
        // Register the request_message tool
        let request_message_tool = Tool {
            name: "request_message".to_string(),
            description: Some("Send a request to an actor and receive a response".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "actor_id": {
                        "type": "string",
                        "description": "ID of the actor to send the request to"
                    },
                    "data": {
                        "type": "string",
                        "description": "Request data (base64 encoded)"
                    }
                },
                "required": ["actor_id", "data"]
            }),
            annotations: None,
        };
        
        let tools_self = self.clone();
        register_async_tool(
            tool_manager,
            request_message_tool,
            move |args| {
                let tools_self = tools_self.clone();
                async move {
                    tools_self.request_message(args).await
                }
            },
        );
    }
}