use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use mcp_protocol::types::tool::{Tool, ToolCallResult, ToolContent};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::debug;

use crate::theater::client::TheaterClient;

/// Tools for sending messages to Theater actors
pub struct MessageTools {
    theater_client: Arc<TheaterClient>,
}

impl MessageTools {
    /// Create a new message tools instance
    pub fn new(theater_client: Arc<TheaterClient>) -> Self {
        Self { theater_client }
    }
    
    /// Send a one-way message to an actor
    pub async fn send_message(&self, args: Value) -> Result<ToolCallResult> {
        debug!("Sending message with args: {:?}", args);
        
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
        let result_json = json!({
            "success": true,
            "actor_id": actor_id
        });
        
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Text { 
                    text: result_json.to_string()
                }
            ],
            is_error: Some(false),
        })
    }
    
    /// Send a request to an actor and receive a response
    pub async fn request_message(&self, args: Value) -> Result<ToolCallResult> {
        debug!("Sending request message with args: {:?}", args);
        
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
        let result_json = json!({
            "actor_id": actor_id,
            "response": response_b64
        });
        
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Text { 
                    text: result_json.to_string()
                }
            ],
            is_error: Some(false),
        })
    }
    
    /// Register message tools with the MCP tool manager
    pub fn register_tools(
        self: Arc<Self>,
        tool_manager: &Arc<mcp_server::tools::ToolManager>,
    ) {
        // Register send_message tool
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
        tool_manager.register_tool(send_message_tool, move |args| {
            let tools_self = tools_self.clone();
            let fut = tools_self.send_message(args);
            
            // Use the current runtime handle instead of creating a new one
            let handle = tokio::runtime::Handle::current();
            handle.block_on(fut)
        });
        
        // Register request_message tool
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
        tool_manager.register_tool(request_message_tool, move |args| {
            let tools_self = tools_self.clone();
            let fut = tools_self.request_message(args);
            
            // Convert async result to sync
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(fut)
        });
    }
}
