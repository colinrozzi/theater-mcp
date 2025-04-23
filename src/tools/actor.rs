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
            // Convert to JSON bytes
            let state_bytes = serde_json::to_vec(state)?;
            Some(state_bytes)
        } else {
            None
        };
        
        // Start the actor
        let actor_id = match initial_state {
            Some(ref bytes) => self.theater_client.start_actor(manifest, Some(bytes.as_slice())).await?,
            None => self.theater_client.start_actor(manifest, None).await?,
        };
        
        // Create result
        let response_json = json!({
            "actor_id": actor_id,
            "status": "RUNNING"
        });
        
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Text { 
                    text: serde_json::to_string(&response_json)? 
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
        let response_json = json!({
            "actor_id": actor_id,
            "status": "STOPPED"
        });
        
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Text { 
                    text: serde_json::to_string(&response_json)? 
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
        let response_json = json!({
            "actor_id": actor_id,
            "status": "RUNNING"
        });
        
        Ok(ToolCallResult {
            content: vec![
                ToolContent::Text { 
                    text: serde_json::to_string(&response_json)? 
                }
            ],
            is_error: Some(false),
        })
    }
    
    pub fn register_tools(self: Arc<Self>, tool_manager: &Arc<mcp_server::tools::ToolManager>) {
        use crate::tools::register_async_tool;
        
        // Register the start_actor tool
        let start_actor_tool = mcp_protocol::types::tool::Tool {
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
        
        let actor_self = self.clone();
        register_async_tool(tool_manager, start_actor_tool, move |args| {
            let actor_self = actor_self.clone();
            async move {
                actor_self.start_actor(args).await
            }
        });
        
        // Register the stop_actor tool
        let stop_actor_tool = mcp_protocol::types::tool::Tool {
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
        
        let actor_self = self.clone();
        register_async_tool(tool_manager, stop_actor_tool, move |args| {
            let actor_self = actor_self.clone();
            async move {
                actor_self.stop_actor(args).await
            }
        });
        
        // Register the restart_actor tool
        let restart_actor_tool = mcp_protocol::types::tool::Tool {
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
        
        let actor_self = self.clone();
        register_async_tool(tool_manager, restart_actor_tool, move |args| {
            let actor_self = actor_self.clone();
            async move {
                actor_self.restart_actor(args).await
            }
        });
    }
}
