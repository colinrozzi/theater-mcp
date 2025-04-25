#!/bin/bash

# Check if Theater server is running
if ! nc -z localhost 9000 2>/dev/null; then
  echo "Error: Theater server not running on localhost:9000"
  echo "Please start the Theater server before running this test"
  exit 1
fi

echo "Building the hello-world client..."
cargo build --example hello_world_client

echo "Running the hello-world client test..."
RUST_BACKTRACE=1 cargo run --example hello_world_client
