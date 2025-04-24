use anyhow::Result;
use mcp_protocol::types::tool::{Tool, ToolCallResult, ToolContent};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::error;

use crate::theater::client::TheaterClient;
use crate::tools::utils::register_async_tool;

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
            // Convert to JSON bytes
            let state_bytes = serde_json::to_vec(state)?;
            Some(state_bytes)
        } else {
            None
        };
        
        // Start the actor and capture any errors for better debugging
        let actor_id = match initial_state {
            Some(ref bytes) => match self.theater_client.start_actor(manifest, Some(bytes.as_slice())).await {
                Ok(id) => id,
                Err(e) => {
                    // Log the error for debugging
                    error!("Error starting actor: {}", e);
                    return Err(anyhow::anyhow!("Failed to start actor: {}", e));
                }
            },
            None => match self.theater_client.start_actor(manifest, None).await {
                Ok(id) => id,
                Err(e) => {
                    // Log the error for debugging
                    error!("Error starting actor: {}", e);
                    return Err(anyhow::anyhow!("Failed to start actor: {}", e));
                }
            },
        };
        
        // Create result
        let result_json = json!({
            "actor_id": actor_id,
            "status": "RUNNING"
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
    
    pub async fn stop_actor(&self, args: Value) -> Result<ToolCallResult> {
        // Extract actor ID
        let actor_id_str = args["actor_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
            
        // Stop the actor
        self.theater_client.stop_actor(actor_id_str).await?;
        
        // Create result
        let result_json = json!({
            "actor_id": actor_id_str,
            "status": "STOPPED"
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
    
    pub async fn restart_actor(&self, args: Value) -> Result<ToolCallResult> {
        // Extract actor ID
        let actor_id_str = args["actor_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
            
        // Restart the actor
        self.theater_client.restart_actor(actor_id_str).await?;
        
        // Create result
        let result_json = json!({
            "actor_id": actor_id_str,
            "status": "RUNNING"
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
        // Register the start_actor tool
        let start_actor_tool = Tool {
            name: "start_actor".to_string(),
            description: Some("Start a new actor from a manifest".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "manifest": {
                        "type": "string",
                        "description": "Path to the actor manifest or manifest content"
                    },
                    "initial_state": {
                        "type": "object",
                        "description": "Optional initial state for the actor"
                    }
                },
                "required": ["manifest"]
            }),
            annotations: None,
        };
        
        let tools_self = self.clone();
        register_async_tool(
            tool_manager,
            start_actor_tool,
            move |args| {
                let tools_self = tools_self.clone();
                async move {
                    tools_self.start_actor(args).await
                }
            },
        );
        
        // Register the stop_actor tool
        let stop_actor_tool = Tool {
            name: "stop_actor".to_string(),
            description: Some("Stop a running actor".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "actor_id": {
                        "type": "string",
                        "description": "ID of the actor to stop"
                    }
                },
                "required": ["actor_id"]
            }),
            annotations: None,
        };
        
        let tools_self = self.clone();
        register_async_tool(
            tool_manager,
            stop_actor_tool,
            move |args| {
                let tools_self = tools_self.clone();
                async move {
                    tools_self.stop_actor(args).await
                }
            },
        );
        
        // Register the restart_actor tool
        let restart_actor_tool = Tool {
            name: "restart_actor".to_string(),
            description: Some("Restart a running actor".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "actor_id": {
                        "type": "string",
                        "description": "ID of the actor to restart"
                    }
                },
                "required": ["actor_id"]
            }),
            annotations: None,
        };
        
        let tools_self = self.clone();
        register_async_tool(
            tool_manager,
            restart_actor_tool,
            move |args| {
                let tools_self = tools_self.clone();
                async move {
                    tools_self.restart_actor(args).await
                }
            },
        );
    }
}