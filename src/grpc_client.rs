use anyhow::Result;

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
}

impl AgentClient {
    pub fn new(server_addr: &str) -> Self {
        Self {
            client: None,
            server_addr: server_addr.to_string(),
        }
    }

    /// Connect to the agent service
    pub async fn connect(&mut self) -> Result<()> {
        let addr = format!("http://{}", self.server_addr);
        let client = ChatServiceClient::connect(addr).await?;
        self.client = Some(client);
        Ok(())
    }

    /// Send a chat message and get AI response
    pub async fn send_message(
        &mut self,
        message: String,
        context: Vec<String>,
    ) -> Result<ChatResponse> {
        // Ensure we're connected
        if self.client.is_none() {
            self.connect().await?;
        }

        let request = tonic::Request::new(ChatRequest { message, context });

        let response = self
            .client
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Not connected"))?
            .send_message(request)
            .await?;

        Ok(response.into_inner())
    }

    /// Check if connected to service
    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}
