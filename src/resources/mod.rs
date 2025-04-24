// Original implementations
mod actors;
mod events;

// New implementations using Theater types directly
mod actors_new;
mod events_new;

// Re-export important types - use the new versions
pub use actors_new::ActorResources;
pub use events_new::EventResources;
