use anyhow::Result;
use clap::Parser;
use mcp_server::transport::stdio::StdioTransport;
use std::net::SocketAddr;
use std::path::PathBuf;
use theater_mcp_server::server::TheaterMcpServer;
use tracing::{info, Level};
use tracing_appender;
use tracing_subscriber::FmtSubscriber;

/// MCP server for interfacing with the Theater WebAssembly actor system
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Theater server address
    #[arg(short, long, default_value = "127.0.0.1:9000")]
    theater_address: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: Level,

    /// Log to file instead of stderr
    #[arg(
        long,
        default_value = "/Users/colinrozzi/work/mcp-servers/theater-mcp-server/theater_mcp.log"
    )]
    log_file: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(args.log_level)
        .with_writer(tracing_appender::rolling::never(
            args.log_file,
            "theater_mcp.log",
        ))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    // Parse Theater server address
    let theater_addr: SocketAddr = args.theater_address.parse()?;
    info!("Connecting to Theater server at {}", theater_addr);

    // Create and run the Theater MCP server
    let server = TheaterMcpServer::new(theater_addr, StdioTransport::new()).await?;
    info!("Theater MCP server started");

    // Run the server (blocks until completion)
    server.run().await?;

    Ok(())
}
