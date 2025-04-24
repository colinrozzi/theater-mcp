# Theater MCP Server Fixes

## 1. Fix Types Implementation

First, let's fix the `TheaterIdExt` trait implementation:

```rust
// In types_new.rs
pub trait TheaterIdExt {
    fn as_string(&self) -> String;
    fn from_str(s: &str) -> Result<TheaterId, anyhow::Error>;
}

impl TheaterIdExt for TheaterId {
    fn as_string(&self) -> String {
        self.to_string() // Use ToString trait
    }
    
    fn from_str(s: &str) -> Result<TheaterId, anyhow::Error> {
        TheaterId::parse(s).map_err(|e| anyhow::anyhow!("Invalid Theater ID: {}", e))
    }
}
```

## 2. Fix ToolContent Usage

The `ToolContent` enum doesn't have a `Json` variant. We need to check the actual API and use the correct variant.
Inspecting the MCP protocol source, we should use:

```rust
ToolContent::Text {
    text: serde_json::to_string(&response_json)?
}
```

## 3. Fix TheaterClient Implementation

We need to update the client's `send_command` method to handle both the new `ManagementCommand` type and JSON values:

```rust
// Overload send_command to handle both types
async fn send_command_json(&self, command: Value) -> Result<Value> {
    // Implementation using JSON values
}

async fn send_command(&self, command: ManagementCommand) -> Result<ManagementResponse> {
    // Implementation using typed commands
}
```

## 4. Update Importing Strategy

We need to update all import statements to point to the correct types:

```rust
use crate::theater::TheaterIdExt;
use theater::id::TheaterId;
```

## 5. Fix Tool Registration

The tool registration method signature is different than what we're using. We need to update all tool registration calls to match the expected signature:

```rust
// Check mcp-server implementation for the correct signature
tool_manager.register_tool(
    mcp_protocol::types::tool::Tool {
        name: "start_actor".to_string(),
        description: "Start a new actor from a manifest".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "manifest": {
                    "type": "string",
                    "description": "Path to the actor manifest or manifest content"
                },
                "initial_state": {
                    "type": "object",
                    "description": "Optional initial state for the actor"
                }
            },
            "required": ["manifest"]
        }),
    },
    move |args| {
        let tools_self = self.clone();
        Box::pin(async move {
            tools_self.start_actor(args).await
        })
    },
);
```

## 6. Convert Client Implementation Strategy

Rather than completely replacing the existing implementation, let's enhance it:

1. Keep the original client implementation but add methods that use Theater types
2. Add conversion methods between string IDs and TheaterId
3. Fix the JSON serialization in state handling

## Implementation Order

1. Fix the types module first
2. Fix the client implementation 
3. Fix resources implementation
4. Fix tools implementation
5. Gradually transition to using Theater types directly

This approach will be less disruptive and allow for incremental adoption of Theater types.
