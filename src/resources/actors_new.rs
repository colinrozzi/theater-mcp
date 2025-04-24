use anyhow::Result;
use mcp_protocol::types::resource::{Resource, ResourceContent, ResourceTemplate};
use serde_json::json;
use std::sync::Arc;
use tracing::debug;
use tokio::runtime::Handle;
use tokio::task;
use std::future::Future;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::theater::client::TheaterClient;

/// Resources for accessing Theater actors
pub struct ActorResources {
    theater_client: Arc<TheaterClient>,
}

impl ActorResources {
    /// Create a new actor resources instance
    pub fn new(theater_client: Arc<TheaterClient>) -> Self {
        Self { theater_client }
    }
    
    /// Get resource content for the actor list
    pub async fn get_actors_list_content(&self) -> Result<ResourceContent> {
        debug!("Getting actor list content");
        let actor_ids = self.theater_client.list_actors().await?;
        
        let actors = actor_ids.iter().map(|id| {
            json!({
                "id": id,
                "name": format!("Actor {}", id),
                "status": "RUNNING",
                "uri": format!("theater://actor/{}", id)
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
        
        // Attempt to get the actor state to verify it exists
        if let Err(e) = self.theater_client.get_actor_state(actor_id).await {
            debug!("Failed to get actor state: {}", e);
            return Err(anyhow::anyhow!("Actor not found: {}", actor_id));
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
        
        // Get the actor state
        let state_result = self.theater_client.get_actor_state(actor_id).await?;
        
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
            // No state, return empty object
            json!({})
        };
        
        // Create the resource content
        Ok(ResourceContent {
            uri: format!("theater://actor/{}/state", actor_id),
            mime_type: "application/json".to_string(),
            text: Some(content.to_string()),
            blob: None,
        })
    }
    
    // Helper function to spawn an async task that can be used in sync callbacks
    fn spawn_blocking<F, Fut, T>(f: F) -> Result<T>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        match Handle::try_current() {
            Ok(_handle) => {
                // We're already in a tokio runtime, use spawn_blocking
                match task::block_in_place(|| {
                    let rt = Handle::current();
                    rt.block_on(f())
                }) {
                    Ok(result) => Ok(result), // Wrap the result in Ok
                    Err(e) => Err(anyhow::anyhow!("Task execution error: {}", e)),
                }
            },
            Err(_) => {
                // No runtime, this is unexpected but try a direct approach
                Err(anyhow::anyhow!("No Tokio runtime available"))
            }
        }
    }
    
    /// Register actor resources with the MCP resource manager
    pub fn register_resources(
        self: Arc<Self>,
        resource_manager: &Arc<mcp_server::resources::ResourceManager>,
    ) {
        // Register the actor list resource
        let actors_list_resource = Resource {
            uri: "theater://actors".to_string(),
            name: "Theater Actors".to_string(),
            description: Some("List of all running actors in the Theater system".to_string()),
            mime_type: Some("application/json".to_string()),
            size: None,
            annotations: None,
        };
        
        let actors_self = self.clone();
        resource_manager.register_resource(actors_list_resource, move || {
            let actors_self = actors_self.clone();
            
            Self::spawn_blocking(move || async move {
                let content = actors_self.get_actors_list_content().await?;
                Ok(vec![content])
            })
        });
        
        // Register the actor details resource template
        let actor_details_template = ResourceTemplate {
            uri_template: "theater://actor/{actor_id}".to_string(),
            name: "Actor Details".to_string(),
            description: Some("Detailed information about a specific actor".to_string()),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        };
        
        resource_manager.register_template(actor_details_template, move |uri, _params| {
            // We just need to return the expanded URI here,
            // the content will be fetched through a separate mechanism
            Ok(uri)
        });
        
        // Register the actor state resource template
        let actor_state_template = ResourceTemplate {
            uri_template: "theater://actor/{actor_id}/state".to_string(),
            name: "Actor State".to_string(),
            description: Some("Current state of a specific actor".to_string()),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        };
        
        resource_manager.register_template(actor_state_template, move |uri, _params| {
            // We just need to return the expanded URI here
            Ok(uri)
        });
    }
}