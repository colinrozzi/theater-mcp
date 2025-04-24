// Original implementations
pub mod client;
pub mod types;

// New implementations using Theater types directly
pub mod client_new;
pub mod types_new;

// Tests
#[cfg(test)]
mod tests;

// Re-export important types - use the new versions
// Re-export important Theater types
pub use theater::id::TheaterId;
pub use theater::messages::ActorStatus;
pub use theater::chain::ChainEvent;

// Re-export our extension trait
pub use types_new::{TheaterError, TheaterIdExt};

// For backwards compatibility during transition
pub use client::TheaterClient;
