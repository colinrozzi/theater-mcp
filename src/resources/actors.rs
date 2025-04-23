use anyhow::Result;
use mcp_protocol::types::resource::{Resource, ResourceContent};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, error};

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
        
        Ok(ResourceContent::Json { json: content })
    }
    
    /// Get resource content for an actor's details
    pub async fn get_actor_details_content(&self, actor_id: &str) -> Result<ResourceContent> {
        debug!("Getting actor details for {}", actor_id);
        
        // Attempt to get the actor state to verify it exists
        if let Err(e) = self.theater_client.get_actor_state(actor_id).await {
            error!("Failed to get actor state: {}", e);
            return Err(anyhow::anyhow!("Actor not found: {}", actor_id));
        }
        
        let content = json!({
            "id": actor_id,
            "status": "RUNNING", // We're simplifying for now
            "created_at": chrono::Utc::now().to_rfc3339(),
            "events_uri": format!("theater://events/{}", actor_id),
            "state_uri": format!("theater://actor/{}/state", actor_id)
        });
        
        Ok(ResourceContent::Json { json: content })
    }
    
    /// Get resource content for an actor's state
    pub async fn get_actor_state_content(&self, actor_id: &str) -> Result<ResourceContent> {
        debug!("Getting actor state for {}", actor_id);
        let state = self.theater_client.get_actor_state(actor_id).await?;
        
        if let Some(state_value) = state {
            return Ok(ResourceContent::Json { json: state_value });
        }
        
        // Return empty JSON if no state
        Ok(ResourceContent::Json { json: json!({}) })
    }
    
    /// Register actor resources with the MCP resource manager
    pub fn register_resources(
        self: Arc<Self>,
        resource_manager: &Arc<mcp_server::resources::ResourceManager>,
    ) {
        // Register the actor list resource
        let actors_list_resource = Resource {
            uri: "theater://actors".to_string(),
            mime_type: Some("application/json".to_string()),
            is_directory: Some(false),
            annotations: None,
        };
        
        let actors_self = self.clone();
        resource_manager.register_resource(actors_list_resource, move || {
            let actors_self = actors_self.clone();
            Box::pin(async move {
                match actors_self.get_actors_list_content().await {
                    Ok(content) => Ok(vec![content]),
                    Err(e) => Err(e),
                }
            })
        });
        
        // Register the actor details resource template
        let actor_details_template = mcp_protocol::types::resource::ResourceTemplate {
            uri_template: "theater://actor/{actor_id}".to_string(),
            mime_type: Some("application/json".to_string()),
            is_directory: Some(false),
            annotations: None,
            parameters: vec![
                mcp_protocol::types::resource::ResourceTemplateParameter {
                    name: "actor_id".to_string(),
                    description: Some("ID of the actor".to_string()),
                    required: Some(true),
                },
            ],
        };
        
        let actors_self = self.clone();
        resource_manager.register_template(actor_details_template, move |params| {
            let actors_self = actors_self.clone();
            Box::pin(async move {
                // Extract actor_id parameter
                let actor_id = params
                    .get("actor_id")
                    .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
                
                match actors_self.get_actor_details_content(actor_id).await {
                    Ok(content) => Ok(vec![content]),
                    Err(e) => Err(e),
                }
            })
        });
        
        // Register the actor state resource template
        let actor_state_template = mcp_protocol::types::resource::ResourceTemplate {
            uri_template: "theater://actor/{actor_id}/state".to_string(),
            mime_type: Some("application/json".to_string()),
            is_directory: Some(false),
            annotations: None,
            parameters: vec![
                mcp_protocol::types::resource::ResourceTemplateParameter {
                    name: "actor_id".to_string(),
                    description: Some("ID of the actor".to_string()),
                    required: Some(true),
                },
            ],
        };
        
        let actors_self = self.clone();
        resource_manager.register_template(actor_state_template, move |params| {
            let actors_self = actors_self.clone();
            Box::pin(async move {
                // Extract actor_id parameter
                let actor_id = params
                    .get("actor_id")
                    .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
                
                match actors_self.get_actor_state_content(actor_id).await {
                    Ok(content) => Ok(vec![content]),
                    Err(e) => Err(e),
                }
            })
        });
    }
}
