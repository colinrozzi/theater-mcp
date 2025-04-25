# Change Request: Theater Connection Resilience Enhancement

## Summary
Implement a reconnection mechanism for the Theater server connection to improve resilience when the connection is dropped unexpectedly. The MCP server should automatically attempt to reconnect to the Theater server when processing tool or resource requests after a connection failure.

## Background
Currently, if the connection to the Theater server is dropped, the MCP server fails and requires manual restart. Since intermittent connection issues are expected in production environments, we need to enhance the server to handle connection drops gracefully and maintain service availability.

## Proposed Changes

### 1. TheaterClient Enhancements

1. Modify the TheaterClient to track connection state:
   ```rust
   pub struct TheaterClient {
       connection: Arc<Mutex<Option<TcpStream>>>,
       address: SocketAddr,
       is_connecting: Arc<AtomicBool>,
   }
   ```

2. Implement a connection check method:
   ```rust
   async fn ensure_connected(&self) -> Result<()> {
       let mut connection_guard = self.connection.lock().await;
       
       // If we already have a connection, check if it's still valid
       if let Some(conn) = &mut *connection_guard {
           // Try a small write to test connection
           if let Err(_) = conn.write_all(&[0; 0]).await {
               // Connection is broken, clear it
               *connection_guard = None;
           }
       }
       
       // If connection is None, create a new connection
       if connection_guard.is_none() {
           if !self.is_connecting.swap(true, Ordering::SeqCst) {
               // Try to establish a new connection
               match TcpStream::connect(self.address).await {
                   Ok(stream) => {
                       *connection_guard = Some(stream);
                       tracing::info!("Successfully reconnected to Theater server at {}", self.address);
                   },
                   Err(e) => {
                       tracing::error!("Failed to reconnect to Theater server: {}", e);
                       self.is_connecting.store(false, Ordering::SeqCst);
                       return Err(anyhow!("Failed to connect to Theater server: {}", e));
                   }
               }
               self.is_connecting.store(false, Ordering::SeqCst);
           } else {
               // Another thread is already trying to connect
               return Err(anyhow!("Connection attempt already in progress"));
           }
       }
       
       Ok(())
   }
   ```

3. Update the send_command method to use this check:
   ```rust
   async fn send_command(&self, command: Value) -> Result<Value> {
       // Try to ensure connection up to 3 times
       let mut attempts = 0;
       let max_attempts = 3;
       
       while attempts < max_attempts {
           if let Err(e) = self.ensure_connected().await {
               attempts += 1;
               if attempts >= max_attempts {
                   return Err(anyhow!("Failed to establish connection after {} attempts: {}", max_attempts, e));
               }
               // Wait before retrying
               tokio::time::sleep(tokio::time::Duration::from_millis(500 * attempts as u64)).await;
               continue;
           }
           
           // We have a connection, proceed with command
           let mut connection_guard = self.connection.lock().await;
           let connection = connection_guard.as_mut().unwrap();
           
           // Create message frame
           let message = serde_json::to_vec(&command)?;
           let len = message.len() as u32;
           let len_bytes = len.to_be_bytes();
           
           // Send length prefix and message
           match connection.write_all(&len_bytes).await {
               Ok(_) => (),
               Err(e) => {
                   // Mark connection as broken
                   *connection_guard = None;
                   attempts += 1;
                   if attempts >= max_attempts {
                       return Err(anyhow!("Failed to send message after {} attempts: {}", max_attempts, e));
                   }
                   // Wait before retrying
                   tokio::time::sleep(tokio::time::Duration::from_millis(500 * attempts as u64)).await;
                   continue;
               }
           }
           
           // Attempt to write the message payload
           if let Err(e) = connection.write_all(&message).await {
               // Mark connection as broken
               *connection_guard = None;
               attempts += 1;
               if attempts >= max_attempts {
                   return Err(anyhow!("Failed to send message payload after {} attempts: {}", max_attempts, e));
               }
               continue;
           }
           
           // Read response length
           let mut len_buf = [0u8; 4];
           match connection.read_exact(&mut len_buf).await {
               Ok(_) => (),
               Err(e) => {
                   // Mark connection as broken
                   *connection_guard = None;
                   attempts += 1;
                   if attempts >= max_attempts {
                       return Err(anyhow!("Failed to read response length after {} attempts: {}", max_attempts, e));
                   }
                   continue;
               }
           }
           
           let len = u32::from_be_bytes(len_buf) as usize;
           let mut buffer = vec![0u8; len];
           
           match connection.read_exact(&mut buffer).await {
               Ok(_) => {
                   // Successfully read response
                   let response: Value = serde_json::from_slice(&buffer)?;
                   return Ok(response);
               },
               Err(e) => {
                   // Mark connection as broken
                   *connection_guard = None;
                   attempts += 1;
                   if attempts >= max_attempts {
                       return Err(anyhow!("Failed to read response payload after {} attempts: {}", max_attempts, e));
                   }
                   continue;
               }
           }
       }
       
       Err(anyhow!("Failed to send command after maximum attempts"))
   }
   ```

### 2. Error Handling Improvements

1. Update tool handlers to properly handle and report Theater connection issues:
   ```rust
   // In each tool implementation
   pub async fn execute(&self, params: Value) -> Result<Value> {
       match self.theater_client.some_operation(params).await {
           Ok(result) => Ok(result),
           Err(e) => {
               if e.to_string().contains("connection") {
                   tracing::warn!("Theater connection issue during tool execution: {}", e);
                   Err(anyhow!("Theater server connection issue: {}. The server will attempt to reconnect on the next request.", e))
               } else {
                   Err(e)
               }
           }
       }
   }
   ```

2. Similarly update resource handlers to handle connection issues gracefully.

### 3. Heartbeat Mechanism (Optional Enhancement)

1. Implement a background task to periodically check the Theater connection health:
   ```rust
   pub fn start_heartbeat(&self) -> JoinHandle<()> {
       let client = self.clone();
       tokio::spawn(async move {
           let interval = tokio::time::Duration::from_secs(30); // Check every 30 seconds
           let mut interval_timer = tokio::time::interval(interval);
           
           loop {
               interval_timer.tick().await;
               if let Err(e) = client.ping().await {
                   tracing::warn!("Theater heartbeat failed: {}. Will attempt reconnection on next request.", e);
               }
           }
       })
   }
   
   async fn ping(&self) -> Result<()> {
       // Simple ping command to check connection
       self.send_command(json!({"type": "ping"})).await?;
       Ok(())
   }
   ```

## Implementation Strategy

1. First, implement the basic reconnection logic in TheaterClient
2. Update the tool and resource handlers to use the enhanced client
3. Add error reporting and logging for connection issues
4. Test with scenarios involving connection drops
5. (Optional) Implement the heartbeat mechanism as a later enhancement

## Testing Plan

1. **Unit Tests**:
   - Test the reconnection logic with mocked connections
   - Ensure proper error handling in reconnection scenarios

2. **Integration Tests**:
   - Force connection drops and verify automatic reconnection
   - Test concurrent requests during reconnection
   - Verify that tools and resources work after reconnection

3. **Stress Tests**:
   - Simulate frequent connection drops under load
   - Verify performance under reconnection conditions

## Risks and Mitigations

**Risk**: Concurrent reconnection attempts could lead to race conditions.  
**Mitigation**: Use atomic flags and locks to ensure only one thread attempts reconnection at a time.

**Risk**: Repeated reconnection attempts could overload the Theater server.  
**Mitigation**: Implement exponential backoff for reconnection attempts and limit maximum tries.

**Risk**: Client requests might time out during reconnection.  
**Mitigation**: Properly communicate connection status in error messages to clients.

## Estimated Work

- Engineering effort: 1-2 days
- Testing: 1 day
- Documentation updates: 0.5 day

## Alternatives Considered

1. Implementing a fully reactive connection system based on the Theater subscription model
2. Using a connection pool to maintain multiple connections to the Theater server
3. Complete server restart on connection failure

These alternatives were rejected due to higher implementation complexity and potentially less reliable behavior compared to the proposed approach.
