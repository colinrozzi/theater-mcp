// Original implementations
mod actor;
mod message;
mod channel;
mod utils;

// New implementations using Theater types directly
mod actor_new;
mod message_new;
mod channel_new;

// Re-export important types - use the new versions
pub use actor_new::ActorTools;
pub use message_new::MessageTools;
pub use channel_new::ChannelTools;
pub use utils::register_async_tool;
