use anyhow::{anyhow, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{trace, warn, error, info};

use theater::id::TheaterId;
use theater::theater_server::{ManagementCommand, ManagementResponse};
use theater::messages::ChannelParticipant;
use theater::chain::ChainEvent;

use crate::theater::types::TheaterError;

/// Client for connecting to and interacting with a Theater server
/// with automatic reconnection capabilities
#[derive(Debug)]
pub struct TheaterClient {
    connection: Arc<Mutex<Option<TcpStream>>>,
    address: SocketAddr,
    is_connecting: Arc<AtomicBool>,
}

impl TheaterClient {
    /// Connect to a Theater server at the given address
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| anyhow!("Failed to connect to Theater server: {}", e))?;

        info!("Connected to Theater server at {}", addr);
        
        Ok(Self {
            connection: Arc::new(Mutex::new(Some(stream))),
            address: addr,
            is_connecting: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Ensure that we have a valid connection to the Theater server
    async fn ensure_connected(&self) -> Result<()> {
        let mut connection_guard = self.connection.lock().await;
        
        // If we already have a connection, check if it's still valid
        if let Some(conn) = &mut *connection_guard {
            // Try a small write to test connection (0-length write is a good way to test)
            if let Err(e) = conn.write_all(&[0; 0]).await {
                warn!("Connection test failed: {}. Will attempt to reconnect.", e);
                // Connection is broken, clear it
                *connection_guard = None;
            }
        }
        
        // If connection is None, create a new connection
        if connection_guard.is_none() {
            // Use atomic flag to prevent multiple reconnection attempts
            if !self.is_connecting.swap(true, Ordering::SeqCst) {
                // Try to establish a new connection
                match TcpStream::connect(self.address).await {
                    Ok(stream) => {
                        *connection_guard = Some(stream);
                        info!("Successfully reconnected to Theater server at {}", self.address);
                    },
                    Err(e) => {
                        error!("Failed to reconnect to Theater server: {}", e);
                        self.is_connecting.store(false, Ordering::SeqCst);
                        return Err(anyhow!("Failed to connect to Theater server: {}", e));
                    }
                }
                self.is_connecting.store(false, Ordering::SeqCst);
            } else {
                // Another thread is already trying to connect
                return Err(anyhow!("Connection attempt already in progress"));
            }
        }
        
        Ok(())
    }

    /// Send a command to the Theater server and receive a response
    /// With automatic reconnection on failure
    async fn send_command(&self, command: ManagementCommand) -> Result<ManagementResponse> {
        let max_attempts = 3;
        let mut backoff_ms = 500; // Start with 500ms backoff
        
        for attempt in 1..=max_attempts {
            // Ensure we have a connection before proceeding
            if let Err(e) = self.ensure_connected().await {
                if attempt == max_attempts {
                    return Err(anyhow!("Failed to establish connection after {} attempts: {}", max_attempts, e));
                }
                
                // Wait before retrying with exponential backoff
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                backoff_ms *= 2; // Exponential backoff
                continue;
            }
            
            // Create message frame
            let message = serde_json::to_vec(&command)?;
            let len = message.len() as u32;
            let len_bytes = len.to_be_bytes();
            
            trace!("Sending command (attempt {}/{}): {:?}", attempt, max_attempts, command);
            
            // Get connection lock - we know it's Some because ensure_connected succeeded
            let mut connection_guard = self.connection.lock().await;
            let connection = connection_guard.as_mut().unwrap();
            
            // Send the length prefix
            if let Err(e) = connection.write_all(&len_bytes).await {
                warn!("Failed to send length prefix: {}", e);
                // Mark connection as broken
                *connection_guard = None;
                
                if attempt == max_attempts {
                    return Err(anyhow!("Failed to send message after {} attempts: {}", max_attempts, e));
                }
                
                // Wait before retrying
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                backoff_ms *= 2;
                continue;
            }
            
            // Send the message payload
            if let Err(e) = connection.write_all(&message).await {
                warn!("Failed to send message payload: {}", e);
                // Mark connection as broken
                *connection_guard = None;
                
                if attempt == max_attempts {
                    return Err(anyhow!("Failed to send message payload after {} attempts: {}", max_attempts, e));
                }
                
                // Wait before retrying
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                backoff_ms *= 2;
                continue;
            }
            
            // Read response length
            let mut len_buf = [0u8; 4];
            if let Err(e) = connection.read_exact(&mut len_buf).await {
                warn!("Failed to read response length: {}", e);
                // Mark connection as broken
                *connection_guard = None;
                
                if attempt == max_attempts {
                    return Err(anyhow!("Failed to read response length after {} attempts: {}", max_attempts, e));
                }
                
                // Wait before retrying
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                backoff_ms *= 2;
                continue;
            }
            
            let len = u32::from_be_bytes(len_buf) as usize;
            
            // Read response
            let mut response_buf = vec![0u8; len];
            if let Err(e) = connection.read_exact(&mut response_buf).await {
                warn!("Failed to read response payload: {}", e);
                // Mark connection as broken
                *connection_guard = None;
                
                if attempt == max_attempts {
                    return Err(anyhow!("Failed to read response payload after {} attempts: {}", max_attempts, e));
                }
                
                // Wait before retrying
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                backoff_ms *= 2;
                continue;
            }
            
            // Parse response
            let response: ManagementResponse = match serde_json::from_slice(&response_buf) {
                Ok(resp) => resp,
                Err(e) => {
                    warn!("Failed to parse response: {}", e);
                    
                    if attempt == max_attempts {
                        return Err(anyhow!("Failed to parse response after {} attempts: {}", max_attempts, e));
                    }
                    
                    // Wait before retrying
                    tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                    backoff_ms *= 2;
                    continue;
                }
            };
            
            trace!("Received response: {:?}", response);
            
            // Check for error
            if let ManagementResponse::Error { message } = &response {
                return Err(TheaterError::ServerError(message.clone()).into());
            }
            
            // Success!
            return Ok(response);
        }
        
        // This should not be reached due to the returns inside the loop
        Err(anyhow!("Failed to send command after maximum attempts"))
    }

    /// Start a heartbeat process to periodically check connection
    pub fn start_heartbeat(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let client = Arc::clone(self);
        tokio::spawn(async move {
            let interval = tokio::time::Duration::from_secs(30); // Check every 30 seconds
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                if let Err(e) = client.ping().await {
                    warn!("Theater heartbeat failed: {}. Will attempt reconnection on next request.", e);
                }
            }
        })
    }
    
    /// Simple ping to check server connection
    async fn ping(&self) -> Result<()> {
        // Use list_actors as a simple ping
        self.list_actors().await?;
        Ok(())
    }

    /// List all running actors
    pub async fn list_actors(&self) -> Result<Vec<TheaterId>> {
        let command = ManagementCommand::ListActors;
        
        let response = self.send_command(command).await?;
        
        match response {
            ManagementResponse::ActorList { actors } => Ok(actors),
            _ => Err(anyhow!("Unexpected response type: {:?}", response)),
        }
    }

    /// Start a new actor from a manifest
    pub async fn start_actor(
        &self,
        manifest: &str,
        initial_state: Option<&[u8]>,
    ) -> Result<TheaterId> {
        let initial_state_vec = initial_state.map(|s| s.to_vec());
        
        let command = ManagementCommand::StartActor {
            manifest: manifest.to_string(),
            initial_state: initial_state_vec,
        };
        
        let response = self.send_command(command).await?;
        
        match response {
            ManagementResponse::ActorStarted { id } => Ok(id),
            _ => Err(anyhow!("Unexpected response type: {:?}", response)),
        }
    }

    /// Stop a running actor
    pub async fn stop_actor(&self, actor_id: &TheaterId) -> Result<()> {
        let command = ManagementCommand::StopActor {
            id: actor_id.clone(),
        };
        
        let response = self.send_command(command).await?;
        
        match response {
            ManagementResponse::ActorStopped { id: _ } => Ok(()),
            _ => Err(anyhow!("Unexpected response type: {:?}", response)),
        }
    }

    /// Restart a running actor
    pub async fn restart_actor(&self, actor_id: &TheaterId) -> Result<()> {
        let command = ManagementCommand::RestartActor {
            id: actor_id.clone(),
        };
        
        let response = self.send_command(command).await?;
        
        match response {
            ManagementResponse::Restarted { id: _ } => Ok(()),
            _ => Err(anyhow!("Unexpected response type: {:?}", response)),
        }
    }

    /// Check if an actor exists
    pub async fn actor_exists(&self, actor_id: &TheaterId) -> Result<bool> {
        // Try to get the actor's state to determine if it exists
        match self.get_actor_state(actor_id).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get the current state of an actor
    pub async fn get_actor_state(&self, actor_id: &TheaterId) -> Result<Option<Vec<u8>>> {
        let command = ManagementCommand::GetActorState {
            id: actor_id.clone(),
        };
        
        let response = self.send_command(command).await?;
        
        match response {
            ManagementResponse::ActorState { id: _, state } => Ok(state),
            _ => Err(anyhow!("Unexpected response type: {:?}", response)),
        }
    }

    /// Get the event history for an actor
    pub async fn get_actor_events(&self, actor_id: &TheaterId) -> Result<Vec<ChainEvent>> {
        let command = ManagementCommand::GetActorEvents {
            id: actor_id.clone(),
        };
        
        let response = self.send_command(command).await?;
        
        match response {
            ManagementResponse::ActorEvents { id: _, events } => Ok(events),
            _ => Err(anyhow!("Unexpected response type: {:?}", response)),
        }
    }

    /// Send a one-way message to an actor
    pub async fn send_message(&self, actor_id: &TheaterId, data: &[u8]) -> Result<()> {
        let command = ManagementCommand::SendActorMessage {
            id: actor_id.clone(),
            data: data.to_vec(),
        };
        
        let response = self.send_command(command).await?;
        
        match response {
            ManagementResponse::SentMessage { id: _ } => Ok(()),
            _ => Err(anyhow!("Unexpected response type: {:?}", response)),
        }
    }

    /// Send a request to an actor and receive a response
    pub async fn request_message(&self, actor_id: &TheaterId, data: &[u8]) -> Result<Vec<u8>> {
        let command = ManagementCommand::RequestActorMessage {
            id: actor_id.clone(),
            data: data.to_vec(),
        };
        
        let response = self.send_command(command).await?;
        
        match response {
            ManagementResponse::RequestedMessage { id: _, message } => Ok(message),
            _ => Err(anyhow!("Unexpected response type: {:?}", response)),
        }
    }

    /// Open a channel to an actor
    pub async fn open_channel(
        &self,
        actor_id: &str,
        initial_message: Option<&[u8]>,
    ) -> Result<String> {
        // Parse actor ID string to TheaterId
        let actor_id = TheaterId::parse(actor_id)?;
        let actor_participant = ChannelParticipant::Actor(actor_id);
        let initial_data = initial_message.map(|m| m.to_vec()).unwrap_or_default();
        
        let command = ManagementCommand::OpenChannel {
            actor_id: actor_participant,
            initial_message: initial_data,
        };
        
        let response = self.send_command(command).await?;
        
        match response {
            ManagementResponse::ChannelOpened { channel_id, actor_id: _ } => Ok(channel_id),
            _ => Err(anyhow!("Unexpected response type: {:?}", response)),
        }
    }

    /// Send a message on an open channel
    pub async fn send_on_channel(&self, channel_id: &str, message: &[u8]) -> Result<()> {
        let command = ManagementCommand::SendOnChannel {
            channel_id: channel_id.to_string(),
            message: message.to_vec(),
        };
        
        let response = self.send_command(command).await?;
        
        match response {
            ManagementResponse::MessageSent { channel_id: _ } => Ok(()),
            _ => Err(anyhow!("Unexpected response type: {:?}", response)),
        }
    }

    /// Close an open channel
    pub async fn close_channel(&self, channel_id: &str) -> Result<()> {
        let command = ManagementCommand::CloseChannel {
            channel_id: channel_id.to_string(),
        };
        
        let response = self.send_command(command).await?;
        
        match response {
            ManagementResponse::ChannelClosed { channel_id: _ } => Ok(()),
            _ => Err(anyhow!("Unexpected response type: {:?}", response)),
        }
    }
}