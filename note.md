2025-04-23T18:11:33.805Z [theater] [info] Message from client: {"method":"prompts/list","params":{},"jsonrpc":"2.0","id":16}
2025-04-23T18:11:33.805Z [theater] [info] Message from server: {"jsonrpc":"2.0","id":16,"result":{"nextCursor":"","prompts":[]}}
2025-04-23T18:11:39.557Z [theater] [info] Message from client: {"method":"resources/read","params":{"uri":"theater://actors"},"jsonrpc":"2.0","id":17}

thread 'main' panicked at /Users/colinrozzi/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.44.2/src/runtime/scheduler/multi_thread/mod.rs:86:9:
Cannot start a runtime from within a runtime. This happens because a function (like `block_on`) attempted to block the current thread while the thread is being used to drive asynchronous tasks.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
2025-04-23T18:11:54.558Z [theater] [info] Message from client: {"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":17,"reason":"Error: MCP error -32001: Request timed out"}}
2025-04-23T18:11:54.560Z [theater] [info] Server transport closed
2025-04-23T18:11:54.560Z [theater] [info] Client transport closed
2025-04-23T18:11:54.561Z [theater] [info] Server transport closed unexpectedly, this is likely due to the process exiting early. If you are developing this MCP server you can add output to stderr (i.e. `console.error('...')` in JavaScript, `print('...', file=sys.stderr)` in python) and it will appear in this log.
