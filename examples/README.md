# Theater MCP Server Examples

This directory contains example clients that demonstrate how to use the Theater MCP Server.

## Hello World Actor Example

The `hello_world_client.rs` example demonstrates a complete workflow for working with the hello-world-actor:

1. Start the actor using its manifest
2. Send one-way messages to the actor
3. Make request-response interactions with the actor
4. Open, use, and close a channel
5. View the actor's event history
6. Stop the actor when finished

### Prerequisites

To run the hello world example:

1. Make sure the Theater server is running on `localhost:9000`
2. Ensure the hello-world-actor is available at `/Users/colinrozzi/work/actors/hello-world-actor`

### Running the Example

Use the provided script to run the example:

```bash
# From the project root directory
./run_hello_world_test.sh
```

Or run it manually:

```bash
# Build the example
cargo build --example hello_world_client

# Run the example
cargo run --example hello_world_client
```

## Simple Client Example

The `simple_client.rs` example demonstrates basic MCP protocol communication:

1. Connect to the Theater MCP server
2. Initialize the MCP connection
3. List available resources and tools
4. Optionally send a message to an actor (if THEATER_ACTOR_ID is set)

### Running the Simple Client

```bash
# Set an actor ID (optional)
export THEATER_ACTOR_ID=<actor-id>

# Run the example
cargo run --example simple_client
```

## Resource Client Example

The `resource_client.rs` example focuses on working with MCP resources:

1. Connect to the Theater MCP server
2. List available resources
3. Get specific resources by URI
4. Monitor resource changes

### Running the Resource Client

```bash
cargo run --example resource_client
```
