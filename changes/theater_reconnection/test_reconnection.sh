#!/bin/bash

# Test script for Theater MCP server reconnection

echo "Building Theater MCP server..."
cd /Users/colinrozzi/work/mcp-servers/theater-mcp-server
cargo build

echo "Starting Theater server..."
# Adjust this path to your Theater server executable
THEATER_SERVER_PATH="/Users/colinrozzi/work/theater/target/debug/theater-server"
$THEATER_SERVER_PATH &
THEATER_SERVER_PID=$!

# Wait for Theater server to start
sleep 2

echo "Starting Theater MCP server..."
./target/debug/theater-mcp-server --theater-address 127.0.0.1:9000 &
MCP_SERVER_PID=$!

# Wait for MCP server to start
sleep 2

echo "Testing basic connectivity..."
# Run a simple client to test initial connectivity
cargo run --example simple_client

# Wait a bit
sleep 3

echo "Simulating Theater server crash..."
# Kill the Theater server to simulate a crash
kill $THEATER_SERVER_PID

# Wait a bit for the MCP server to detect the disconnection
sleep 2

echo "Restarting Theater server..."
# Restart the Theater server
$THEATER_SERVER_PATH &
THEATER_SERVER_PID=$!

# Wait for Theater server to restart
sleep 2

echo "Testing reconnection..."
# Run the client again to test reconnection
cargo run --example simple_client

echo "Testing completed. Cleaning up..."
# Clean up processes
kill $MCP_SERVER_PID
kill $THEATER_SERVER_PID

echo "Done!"
