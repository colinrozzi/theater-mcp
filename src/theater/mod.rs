// Original implementations
pub mod client;
pub mod types;

// Tests
#[cfg(test)]
mod tests;

// Re-export important types - use the new versions
// Re-export important Theater types
pub use theater::chain::ChainEvent;
pub use theater::id::TheaterId;
pub use theater::messages::ActorStatus;

// Re-export our extension trait
pub use types::{TheaterError, TheaterIdExt};

// For backwards compatibility during transition
pub use client::TheaterClient;
