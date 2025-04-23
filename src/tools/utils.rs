use anyhow::Result;
use mcp_protocol::types::tool::{Tool, ToolCallResult};
use mcp_server::tools::ToolManager;
use std::future::Future;
use std::sync::Arc;
use tokio::runtime::Handle;

/// Register an async tool with the tool manager
pub fn register_async_tool<F, Fut>(
    tool_manager: &Arc<ToolManager>,
    tool: Tool,
    handler: F,
)
where
    F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<ToolCallResult>> + Send + 'static,
{
    // Clone the handler to an Arc
    let handler = Arc::new(handler);
    
    // Create a sync wrapper that will execute the async handler
    let sync_handler = move |args: serde_json::Value| -> Result<ToolCallResult> {
        let handler = handler.clone();
        let args = args.clone(); // Clone args to avoid borrowing issues
        
        // Try to get the current runtime handle
        if let Ok(handle) = Handle::try_current() {
            // We're in a tokio runtime, use block_in_place
            tokio::task::block_in_place(move || {
                // Run the async handler and wait for the result
                handle.block_on(async move {
                    handler(args).await
                })
            })
        } else {
            // No runtime available, create a new one
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
                
            // Run the async handler and wait for the result
            rt.block_on(async move {
                handler(args).await
            })
        }
    };
    
    // Register the sync wrapper with the tool manager
    tool_manager.register_tool(tool, sync_handler);
}
