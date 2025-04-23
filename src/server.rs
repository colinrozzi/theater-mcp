use anyhow::Result;
use mcp_server::{
    resources::ResourceManager, server::ServerBuilder, tools::ToolManager, transport::Transport,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use crate::resources::{ActorResources, EventResources};
use crate::theater::client::TheaterClient;
use crate::tools::{ActorTools, ChannelTools, MessageTools};

/// MCP server that interfaces with the Theater actor system
pub struct TheaterMcpServer {
    server: mcp_server::server::Server,
}

impl TheaterMcpServer {
    /// Create a new Theater MCP server
    pub async fn new<T: Transport + 'static>(
        theater_addr: SocketAddr,
        transport: T,
    ) -> Result<Self> {
        // Connect to the Theater server
        let theater_client = Arc::new(TheaterClient::connect(theater_addr).await?);
        info!("Connected to Theater server at {}", theater_addr);

        // Create shared managers
        let tool_manager = Arc::new(ToolManager::new());
        let resource_manager = Arc::new(ResourceManager::new());

        // Create and register resources
        let actor_resources = Arc::new(ActorResources::new(theater_client.clone()));
        let event_resources = Arc::new(EventResources::new(theater_client.clone()));

        actor_resources.register_resources(&resource_manager);
        event_resources.register_resources(&resource_manager);

        // Create and register tools
        let actor_tools = Arc::new(ActorTools::new(theater_client.clone()));
        let message_tools = Arc::new(MessageTools::new(theater_client.clone()));
        let channel_tools = Arc::new(ChannelTools::new(theater_client.clone()));

        actor_tools.register_tools(&tool_manager);
        message_tools.register_tools(&tool_manager);
        channel_tools.register_tools(&tool_manager);

        // Create the MCP server
        let server = ServerBuilder::new("theater-mcp", "0.1.0")
            .with_transport(transport)
            .with_tool_manager(tool_manager)
            .with_resource_manager(resource_manager)
            .build()?;

        info!("Theater MCP server created");
        Ok(Self { server })
    }

    /// Run the server (blocking)
    pub async fn run(self) -> Result<()> {
        info!("Starting Theater MCP server");
        self.server.run().await
    }
}
