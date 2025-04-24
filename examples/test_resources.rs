use anyhow::{anyhow, Result};
use mcp_client::resources::ResourceClient;
use std::path::PathBuf;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // Read config from the command line or use defaults
    let args: Vec<String> = std::env::args().collect();
    
    let server_url = if args.len() > 1 {
        &args[1]
    } else {
        "http://localhost:8080"
    };
    
    // Connect to the MCP server
    let client = ResourceClient::connect(server_url).await?;
    
    // List available resources
    println!("Listing available resources...");
    let resources = client.list_resources().await?;
    
    for resource in resources {
        println!("Resource: {} ({})", resource.name, resource.uri);
        
        if resource.uri.starts_with("theater://") {
            // This is a Theater resource, fetch it
            match client.get_resource(&resource.uri).await {
                Ok(content) => {
                    println!("Content: {}", content.text.unwrap_or_default());
                    
                    // If this is the actors list, we could parse it and fetch each actor
                    if resource.uri == "theater://actors" {
                        // TODO: Parse the actors list and fetch each actor
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching resource {}: {}", resource.uri, e);
                }
            }
        }
    }
    
    // Try to get a specific resource by URI
    if args.len() > 2 {
        let resource_uri = &args[2];
        
        println!("\nFetching specific resource: {}", resource_uri);
        
        match client.get_resource(resource_uri).await {
            Ok(content) => {
                println!("Content: {}", content.text.unwrap_or_default());
                
                // Save the content to a file
                let filename = resource_uri.replace("://", "_").replace("/", "_");
                let path = PathBuf::from(format!("{}.json", filename));
                
                match fs::write(&path, content.text.unwrap_or_default()).await {
                    Ok(_) => println!("Saved content to {}", path.display()),
                    Err(e) => return Err(anyhow!("Failed to save content: {}", e)),
                }
            }
            Err(e) => {
                return Err(anyhow!("Failed to fetch resource: {}", e));
            }
        }
    }
    
    Ok(())
}