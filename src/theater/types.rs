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

/// Actor status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorStatus {
    Running,
    Stopped,
    Failed,
}

/// Theater event types 
#[derive(Debug, Clone)]
pub enum EventType {
    StateChanged,
    MessageReceived,
    MessageSent,
    ChannelOpened,
    ChannelClosed,
    ActorStarted,
    ActorStopped,
    ActorFailed,
    Custom(String),
}

/// Theater event
#[derive(Debug, Clone)]
pub struct Event {
    pub event_id: String,
    pub actor_id: String,
    pub timestamp: String,
    pub event_type: EventType,
    pub data: Option<serde_json::Value>,
}
