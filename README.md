# Theater MCP Server

A Model Context Protocol (MCP) server for interfacing with the Theater WebAssembly actor system.

## Overview

The Theater MCP Server provides a standardized interface for language models and MCP clients to interact with the Theater WebAssembly actor system. It implements the MCP protocol and exposes Theater actors, their state, and event history as resources, as well as providing tools for actor management, messaging, and channel operations.

## Features

- **Actor Resources**: Access actor lists, details, and state via resources
- **Event Resources**: Access actor event history
- **Actor Management Tools**: Start, stop, and restart actors
- **Message Tools**: Send one-way messages and request-response messages
- **Channel Tools**: Open, send on, and close communication channels

## Prerequisites

- Rust 1.70 or newer
- A running Theater server (typically on localhost:9000)

## Installation

Clone the repository and build the server:

```bash
# Clone the repository
git clone https://github.com/yourusername/theater-mcp-server.git
cd theater-mcp-server

# Build the server
cargo build
```

## Usage

Start the Theater MCP server, pointing it to your Theater server instance:

```bash
cargo run -- --theater-address 127.0.0.1:9000
```

Additional command line options:

- `--log-level <LEVEL>`: Sets the log level (trace, debug, info, warn, error)
- `--log-file <FILE>`: Logs to a file instead of stderr

## Client Example

The `examples/simple_client.rs` file demonstrates how to use a basic MCP client to interact with the Theater MCP server:

```bash
# Run the example client
cargo run --example simple_client

# To test sending a message to an actor, set the actor ID
THEATER_ACTOR_ID=your-actor-id cargo run --example simple_client
```

## MCP Resources

The server exposes the following resources:

- `theater://actors`: List of all running actors
- `theater://actor/{actor_id}`: Detailed information about a specific actor
- `theater://actor/{actor_id}/state`: Current state of a specific actor
- `theater://events/{actor_id}`: Event history for a specific actor

## MCP Tools

The server provides the following tools:

- `start_actor`: Start a new actor from a manifest
- `stop_actor`: Stop a running actor
- `restart_actor`: Restart a running actor
- `send_message`: Send a one-way message to an actor
- `request_message`: Send a request to an actor and receive a response
- `open_channel`: Open a communication channel to an actor
- `send_on_channel`: Send a message on an open channel
- `close_channel`: Close an open channel

## License

MIT
