use anyhow::Result;
use std::time::Duration;
use tracing::{debug, error, info, warn};

// Include generated proto code
pub mod chat {
    tonic::include_proto!("petoncle");
}

use chat::chat_service_client::ChatServiceClient;
use chat::{ChatRequest, ChatResponse};

/// gRPC client for communicating with Python agent service
pub struct AgentClient {
    client: Option<ChatServiceClient<tonic::transport::Channel>>,
    server_addr: String,
    max_retries: u32,
}

impl AgentClient {
    pub fn new(server_addr: &str) -> Self {
        Self {
            client: None,
            server_addr: server_addr.to_string(),
            max_retries: 3,  // Retry up to 3 times
        }
    }

    /// Connect to the agent service
    pub async fn connect(&mut self) -> Result<()> {
        let addr = format!("http://{}", self.server_addr);
        debug!("Connecting to gRPC service at {}", addr);

        // Create endpoint with timeout configuration
        let channel = tonic::transport::Channel::from_shared(addr)?
            .timeout(Duration::from_secs(10))  // 10s timeout for connection
            .connect_timeout(Duration::from_secs(5))  // 5s timeout for initial connect
            .connect()
            .await?;
        let client = ChatServiceClient::new(channel);
        self.client = Some(client);
        info!("Successfully connected to gRPC service");
        Ok(())
    }

    /// Send a chat message and get AI response with automatic retry
    pub async fn send_message(
        &mut self,
        message: String,
        context: Vec<String>,
    ) -> Result<ChatResponse> {
        let mut last_error = None;

        // Retry loop with exponential backoff
        for attempt in 0..=self.max_retries {
            // Ensure we're connected (will reconnect if needed)
            if self.client.is_none() {
                debug!("Not connected, attempting to connect (attempt {})", attempt + 1);
                match self.connect().await {
                    Ok(_) => {},
                    Err(e) => {
                        warn!("Connection attempt {} failed: {}", attempt + 1, e);
                        last_error = Some(e);
                        if attempt < self.max_retries {
                            // Exponential backoff: 1s, 2s, 4s
                            let backoff = Duration::from_secs(2u64.pow(attempt));
                            debug!("Retrying in {:?}", backoff);
                            tokio::time::sleep(backoff).await;
                            continue;
                        }
                        break;
                    }
                }
            }

            let mut request = tonic::Request::new(ChatRequest {
                message: message.clone(),
                context: context.clone(),
            });

            // Set timeout for this request (45 seconds to account for Mistral API timeout)
            request.set_timeout(Duration::from_secs(45));

            match self
                .client
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("Not connected"))?
                .send_message(request)
                .await
            {
                Ok(response) => {
                    debug!("Successfully received response from gRPC service");
                    return Ok(response.into_inner());
                }
                Err(e) => {
                    // Connection lost, reset client for reconnection
                    error!("gRPC request failed (attempt {}): {}", attempt + 1, e);
                    self.client = None;
                    last_error = Some(e.into());

                    if attempt < self.max_retries {
                        // Exponential backoff before retry
                        let backoff = Duration::from_secs(2u64.pow(attempt));
                        debug!("Retrying in {:?}", backoff);
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        let final_error = last_error.unwrap_or_else(|| anyhow::anyhow!("Failed to send message after retries"));
        error!("All retry attempts exhausted: {}", final_error);
        Err(final_error)
    }

    /// Check if connected to service
    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}
