use anyhow::Result;
use mcp_protocol::types::resource::{Resource, ResourceContent, ResourceTemplate, ResourceTemplateParameter};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error};

use crate::theater::client::TheaterClient;

/// Resources for accessing Theater events
pub struct EventResources {
    theater_client: Arc<TheaterClient>,
}

impl EventResources {
    /// Create a new event resources instance
    pub fn new(theater_client: Arc<TheaterClient>) -> Self {
        Self { theater_client }
    }
    
    /// Get resource content for an actor's events
    pub async fn get_actor_events_content(&self, actor_id: &str) -> Result<ResourceContent> {
        debug!("Getting events for actor {}", actor_id);
        let events = self.theater_client.get_actor_events(actor_id).await?;
        
        // Return the events as JSON
        Ok(ResourceContent::Json { json: json!(events) })
    }
    
    /// Register event resources with the MCP resource manager
    pub fn register_resources(
        self: Arc<Self>,
        resource_manager: &Arc<mcp_server::resources::ResourceManager>,
    ) {
        // Register the actor events resource template
        let events_template = ResourceTemplate {
            uri_template: "theater://events/{actor_id}".to_string(),
            mime_type: Some("application/json".to_string()),
            is_directory: Some(false),
            annotations: None,
            parameters: vec![
                ResourceTemplateParameter {
                    name: "actor_id".to_string(),
                    description: Some("ID of the actor".to_string()),
                    required: Some(true),
                },
            ],
        };
        
        let events_self = self.clone();
        resource_manager.register_template(events_template, move |params| {
            let events_self = events_self.clone();
            Box::pin(async move {
                // Extract actor_id parameter
                let actor_id = params
                    .get("actor_id")
                    .ok_or_else(|| anyhow::anyhow!("Missing actor_id parameter"))?;
                
                match events_self.get_actor_events_content(actor_id).await {
                    Ok(content) => Ok(vec![content]),
                    Err(e) => Err(e),
                }
            })
        });
    }
}
