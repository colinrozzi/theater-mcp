use anyhow::Result;
use mcp_protocol::types::tool::{Tool, ToolCallResult, ToolContent};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::debug;

use crate::theater::client::TheaterClient;

/// Tools for managing Theater actors
pub struct ActorTools {
    theater_client: Arc<TheaterClient>,
}

impl ActorTools {
    /// Create a new actor tools instance
    pub fn new(theater_client: Arc<TheaterClient>) -> Self {
        Self { theater_client }
    }
    
    /// Start a new actor from a manifest
    pub async fn start_actor(&self, args: Value) -> Result<ToolCallResult> {
        debug!("Starting actor with args: {:?}", args);
        
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
        let initial_state_bytes = initial_state.as_deref();
        let actor_id = self.theater_client.start_actor(manifest, initial_state_bytes).await?;
        
        // Create result
        let result_json = json!({
            "actor_id": actor_id,
            "status": "RUNNING"
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
    
    /// Stop a running actor
    pub async fn stop_actor(&self, args: Value) -> Result<ToolCallResult> {
        debug!("Stopping actor with args: {:?}", args);
        
        // Extract actor ID
        let actor_id = args["actor_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
            
        // Stop the actor
        self.theater_client.stop_actor(actor_id).await?;
        
        // Create result
        let result_json = json!({
            "actor_id": actor_id,
            "status": "STOPPED"
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
    
    /// Restart a running actor
    pub async fn restart_actor(&self, args: Value) -> Result<ToolCallResult> {
        debug!("Restarting actor with args: {:?}", args);
        
        // Extract actor ID
        let actor_id = args["actor_id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
            
        // Restart the actor
        self.theater_client.restart_actor(actor_id).await?;
        
        // Create result
        let result_json = json!({
            "actor_id": actor_id,
            "status": "RUNNING"
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
    
    /// Register actor tools with the MCP tool manager
    pub fn register_tools(
        self: Arc<Self>,
        tool_manager: &Arc<mcp_server::tools::ToolManager>,
    ) {
        // Register start_actor tool
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
        tool_manager.register_tool(start_actor_tool, move |args| {
            let tools_self = tools_self.clone();
            let fut = tools_self.start_actor(args);
            
            // Use the current runtime handle instead of creating a new one
            let handle = tokio::runtime::Handle::current();
            handle.block_on(fut)
        });
        
        // Register stop_actor tool
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
        tool_manager.register_tool(stop_actor_tool, move |args| {
            let tools_self = tools_self.clone();
            let fut = tools_self.stop_actor(args);
            
            // Convert async result to sync
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(fut)
        });
        
        // Register restart_actor tool
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
        tool_manager.register_tool(restart_actor_tool, move |args| {
            let tools_self = tools_self.clone();
            let fut = tools_self.restart_actor(args);
            
            // Convert async result to sync
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(fut)
        });
    }
}
