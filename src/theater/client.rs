use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use theater::theater_server::ManagementCommand;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::trace;
use uuid::Uuid;

use crate::theater::types::TheaterError;

/// Client for connecting to and interacting with a Theater server
pub struct TheaterClient {
    connection: Arc<Mutex<TcpStream>>,
}

impl TheaterClient {
    /// Connect to a Theater server at the given address
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| anyhow!("Failed to connect to Theater server: {}", e))?;

        Ok(Self {
            connection: Arc::new(Mutex::new(stream)),
        })
    }

    /// Send a command to the Theater server and receive a response
    async fn send_command(&self, command: ManagementCommand) -> Result<Value> {
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
            let message = error
                .get("message")
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
        let command = theater::theater_server::ManagementCommand::ListActors;

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
    pub async fn start_actor(
        &self,
        manifest: &str,
        initial_state: Option<&[u8]>,
    ) -> Result<String> {
        // The Theater server expects initial_state as a sequence of bytes, not a base64 string
        let initial_state_value = if let Some(state) = initial_state {
            // Convert the bytes to an array of numbers (u8 values)
            let byte_array: Vec<u8> = state.to_vec();
            Value::Array(
                byte_array
                    .into_iter()
                    .map(|b| Value::Number(b.into()))
                    .collect(),
            )
        } else {
            Value::Null
        };

        // The Theater server expects direct command objects, not JSON-RPC style
        // Do not include an id field for Theater commands
        let command = serde_json::json!({
            "StartActor": {
                "manifest": manifest,
                "initial_state": initial_state_value
            }
        });

        let response = self.send_command(command).await?;

        // Debug the response to understand its structure
        trace!("Start actor response: {:?}", response);

        // Extract actor ID from response - the structure may vary
        // Based on the actual response: {"ActorStarted": {"id": "1edf3c18-43c0-46f0-80a9-cdcabdf5d137"}}
        let actor_id = if let Some(id) = response.get("id").and_then(|id| id.as_str()) {
            id.to_string()
        } else if let Some(id) = response.get("actor_id").and_then(|id| id.as_str()) {
            id.to_string()
        } else if let Some(actor_started) = response.get("ActorStarted") {
            // Extract ID from ActorStarted event
            actor_started
                .get("id")
                .and_then(|id| id.as_str())
                .ok_or_else(|| anyhow!("Missing id in ActorStarted event: {:?}", actor_started))?
                .to_string()
        } else {
            // Dump the entire response to make debugging easier
            return Err(anyhow!(
                "Could not find actor ID in response: {:?}",
                response
            ));
        };

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
        // Convert the bytes to an array of numbers for Theater's protocol
        let byte_array: Vec<u8> = data.to_vec();
        let data_array = Value::Array(
            byte_array
                .into_iter()
                .map(|b| Value::Number(b.into()))
                .collect(),
        );

        let command = json!({
            "SendActorMessage": {
                "id": actor_id,
                "data": data_array
            },
            "id": Uuid::new_v4().to_string()
        });

        let _response = self.send_command(command).await?;
        Ok(())
    }

    /// Send a request to an actor and receive a response
    pub async fn request_message(&self, actor_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        // Convert the bytes to an array of numbers for Theater's protocol
        let byte_array: Vec<u8> = data.to_vec();
        let data_array = Value::Array(
            byte_array
                .into_iter()
                .map(|b| Value::Number(b.into()))
                .collect(),
        );

        let command = json!({
            "RequestActorMessage": {
                "id": actor_id,
                "data": data_array
            },
            "id": Uuid::new_v4().to_string()
        });

        let response = self.send_command(command).await?;

        // Extract response data - Theater may return an array of bytes
        let response_data = response
            .get("data")
            .ok_or_else(|| anyhow!("Response missing data field"))?;

        // Handle different formats of response data
        let data = if response_data.is_array() {
            // Handle byte array format
            response_data
                .as_array()
                .ok_or_else(|| anyhow!("Invalid data format"))?
                .iter()
                .filter_map(|v| v.as_u64().map(|n| n as u8))
                .collect()
        } else if response_data.is_string() {
            // Handle base64 string format
            let base64_str = response_data
                .as_str()
                .ok_or_else(|| anyhow!("Invalid data format"))?;
            BASE64.decode(base64_str)?
        } else {
            return Err(anyhow!("Unexpected data format in response"));
        };

        Ok(data)
    }

    /// Open a channel to an actor
    pub async fn open_channel(
        &self,
        actor_id: &str,
        initial_message: Option<&[u8]>,
    ) -> Result<String> {
        // Handle initial message as a byte array if present
        let initial_data = if let Some(data) = initial_message {
            let byte_array: Vec<u8> = data.to_vec();
            Value::Array(
                byte_array
                    .into_iter()
                    .map(|b| Value::Number(b.into()))
                    .collect(),
            )
        } else {
            Value::Array(vec![])
        };

        let command = json!({
            "OpenChannel": {
                "id": actor_id,
                "initial_message": initial_data
            },
            "id": Uuid::new_v4().to_string()
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
        // Convert the bytes to an array of numbers for Theater's protocol
        let byte_array: Vec<u8> = message.to_vec();
        let message_array = Value::Array(
            byte_array
                .into_iter()
                .map(|b| Value::Number(b.into()))
                .collect(),
        );

        let command = json!({
            "SendOnChannel": {
                "channel_id": channel_id,
                "message": message_array
            },
            "id": Uuid::new_v4().to_string()
        });

        let _response = self.send_command(command).await?;
        Ok(())
    }

    /// Close an open channel
    pub async fn close_channel(&self, channel_id: &str) -> Result<()> {
        let command = json!({
            "CloseChannel": {
                "channel_id": channel_id
            },
            "id": Uuid::new_v4().to_string()
        });

        let _response = self.send_command(command).await?;
        Ok(())
    }
}
