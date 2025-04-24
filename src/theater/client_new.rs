use anyhow::{anyhow, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::trace;

use theater::id::TheaterId;
use theater::theater_server::{ManagementCommand, ManagementResponse};
use theater::messages::ChannelParticipant;
use theater::chain::ChainEvent;

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
    async fn send_command(&self, command: ManagementCommand) -> Result<ManagementResponse> {
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
        let response: ManagementResponse = serde_json::from_slice(&response_buf)?;
        trace!("Received response: {:?}", response);

        // Check for error
        if let ManagementResponse::Error { message } = &response {
            return Err(TheaterError::ServerError(message.clone()).into());
        }

        Ok(response)
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