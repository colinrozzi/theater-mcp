use anyhow::Result;
use mcp_protocol::types::tool::{Tool, ToolCallResult};
use mcp_server::tools::ToolManager;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::runtime::Handle;

/// Type for async tool handlers
pub type AsyncToolHandler = Arc<dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = Result<ToolCallResult>> + Send>> + Send + Sync>;

/// Extension trait to add async support to ToolManager
pub trait ToolManagerExt {
    /// Register an async tool handler
    fn register_async_tool<F, Fut>(&self, tool: Tool, handler: F)
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ToolCallResult>> + Send + 'static;
}

impl ToolManagerExt for ToolManager {
    fn register_async_tool<F, Fut>(&self, tool: Tool, handler: F)
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ToolCallResult>> + Send + 'static,
    {
        // Convert the async handler to a sync handler that spawns the async task
        // This avoids the need for block_on which causes runtime panics
        let sync_handler = move |args: serde_json::Value| -> Result<ToolCallResult> {
            // Create a oneshot channel to receive the result
            let (tx, rx) = tokio::sync::oneshot::channel();
            
            // Get the current tokio runtime handle
            let handle = Handle::current();
            
            // Spawn the async handler on the runtime
            handle.spawn(async move {
                let result = handler(args).await;
                let _ = tx.send(result);
            });
            
            // Wait for the result (this doesn't block the runtime)
            match rx.blocking_recv() {
                Ok(result) => result,
                Err(e) => Err(anyhow::anyhow!("Failed to execute async tool: {}", e)),
            }
        };
        
        // Register the sync wrapper
        self.register_tool(tool, sync_handler);
    }
}
