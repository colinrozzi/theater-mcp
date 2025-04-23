use anyhow::{anyhow, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;
use tracing::{trace, error};
use uuid::Uuid;

use crate::theater::types::TheaterError;

/// Client for connecting to and interacting with a Theater server
pub struct TheaterClient {
    connection: Arc<Mutex<TcpStream>>,
}

impl TheaterClient {
    /// Connect to a Theater server at the given address
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(addr).await
            .map_err(|e| anyhow!("Failed to connect to Theater server: {}", e))?;
        
        Ok(Self {
            connection: Arc::new(Mutex::new(stream)),
        })
    }
    
    /// Send a command to the Theater server and receive a response
    async fn send_command(&self, command: Value) -> Result<Value> {
        // Create message frame
        let message = serde_json::to_vec(&command)?;
        let len = message.len() as u32;
        let len_bytes = len.to_be_bytes();
        
        // Get connection lock
        let mut connection = self.connection.lock().await;
        
        trace!("Sending command: {:?}", command);
        
        // Write length prefix and message
        connection.write_all(&len_bytes).await?;
        connection.write_all(&message).await?;
        
        // Read response length
        let mut len_buf = [0u8; 4];
        connection.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;
        
        // Read response
        let mut response_buf = vec![0u8; len];
        connection.read_exact(&mut response_buf).await?;
        
        // Parse response
        let response: Value = serde_json::from_slice(&response_buf)?;
        trace!("Received response: {:?}", response);
        
        // Check for error
        if let Some(error) = response.get("error") {
            let message = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            
            return Err(TheaterError::ServerError(message).into());
        }
        
        Ok(response)
    }
    
    /// List all running actors
    pub async fn list_actors(&self) -> Result<Vec<String>> {
        // In this version, the key is just the command name without the method/id structure
        let command = json!({
            "ListActors": {}
        });
        
        let response = self.send_command(command).await?;
        
        // Extract actor IDs from response
        let actors = response
            .get("actors")
            .and_then(|a| a.as_array())
            .ok_or_else(|| anyhow!("Invalid response format"))?
            .iter()
            .filter_map(|a| a.as_str().map(String::from))
            .collect();
            
        Ok(actors)
    }
    
    /// Start a new actor from a manifest
    pub async fn start_actor(&self, manifest: &str, initial_state: Option<&[u8]>) -> Result<String> {
        let initial_state_value = if let Some(state) = initial_state {
            Value::String(BASE64.encode(state))
        } else {
            Value::Null
        };
        
        // The Theater server expects direct command objects, not JSON-RPC style
        let command = json!({
            "StartActor": {
                "manifest": manifest,
                "initial_state": initial_state_value
            }
        });
        
        let response = self.send_command(command).await?;
        
        // Extract actor ID from response
        let actor_id = response
            .get("id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| anyhow!("Invalid response format"))?
            .to_string();
            
        Ok(actor_id)
    }
    
    /// Stop a running actor
    pub async fn stop_actor(&self, actor_id: &str) -> Result<()> {
        let command = json!({
            "StopActor": {
                "actor_id": actor_id
            }
        });
        
        let _response = self.send_command(command).await?;
        Ok(())
    }
    
    /// Restart a running actor
    pub async fn restart_actor(&self, actor_id: &str) -> Result<()> {
        let command = json!({
            "RestartActor": {
                "actor_id": actor_id
            }
        });
        
        let _response = self.send_command(command).await?;
        Ok(())
    }
    
    /// Get the current state of an actor
    pub async fn get_actor_state(&self, actor_id: &str) -> Result<Option<Value>> {
        let command = json!({
            "GetActorState": {
                "actor_id": actor_id
            }
        });
        
        let response = self.send_command(command).await?;
        
        // Extract state from response
        let state = response.get("state");
            
        if let Some(state) = state {
            if state.is_null() {
                return Ok(None);
            } else {
                return Ok(Some(state.clone()));
            }
        }
        
        Ok(None)
    }
    
    /// Get the event history for an actor
    pub async fn get_actor_events(&self, actor_id: &str) -> Result<Vec<Value>> {
        let command = json!({
            "GetActorEvents": {
                "actor_id": actor_id
            }
        });
        
        let response = self.send_command(command).await?;
        
        // Extract events from response
        let events = response
            .get("events")
            .and_then(|e| e.as_array())
            .ok_or_else(|| anyhow!("Invalid response format"))?
            .clone();
            
        Ok(events)
    }
    
    /// Send a one-way message to an actor
    pub async fn send_message(&self, actor_id: &str, data: &[u8]) -> Result<()> {
        let command = json!({
            "SendActorMessage": {
                "actor_id": actor_id,
                "data": BASE64.encode(data)
            }
        });
        
        let _response = self.send_command(command).await?;
        Ok(())
    }
    
    /// Send a request to an actor and receive a response
    pub async fn request_message(&self, actor_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        let command = json!({
            "RequestActorMessage": {
                "actor_id": actor_id,
                "data": BASE64.encode(data)
            }
        });
        
        let response = self.send_command(command).await?;
        
        // Extract response data
        let response_data = response
            .get("data")
            .and_then(|d| d.as_str())
            .ok_or_else(|| anyhow!("Invalid response format"))?;
            
        let data = BASE64.decode(response_data)?;
        Ok(data)
    }
    
    /// Open a channel to an actor
    pub async fn open_channel(&self, actor_id: &str, initial_message: Option<&[u8]>) -> Result<String> {
        let initial_data = if let Some(data) = initial_message {
            BASE64.encode(data)
        } else {
            "".to_string()
        };
        
        let command = json!({
            "OpenChannel": {
                "actor_id": actor_id,
                "initial_message": initial_data
            }
        });
        
        let response = self.send_command(command).await?;
        
        // Extract channel ID
        let channel_id = response
            .get("channel_id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| anyhow!("Invalid response format"))?
            .to_string();
            
        Ok(channel_id)
    }
    
    /// Send a message on an open channel
    pub async fn send_on_channel(&self, channel_id: &str, message: &[u8]) -> Result<()> {
        let command = json!({
            "SendOnChannel": {
                "channel_id": channel_id,
                "message": BASE64.encode(message)
            }
        });
        
        let _response = self.send_command(command).await?;
        Ok(())
    }
    
    /// Close an open channel
    pub async fn close_channel(&self, channel_id: &str) -> Result<()> {
        let command = json!({
            "CloseChannel": {
                "channel_id": channel_id
            }
        });
        
        let _response = self.send_command(command).await?;
        Ok(())
    }
}
