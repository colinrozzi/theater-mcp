use anyhow::{anyhow, Result};
use mcp_protocol::types::resource::{Resource, ResourceContent, ResourceTemplate};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, warn};

use theater::id::TheaterId;
use crate::theater::client::TheaterClient;
use crate::theater::TheaterIdExt;

/// Resources for accessing Theater events
pub struct EventResources {
    theater_client: Arc<TheaterClient>,
}

impl EventResources {
    /// Create a new event resources instance
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
    
    /// Get resource content for an actor's events
    pub async fn get_actor_events_content(&self, actor_id: &str) -> Result<ResourceContent> {
        debug!("Getting events for actor {}", actor_id);
        
        // Convert string ID to TheaterId
        let theater_id = TheaterId::from_str(actor_id)?;
        
        // Get actor events with connection error handling
        let events = self.handle_connection_error(
            self.theater_client.get_actor_events(&theater_id).await,
            &format!("actor events retrieval for {}", actor_id)
        )?;
        
        // Return the events as JSON
        Ok(ResourceContent {
            uri: format!("theater://events/{}", actor_id),
            mime_type: "application/json".to_string(),
            text: Some(json!(events).to_string()),
            blob: None,
        })
    }
    
    /// Register a specific actor's event resources
    pub async fn register_actor_events(
        self: Arc<Self>,
        actor_id: String,
        resource_manager: Arc<mcp_server::resources::ResourceManager>,
    ) -> Result<()> {
        // Convert string ID to TheaterId
        let theater_id = TheaterId::from_str(&actor_id)?;
        
        // Check if actor exists
        if !self.theater_client.actor_exists(&theater_id).await? {
            return Err(anyhow!("Actor not found: {}", actor_id));
        }
        
        // Register actor events resource
        let events_resource = Resource {
            uri: format!("theater://events/{}", actor_id),
            name: format!("Actor {} Events", actor_id),
            description: Some("Event history for a specific actor".to_string()),
            mime_type: Some("application/json".to_string()),
            size: None,
            annotations: None,
        };
        
        let events_self = self.clone();
        let events_actor_id = actor_id.clone();
        // Create a safe content provider that won't block the current async context
        let self_ref = events_self.clone();
        let aid = events_actor_id.clone();
        
        resource_manager.register_resource(
            events_resource,
            move || {
                // Clone for the thread
                let self_ref = self_ref.clone();
                let aid = aid.clone();
                
                // Use a thread-safe channel to communicate between threads
                let (tx, rx) = std::sync::mpsc::channel();
                
                // Spawn a new thread to run the future
                std::thread::spawn(move || {
                    // Create a new runtime for this thread only
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();
                    
                    // Run the async code in this isolated runtime
                    let result = rt.block_on(async {
                        self_ref.get_actor_events_content(&aid).await
                    });
                    
                    // Send the result back to the main thread
                    let _ = tx.send(result.map(|content| vec![content]));
                });
                
                // Receive the result - this is a blocking operation but we're not in an async context here
                rx.recv().unwrap_or_else(|e| {
                    Err(anyhow::anyhow!("Failed to get actor events: {}", e))
                })
            }
        );

        
        Ok(())
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