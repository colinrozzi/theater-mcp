use anyhow::{anyhow, Result};
use mcp_protocol::types::tool::{Tool, ToolCallResult, ToolContent};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, warn};

use theater::id::TheaterId;
use crate::theater::client::TheaterClient;
use crate::theater::TheaterIdExt;
use crate::tools::utils::register_async_tool;

pub struct ActorTools {
    theater_client: Arc<TheaterClient>,
    resource_manager: Option<Arc<mcp_server::resources::ResourceManager>>,
    actor_resources: Option<Arc<crate::resources::ActorResources>>,
    event_resources: Option<Arc<crate::resources::EventResources>>,
}

impl ActorTools {
    pub fn new(theater_client: Arc<TheaterClient>) -> Self {
        Self {
            theater_client,
            resource_manager: None,
            actor_resources: None,
            event_resources: None,
        }
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
                    Err(anyhow!("Theater server connection issue: {}. The server will attempt to reconnect on the next request.", error_msg))
                } else {
                    // Other type of error
                    Err(e)
                }
            }
        }
    }
    
    pub fn with_resources(
        mut self,
        resource_manager: Arc<mcp_server::resources::ResourceManager>,
        actor_resources: Arc<crate::resources::ActorResources>,
        event_resources: Arc<crate::resources::EventResources>,
    ) -> Self {
        self.resource_manager = Some(resource_manager);
        self.actor_resources = Some(actor_resources);
        self.event_resources = Some(event_resources);
        self
    }
    
    pub async fn start_actor(&self, args: Value) -> Result<ToolCallResult> {
        // Extract manifest path
        let manifest = args["manifest"].as_str()
            .ok_or_else(|| anyhow!("Missing manifest parameter"))?;
            
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
            Some(ref bytes) => {
                self.handle_connection_error(
                    self.theater_client.start_actor(manifest, Some(bytes.as_slice())).await,
                    "actor start"
                )?
            },
            None => {
                self.handle_connection_error(
                    self.theater_client.start_actor(manifest, None).await,
                    "actor start"
                )?
            },
        };
        
        // Register resources for this actor if resource managers are available
        let actor_id_str = actor_id.as_string();
        if let (Some(rm), Some(ar), Some(er)) = (
            &self.resource_manager,
            &self.actor_resources,
            &self.event_resources
        ) {
            // Prepare resource registration
            let actor_resources_fut = ar.clone().register_actor_resources(actor_id_str.clone(), rm.clone());
            let event_resources_fut = er.clone().register_actor_events(actor_id_str.clone(), rm.clone());
            
            // Execute them in parallel
            tokio::spawn(async move {
                if let Err(e) = actor_resources_fut.await {
                    error!("Error registering actor resources: {}", e);
                    // Continue anyway, don't fail the actor start
                }
                
                if let Err(e) = event_resources_fut.await {
                    error!("Error registering event resources: {}", e);
                    // Continue anyway, don't fail the actor start
                }
            });
        }
        
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
    
    pub async fn stop_actor(&self, args: Value) -> Result<ToolCallResult> {
        // Extract actor ID
        let actor_id_str = args["actor_id"].as_str()
            .ok_or_else(|| anyhow!("Missing actor_id parameter"))?;
         
        // Convert to TheaterId
        let theater_id = TheaterId::from_str(actor_id_str)?;
            
        // Stop the actor with connection error handling
        self.handle_connection_error(
            self.theater_client.stop_actor(&theater_id).await,
            "actor stop"
        )?;
        
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
            .ok_or_else(|| anyhow!("Missing actor_id parameter"))?;
            
        // Convert to TheaterId
        let theater_id = TheaterId::from_str(actor_id_str)?;
            
        // Restart the actor with connection error handling
        self.handle_connection_error(
            self.theater_client.restart_actor(&theater_id).await,
            "actor restart"
        )?;
        
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