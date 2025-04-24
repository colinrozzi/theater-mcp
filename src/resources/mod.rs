// Original implementations
mod actors;
mod events;

// New implementations using Theater types directly
mod actors_new;
mod events_new;

// Use the original implementations until the new ones are fully tested
pub use actors::ActorResources;
pub use events::EventResources;

// Comment these out for now until we're ready to switch
// pub use actors_new::ActorResources;
// pub use events_new::EventResources;
