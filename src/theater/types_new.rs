use theater::id::TheaterId;
use theater::messages::ActorStatus as TheaterActorStatus;
use theater::chain::ChainEvent as TheaterChainEvent;
use thiserror::Error;

/// Custom error types for Theater client interactions
#[derive(Error, Debug)]
pub enum TheaterError {
    /// Error from the Theater server
    #[error("Theater server error: {0}")]
    ServerError(String),
    
    /// Connection error
    #[error("Theater connection error: {0}")]
    ConnectionError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Actor not found
    #[error("Actor not found: {0}")]
    ActorNotFound(String),
    
    /// Channel not found
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),
}

/// Actor status (re-exported from Theater)
pub type ActorStatus = TheaterActorStatus;

/// Theater event (re-exported from Theater)
pub type ChainEvent = TheaterChainEvent;

/// Types for converting between string IDs and Theater IDs
pub trait TheaterIdExt {
    fn to_string(&self) -> String;
    fn from_string(s: &str) -> Result<TheaterId, anyhow::Error>;
}

impl TheaterIdExt for TheaterId {
    fn to_string(&self) -> String {
        self.to_string()
    }
    
    fn from_string(s: &str) -> Result<TheaterId, anyhow::Error> {
        TheaterId::try_from(s).map_err(|e| anyhow::anyhow!("Invalid Theater ID: {}", e))
    }
}
