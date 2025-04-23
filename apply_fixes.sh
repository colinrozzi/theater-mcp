#!/bin/bash

# Apply the fixes to the tool implementations
cp src/tools/actor.rs.fix src/tools/actor.rs
cp src/tools/message.rs.fix src/tools/message.rs
cp src/tools/channel.rs.fix src/tools/channel.rs
cp src/tools/mod.rs.fix src/tools/mod.rs

# Compile the project to check for errors
echo "Running cargo build to check for errors..."
cargo build
