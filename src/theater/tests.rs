#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::net::SocketAddr;
    use tokio::test;
    
    use crate::theater::client as original_client;
    use crate::theater::client_new as new_client;
    
    // Test that both client implementations can connect to a Theater server
    #[test]
    async fn test_client_connect() -> Result<()> {
        // This test doesn't actually connect to a server, just verifies the function exists
        let addr = "127.0.0.1:9000".parse::<SocketAddr>()?;
        
        // These will fail since no server is running, but we just want to check the API
        let original_result = original_client::TheaterClient::connect(addr.clone()).await;
        let new_result = new_client::TheaterClient::connect(addr.clone()).await;
        
        assert!(original_result.is_err());
        assert!(new_result.is_err());
        
        // Check that both implementations return the same error message
        let original_error = original_result.unwrap_err().to_string();
        let new_error = new_result.unwrap_err().to_string();
        
        // The error messages should both be about connection failure
        assert!(original_error.contains("connect"), "Original error: {}", original_error);
        assert!(new_error.contains("connect"), "New error: {}", new_error);
        
        Ok(())
    }
    
    // Mock tests for client methods
    // In a real implementation, we would use a mock server to test these properly
    
    // Test plan for comparing implementations:
    // 1. Create a mock Theater server that returns predefined responses
    // 2. Create clients for both implementations
    // 3. Perform the same operations with both clients
    // 4. Compare the results to ensure they match
    
    /*
    Example of what a full test might look like with mocking:
    
    #[test]
    async fn test_list_actors() -> Result<()> {
        // Setup mock server
        let mock_server = MockServer::start().await;
        
        // Configure mock response
        mock_server.expect(
            Method::GET, 
            "/actors"
        ).respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({
                    "actors": [
                        {"id": "actor-1", "status": "running"},
                        {"id": "actor-2", "status": "running"}
                    ]
                }))
        );
        
        let addr = mock_server.address();
        
        // Create clients
        let original_client = original_client::TheaterClient::connect(addr).await?;
        let new_client = new_client::TheaterClient::connect(addr).await?;
        
        // Call list_actors on both
        let original_result = original_client.list_actors().await?;
        let new_result = new_client.list_actors().await?;
        
        // Compare results
        assert_eq!(original_result.len(), new_result.len());
        assert_eq!(original_result.len(), 2);
        
        // Check the actor IDs
        assert!(original_result.contains(&"actor-1".to_string()));
        assert!(original_result.contains(&"actor-2".to_string()));
        
        // For the new client, the result type is different (TheaterId instead of String)
        // So we would need to convert to string for comparison
        let new_result_strings: Vec<String> = new_result.iter()
            .map(|id| id.to_string())
            .collect();
            
        assert!(new_result_strings.contains(&"actor-1".to_string()));
        assert!(new_result_strings.contains(&"actor-2".to_string()));
        
        Ok(())
    }
    */
}