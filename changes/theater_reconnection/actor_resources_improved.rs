use anyhow::{anyhow, Result};
use mcp_protocol::types::resource::{Resource, ResourceContent, ResourceTemplate};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, warn};
use tokio::runtime::Handle;
use tokio::task;
use std::future::Future;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use theater::id::TheaterId;
use crate::theater::client::TheaterClient;
use crate::theater::TheaterIdExt;

/// Resources for accessing Theater actors
pub struct ActorResources {
    theater_client: Arc<TheaterClient>,
}

impl ActorResources {
    /// Create a new actor resources instance
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
                    Err(anyhow!("Theater server connection issue: {}. The server will attempt to reconnect on the next request.", error_msg))
                } else {
                    // Other type of error
                    Err(e)
                }
            }
        }
    }
    
    /// Get resource content for the actor list
    pub async fn get_actors_list_content(&self) -> Result<ResourceContent> {
        debug!("Getting actor list content");
        
        // Get actors with connection error handling
        let actor_ids = self.handle_connection_error(
            self.theater_client.list_actors().await,
            "actor list retrieval"
        )?;
        
        let actors = actor_ids.iter().map(|id| {
            json!({
                "id": id.as_string(),
                "name": format!("Actor {}", id),
                "status": "RUNNING",
                "uri": format!("theater://actor/{}", id.as_string())
            })
        }).collect::<Vec<_>>();
        
        let content = json!({
            "actors": actors,
            "total": actors.len()
        });
        
        Ok(ResourceContent {
            uri: "theater://actors".to_string(),
            mime_type: "application/json".to_string(),
            text: Some(content.to_string()),
            blob: None,
        })
    }
    
    /// Get resource content for an actor's details
    pub async fn get_actor_details_content(&self, actor_id: &str) -> Result<ResourceContent> {
        debug!("Getting actor details for {}", actor_id);
        
        // Convert string ID to TheaterId
        let theater_id = TheaterId::from_str(actor_id)?;
        
        // Attempt to get the actor state to verify it exists with connection error handling
        if let Err(e) = self.handle_connection_error(
            self.theater_client.get_actor_state(&theater_id).await,
            &format!("actor details retrieval for {}", actor_id)
        ) {
            debug!("Failed to get actor state: {}", e);
            return Err(anyhow!("Actor not found or connection issue: {}", actor_id));
        }
        
        let content = json!({
            "id": actor_id,
            "status": "RUNNING", // We're simplifying for now
            "created_at": chrono::Utc::now().to_rfc3339(),
            "events_uri": format!("theater://events/{}", actor_id),
            "state_uri": format!("theater://actor/{}/state", actor_id)
        });
        
        Ok(ResourceContent {
            uri: format!("theater://actor/{}", actor_id),
            mime_type: "application/json".to_string(),
            text: Some(content.to_string()),
            blob: None,
        })
    }
    
    /// Get resource content for an actor's state
    pub async fn get_actor_state_content(&self, actor_id: &str) -> Result<ResourceContent> {
        debug!("Getting actor state for {}", actor_id);
        
        // Convert string ID to TheaterId
        let theater_id = TheaterId::from_str(actor_id)?;
        
        // Get the actor state with connection error handling
        let state_result = self.handle_connection_error(
            self.theater_client.get_actor_state(&theater_id).await,
            &format!("actor state retrieval for {}", actor_id)
        )?;
        
        // Process the state
        let content = if let Some(state_bytes) = state_result {
            // Try to parse the binary data as JSON
            match serde_json::from_slice::<serde_json::Value>(&state_bytes) {
                Ok(json_value) => json_value,
                Err(_) => {
                    // If not valid JSON, encode as base64
                    let base64_str = BASE64.encode(&state_bytes);
                    json!({
                        "_raw_state_base64": base64_str
                    })
                }
            }
        } else {
            // No state available
            json!({
                "_state": "empty"
            })
        };
        
        Ok(ResourceContent {
            uri: format!("theater://actor/{}/state", actor_id),
            mime_type: "application/json".to_string(),
            text: Some(content.to_string()),
            blob: None,
        })
    }
    
    /// Register actor resources with the MCP resource manager
    pub async fn register_actor_resources(
        self: Arc<Self>,
        actor_id: String,
        resource_manager: Arc<mcp_server::resources::ResourceManager>,
    ) -> Result<()> {
        // Actor details resource
        let actor_details_uri = format!("theater://actor/{}", actor_id);
        let actor_details_resource = Resource {
            uri: actor_details_uri.clone(),
            mime_type: "application/json".to_string(),
            template: None,
            description: Some(format!("Details for actor {}", actor_id)),
            annotations: None,
        };
        
        let self_clone = self.clone();
        resource_manager.register_resource(
            actor_details_resource,
            Box::new(move |_| {
                let self_clone = self_clone.clone();
                let actor_id = actor_id.clone();
                Box::pin(async move {
                    self_clone.get_actor_details_content(&actor_id).await
                })
            }),
        )?;
        
        // Actor state resource
        let actor_state_uri = format!("theater://actor/{}/state", actor_id);
        let actor_state_resource = Resource {
            uri: actor_state_uri.clone(),
            mime_type: "application/json".to_string(),
            template: None,
            description: Some(format!("Current state for actor {}", actor_id)),
            annotations: None,
        };
        
        let self_clone = self.clone();
        resource_manager.register_resource(
            actor_state_resource,
            Box::new(move |_| {
                let self_clone = self_clone.clone();
                let actor_id = actor_id.clone();
                Box::pin(async move {
                    self_clone.get_actor_state_content(&actor_id).await
                })
            }),
        )?;
        
        Ok(())
    }
    
    /// Register resources with the MCP resource manager
    pub fn register_resources(
        self: Arc<Self>,
        resource_manager: &Arc<mcp_server::resources::ResourceManager>,
    ) {
        // Register the actors list resource
        let actors_list_resource = Resource {
            uri: "theater://actors".to_string(),
            mime_type: "application/json".to_string(),
            template: None,
            description: Some("List of actors in the Theater system".to_string()),
            annotations: None,
        };
        
        let self_clone = self.clone();
        if let Err(e) = resource_manager.register_resource(
            actors_list_resource,
            Box::new(move |_| {
                let self_clone = self_clone.clone();
                Box::pin(async move {
                    self_clone.get_actors_list_content().await
                })
            }),
        ) {
            warn!("Failed to register actors list resource: {}", e);
        }
    }
}