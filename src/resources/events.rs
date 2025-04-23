use anyhow::Result;
use mcp_protocol::types::resource::{ResourceContent, ResourceTemplate};
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

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
        Ok(ResourceContent {
            uri: format!("theater://events/{}", actor_id),
            mime_type: "application/json".to_string(),
            text: Some(json!(events).to_string()),
            blob: None,
        })
    }
    
    /// Register event resources with the MCP resource manager
    pub fn register_resources(
        self: Arc<Self>,
        resource_manager: &Arc<mcp_server::resources::ResourceManager>,
    ) {
        // Register the actor events resource template
        let events_template = ResourceTemplate {
            uri_template: "theater://events/{actor_id}".to_string(),
            name: "Actor Events".to_string(),
            description: Some("Event chain for a specific actor".to_string()),
            mime_type: Some("application/json".to_string()),
            annotations: None,
        };
        
        resource_manager.register_template(events_template, move |uri, _params| {
            // We just need to return the expanded URI here
            Ok(uri)
        });
    }
}
