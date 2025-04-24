// Original implementations
pub mod client;
pub mod types;

// New implementations using Theater types directly
pub mod client_new;
pub mod types_new;

// Re-export important types - use the new versions
pub use client_new::TheaterClient;
pub use types_new::{TheaterError, TheaterIdExt};
// Re-export important Theater types
pub use theater::id::TheaterId;
pub use theater::messages::ActorStatus;
pub use theater::chain::ChainEvent;
