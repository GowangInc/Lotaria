use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Vision service trait
#[async_trait::async_trait]
pub trait VisionService: Send + Sync {
    async fn analyze(&self, image_base64: &str, prompt: &str) -> Result<String>;
}

/// Gemini vision service
pub struct GeminiVisionService {
    api_key: String,
    model: String,
    client: Client,
}

impl GeminiVisionService {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: Client::new(),
        }
    }

    fn model_name(&self) -> String {
        // Convert model ID to Gemini format
        if self.model.starts_with("gemini-") {
            self.model.clone()
        } else {
            format!("gemini-{}", self.model)
        }
    }
}

#[async_trait::async_trait]
impl VisionService for GeminiVisionService {
    async fn analyze(&self, image_base64: &str, prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model_name(),
            self.api_key
        );

        let body = json!({
            "contents": [{
                "parts": [
                    {
                        "text": prompt
                    },
                    {
                        "inline_data": {
                            "mime_type": "image/png",
                            "data": image_base64
                        }
                    }
                ]
            }],
            "generationConfig": {
                "maxOutputTokens": 256,
                "temperature": 0.7
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            if error_text.contains("429") || error_text.contains("rate limit") {
                return Err(anyhow!("Rate limit exceeded. Please try again later."));
            }
            return Err(anyhow!("Gemini API error: {}", error_text));
        }

        let gemini_response: GeminiResponse = response.json().await?;
        
        let text = gemini_response
            .candidates
            .get(0)
            .and_then(|c| c.content.parts.get(0))
            .map(|p| p.text.clone())
            .unwrap_or_default();

        Ok(text)
    }
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize, Clone)]
struct GeminiPart {
    text: String,
}

/// OpenAI-compatible vision service (works with OpenAI, Groq, etc.)
pub struct OpenAIVisionService {
    api_key: String,
    model: String,
    base_url: String,
    client: Client,
}

impl OpenAIVisionService {
    pub fn new(api_key: String, model: String, provider: &str) -> Self {
        let base_url = match provider {
            "openai" => "https://api.openai.com/v1".to_string(),
            "groq" => "https://api.groq.com/openai/v1".to_string(),
            "anthropic" => "https://api.anthropic.com/v1".to_string(),
            "deepseek" => "https://api.deepseek.com/v1".to_string(),
            _ => "https://api.openai.com/v1".to_string(),
        };

        Self {
            api_key,
            model,
            base_url,
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl VisionService for OpenAIVisionService {
    async fn analyze(&self, image_base64: &str, prompt: &str) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": prompt
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:image/png;base64,{}", image_base64)
                            }
                        }
                    ]
                }
            ],
            "max_tokens": 256,
            "temperature": 0.7
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            if error_text.contains("429") || error_text.contains("rate limit") {
                return Err(anyhow!("Rate limit exceeded. Please try again later."));
            }
            return Err(anyhow!("API error: {}", error_text));
        }

        let openai_response: OpenAIResponse = response.json().await?;
        
        let text = openai_response
            .choices
            .get(0)
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(text)
    }
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize, Clone)]
struct OpenAIMessage {
    content: String,
}

/// Factory function to create the appropriate vision service
pub fn create_vision_service(provider: &str, api_key: String, model: String) -> Box<dyn VisionService> {
    match provider {
        "gemini" => Box::new(GeminiVisionService::new(api_key, model)),
        _ => Box::new(OpenAIVisionService::new(api_key, model, provider)),
    }
}

