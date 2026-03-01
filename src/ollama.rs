use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};

#[derive(Clone)]
pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: Message,
    #[serde(default)]
    done: bool,
}

#[derive(Debug, Serialize)]
struct EmbedRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    name: String,
}

impl OllamaClient {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { base_url, client }
    }
    
    pub async fn check_connection(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                error!("Ollama connection check failed: {}", e);
                Ok(false)
            }
        }
    }
    
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to list models")?;
        
        let models_response: ModelsResponse = response
            .json()
            .await
            .context("Failed to parse models response")?;
        
        Ok(models_response.models.into_iter().map(|m| m.name).collect())
    }
    
    pub async fn chat(
        &self,
        model: &str,
        messages: Vec<Message>,
        json_format: bool,
    ) -> Result<(String, Duration)> {
        let start = Instant::now();
        let url = format!("{}/api/chat", self.base_url);
        
        let request = ChatRequest {
            model: model.to_string(),
            messages,
            stream: false,
            format: if json_format { Some("json".to_string()) } else { None },
        };
        
        debug!("Sending chat request to Ollama: model={}", model);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send chat request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama chat request failed: {} - {}", status, body);
        }
        
        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse chat response")?;
        
        let duration = start.elapsed();
        debug!("Chat request completed in {:?}", duration);
        
        Ok((chat_response.message.content, duration))
    }
    
    pub async fn embed(&self, model: &str, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let url = format!("{}/api/embed", self.base_url);
        
        let request = EmbedRequest {
            model: model.to_string(),
            input: texts,
        };
        
        debug!("Sending embedding request to Ollama: model={}", model);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send embedding request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama embed request failed: {} - {}", status, body);
        }
        
        let embed_response: EmbedResponse = response
            .json()
            .await
            .context("Failed to parse embedding response")?;
        
        Ok(embed_response.embeddings)
    }
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }
    
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }
    
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}
