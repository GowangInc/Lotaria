use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;
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

        tracing::info!("Vision API call - model: {}, prompt_len: {}, image_len: {}", 
            self.model_name(), prompt.len(), image_base64.len());

        // Build parts dynamically - only include image if provided
        let mut parts = vec![json!({"text": prompt})];
        if !image_base64.is_empty() {
            parts.push(json!({
                "inline_data": {
                    "mime_type": "image/png",
                    "data": image_base64
                }
            }));
        }

        let body = json!({
            "contents": [{
                "parts": parts
            }],
            "generationConfig": {
                "maxOutputTokens": 2048,
                "temperature": 0.7
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;
        
        tracing::info!("Vision API response status: {}", status);

        if !status.is_success() {
            tracing::error!("Vision API error: {}", response_text);
            if response_text.contains("429") || response_text.contains("rate limit") {
                return Err(anyhow!("Rate limit exceeded. Please try again later."));
            }
            return Err(anyhow!("Gemini API error: {}", response_text));
        }

        let gemini_response: GeminiResponse = match serde_json::from_str(&response_text) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to parse vision response: {}. Response: {}", e, response_text);
                return Err(anyhow!("Failed to parse response: {}", e));
            }
        };
        
        // Gemini 2.5 models return "thought" parts (internal reasoning) followed by
        // the actual response. Filter out thought parts to get the real answer.
        let parts = gemini_response
            .candidates
            .get(0)
            .map(|c| &c.content.parts)
            .cloned()
            .unwrap_or_default();

        // Prefer non-thought parts (the actual response)
        let non_thought: Vec<_> = parts.iter().filter(|p| !p.thought).collect();
        let text = if !non_thought.is_empty() {
            non_thought.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("")
        } else {
            // Fallback: use all parts if none are marked as non-thought
            parts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("")
        };

        tracing::info!("Vision analysis result ({} parts, {} thought): {}",
            parts.len(), parts.iter().filter(|p| p.thought).count(), text);
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
    #[serde(default)]
    text: String,
    /// Gemini 2.5 models include "thought" parts for internal reasoning
    #[serde(default)]
    thought: bool,
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

        // Build content dynamically - only include image if provided
        let mut content = vec![json!({
            "type": "text",
            "text": prompt
        })];
        if !image_base64.is_empty() {
            content.push(json!({
                "type": "image_url",
                "image_url": {
                    "url": format!("data:image/png;base64,{}", image_base64)
                }
            }));
        }

        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": content
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
    #[serde(default)]
    reasoning: Option<String>,
}

/// Ollama vision service (local models)
pub struct OllamaVisionService {
    model: String,
    client: Client,
}

impl OllamaVisionService {
    pub fn new(model: String) -> Self {
        Self {
            model,
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl VisionService for OllamaVisionService {
    async fn analyze(&self, image_base64: &str, prompt: &str) -> Result<String> {
        let url = "http://localhost:11434/v1/chat/completions";

        tracing::info!("Ollama Vision API call - model: {}, prompt_len: {}, image_len: {}",
            self.model, prompt.len(), image_base64.len());

        let mut content = vec![json!({"type": "text", "text": prompt})];
        if !image_base64.is_empty() {
            content.push(json!({
                "type": "image_url",
                "image_url": {"url": format!("data:image/png;base64,{}", image_base64)}
            }));
        }

        // For Qwen models, add explicit instruction to avoid reasoning mode
        let enhanced_prompt = if self.model.contains("qwen") {
            format!("{}\n\nRespond directly with ONLY the roast/comment itself. Do not include any thinking, reasoning, or analysis - just the final response.", prompt)
        } else {
            format!("{}\n\nIMPORTANT: Respond with ONLY the roast itself. No thinking, no analysis, no meta-commentary - just deliver the roast directly.", prompt)
        };

        let mut content = vec![json!({"type": "text", "text": enhanced_prompt})];
        if !image_base64.is_empty() {
            content.push(json!({
                "type": "image_url",
                "image_url": {"url": format!("data:image/png;base64,{}", image_base64)}
            }));
        }

        let body = json!({
            "model": self.model,
            "messages": [{"role": "user", "content": content}],
            "max_tokens": 2048,
            "temperature": 0.7,
            "stream": false,
            // Disable reasoning mode for Qwen models
            "options": {
                "num_predict": 2048
            }
        });

        let response = self.client
            .post(url)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        tracing::info!("Ollama API response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await?;
            tracing::error!("Ollama API error: {}", error_text);
            return Err(anyhow!("Ollama API error: {}", error_text));
        }

        let response_text = response.text().await?;
        tracing::info!("Ollama API response body: {}", response_text);

        let openai_response: OpenAIResponse = serde_json::from_str(&response_text)?;
        let text = openai_response.choices.get(0)
            .map(|c| {
                // Prefer content field first (the actual response)
                if !c.message.content.is_empty() {
                    return c.message.content.clone();
                }
                // If content is empty but reasoning exists, extract the roast from reasoning
                if let Some(reasoning) = &c.message.reasoning {
                    if !reasoning.is_empty() {
                        // Extract the actual roast from the reasoning field
                        // Look for the last few sentences that sound like the actual roast
                        return extract_roast_from_reasoning(reasoning);
                    }
                }
                String::new()
            })
            .unwrap_or_default();

        tracing::info!("Ollama analysis result: {}", text);
        Ok(text)
    }
}

/// Extract the actual roast from Qwen's reasoning field
fn extract_roast_from_reasoning(reasoning: &str) -> String {
    // Qwen's reasoning is cut off at max_tokens, so the actual roast might be incomplete
    // The reasoning contains analysis, but we want to extract something useful

    // Strategy: Look for the last complete thought/sentence that sounds like commentary
    // Skip meta-analysis phrases like "I should", "I need to", "Let me", "Wait"

    let lines: Vec<&str> = reasoning.lines().collect();

    // Find the last few lines that don't contain meta-analysis
    let mut roast_lines = Vec::new();
    for line in lines.iter().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Skip lines that are clearly meta-analysis
        if line.starts_with("I should")
            || line.starts_with("I need to")
            || line.starts_with("Let me")
            || line.starts_with("Wait,")
            || line.starts_with("First,")
            || line.starts_with("Also,")
            || line.starts_with("The user wants")
            || line.starts_with("Okay,")
            || line.contains("let's see")
            || line.contains("I'll")
        {
            continue;
        }

        roast_lines.push(line);

        // Take up to 3 good lines
        if roast_lines.len() >= 3 {
            break;
        }
    }

    if roast_lines.is_empty() {
        // Fallback: just take the last sentence
        let sentences: Vec<&str> = reasoning
            .split(|c| c == '.' || c == '!')
            .filter(|s| !s.trim().is_empty())
            .collect();

        if let Some(last) = sentences.last() {
            return last.trim().to_string();
        }

        return reasoning.to_string();
    }

    // Reverse to get original order
    roast_lines.reverse();
    roast_lines.join(" ")
}

/// Factory function to create the appropriate vision service
pub fn create_vision_service(provider: &str, api_key: String, model: String) -> Box<dyn VisionService> {
    match provider {
        "gemini" => Box::new(GeminiVisionService::new(api_key, model)),
        "ollama" => Box::new(OllamaVisionService::new(model)),
        _ => Box::new(OpenAIVisionService::new(api_key, model, provider)),
    }
}

