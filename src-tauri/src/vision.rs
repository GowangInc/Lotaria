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

        // For Qwen models, use /no_think to disable reasoning mode
        let enhanced_prompt = if self.model.contains("qwen") {
            format!("/no_think\n{}\n\nRespond directly with ONLY the roast/comment. No thinking, no reasoning. Just the final 2-3 sentence response.", prompt)
        } else {
            format!("{}\n\nIMPORTANT: Respond with ONLY the roast itself. No thinking, no analysis - just deliver the roast directly.", prompt)
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
            "max_tokens": 40960,
            "temperature": 0.7,
            "stream": false,
            "options": {
                "num_ctx": 131072,
                "num_predict": 40960
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
        let raw_text = openai_response.choices.get(0)
            .map(|c| {
                if !c.message.content.is_empty() {
                    return c.message.content.clone();
                }
                if let Some(reasoning) = &c.message.reasoning {
                    if !reasoning.is_empty() {
                        return extract_roast_from_reasoning(reasoning);
                    }
                }
                String::new()
            })
            .unwrap_or_default();

        // Strip any thinking/reasoning that leaked into the content
        let text = strip_thinking_from_content(&raw_text);

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

/// Strip thinking/reasoning content that leaked into the actual response.
/// Qwen models often dump their chain-of-thought into the content field despite /no_think.
fn strip_thinking_from_content(text: &str) -> String {
    // Strategy 1: Extract the longest quoted block (the model's "draft")
    // Handles both straight quotes and smart quotes
    if let Some(roast) = extract_longest_quoted(text) {
        if roast.len() > 30 {
            return roast;
        }
    }

    // Strategy 2: Split on known thinking markers and take the content after the last one
    let thinking_markers = [
        "Possible draft:",
        "possible draft:",
        "Let's draft:",
        "let's draft:",
        "Draft:",
        "draft:",
        "Final version:",
        "final version:",
        "Here's the roast:",
        "here's the roast:",
        "My response:",
        "Response:",
    ];

    for marker in &thinking_markers {
        if let Some(idx) = text.find(marker) {
            let after = text[idx + marker.len()..].trim();
            // Strip leading quotes if present
            let after = after.trim_start_matches('"')
                .trim_start_matches('\u{201c}'); // left smart quote
            // Take until end or next thinking marker
            let cleaned = strip_trailing_thinking(after);
            if cleaned.len() > 30 {
                return cleaned;
            }
        }
    }

    // Strategy 3: Sentence-level filtering — keep sentences that look like actual roasts
    // (contain "you", "your", direct address) and drop meta-analysis sentences
    let sentences = split_sentences(text);
    let roast_sentences: Vec<&str> = sentences.iter()
        .filter(|s| is_roast_sentence(s))
        .copied()
        .collect();

    if !roast_sentences.is_empty() {
        // Take up to 3 roast sentences
        let result: String = roast_sentences.iter()
            .take(3)
            .copied()
            .collect::<Vec<_>>()
            .join(" ");
        if result.len() > 30 {
            return result;
        }
    }

    // Strategy 4: Last resort — just return the original, truncated
    text.to_string()
}

/// Extract the longest quoted block from text (straight or smart quotes)
fn extract_longest_quoted(text: &str) -> Option<String> {
    let mut best: Option<String> = None;

    // Try straight quotes
    for block in extract_between(text, '"', '"') {
        if best.as_ref().map_or(true, |b| block.len() > b.len()) {
            best = Some(block);
        }
    }
    // Try smart quotes
    for block in extract_between_str(text, "\u{201c}", "\u{201d}") {
        if best.as_ref().map_or(true, |b| block.len() > b.len()) {
            best = Some(block);
        }
    }

    best
}

fn extract_between(text: &str, open: char, close: char) -> Vec<String> {
    let mut results = Vec::new();
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].1 == open {
            let content_start = chars[i].0 + chars[i].1.len_utf8();
            i += 1;
            while i < chars.len() {
                if chars[i].1 == '\\' {
                    i += 2; // skip escaped char
                    continue;
                }
                if chars[i].1 == close {
                    results.push(text[content_start..chars[i].0].to_string());
                    break;
                }
                i += 1;
            }
        }
        i += 1;
    }
    results
}

fn extract_between_str(text: &str, open: &str, close: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut search_from = 0;
    while let Some(start) = text[search_from..].find(open) {
        let content_start = search_from + start + open.len();
        if let Some(end) = text[content_start..].find(close) {
            results.push(text[content_start..content_start + end].to_string());
            search_from = content_start + end + close.len();
        } else {
            break;
        }
    }
    results
}

/// Strip trailing thinking/meta content from extracted text
fn strip_trailing_thinking(text: &str) -> String {
    let trailing_markers = [
        "Check character count",
        "check character count",
        "Need to be",
        "need to be",
        "Let's count",
        "let's count",
        "Wait,",
        "Make it more",
        "make it more",
        "Previous drafts",
        "previous drafts",
    ];

    let mut result = text.to_string();
    for marker in &trailing_markers {
        if let Some(idx) = result.find(marker) {
            result.truncate(idx);
        }
    }

    // Trim trailing quote chars and whitespace
    result.trim_end_matches('"')
        .trim_end_matches('\u{201d}')
        .trim()
        .to_string()
}

/// Split text into sentences (on . ! ?)
fn split_sentences(text: &str) -> Vec<&str> {
    let mut sentences = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        if (b == b'.' || b == b'!' || b == b'?')
            && (i + 1 >= bytes.len() || bytes[i + 1] == b' ' || bytes[i + 1] == b'\n')
        {
            let s = text[start..=i].trim();
            if !s.is_empty() {
                sentences.push(s);
            }
            start = i + 1;
        }
    }
    // Trailing fragment
    let s = text[start..].trim();
    if !s.is_empty() {
        sentences.push(s);
    }
    sentences
}

/// Heuristic: does this sentence look like an actual roast vs meta-analysis?
fn is_roast_sentence(s: &str) -> bool {
    let lower = s.to_lowercase();

    // Definitely meta-analysis — reject
    let meta_patterns = [
        "check character", "character count", "need to be", "let's count",
        "let's draft", "possible draft", "previous draft", "make it more",
        "2-3 sentences", "under 500", "max characters", "i should",
        "i need to", "the user", "add the failure", "wait,",
        "let me", "first,", "also,", "okay,",
    ];
    for pat in &meta_patterns {
        if lower.contains(pat) {
            return false;
        }
    }

    // Likely a roast — contains direct address or vivid language
    let roast_signals = ["you", "your", "you're", "you've"];
    let has_direct_address = roast_signals.iter().any(|w| {
        lower.split_whitespace().any(|word| word.trim_matches(|c: char| !c.is_alphanumeric()) == *w)
    });

    // Also accept sentences with em-dashes, strong imagery
    let has_style = lower.contains("—") || lower.contains("...") || lower.contains("*");

    has_direct_address || has_style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_thinking_with_quoted_draft() {
        let input = r#"Make sure it's brutal, no softening. Example: "9 PM and your terminal's still screaming 'bypass permissions' like your life's a pirated movie—while 'Beijing Dads' learns Chinese and *you* learn how many ways to fail. Your code's literally mocking you." Check character count. Need to be under 500."#;
        let result = strip_thinking_from_content(input);
        assert!(result.contains("9 PM"), "Should extract the quoted roast, got: {}", result);
        assert!(!result.contains("Check character count"), "Should not contain meta-analysis, got: {}", result);
    }

    #[test]
    fn test_strip_thinking_with_meta_sentences() {
        let input = "Add the failure angle: their life is a pirated movie. Check character count. Previous drafts were around 2-3 sentences. Need to be brutal. 9 PM and your terminal's still screaming—while you learn how many ways to fail.";
        let result = strip_thinking_from_content(input);
        assert!(result.contains("your"), "Should keep roast sentences, got: {}", result);
        assert!(!result.contains("Check character count"), "Should strip meta, got: {}", result);
    }

    #[test]
    fn test_strip_clean_roast_unchanged() {
        let input = "9 PM and you're still here pretending to code while your terminal judges you silently.";
        let result = strip_thinking_from_content(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_strip_thinking_draft_marker() {
        let input = "Let me think about this. Possible draft: \"Your 47 tabs are a cry for help that nobody's answering.\" That should work.";
        let result = strip_thinking_from_content(input);
        assert!(result.contains("47 tabs"), "Should extract after draft marker, got: {}", result);
    }
}

/// Factory function to create the appropriate vision service
pub fn create_vision_service(provider: &str, api_key: String, model: String) -> Box<dyn VisionService> {
    match provider {
        "gemini" => Box::new(GeminiVisionService::new(api_key, model)),
        "ollama" => Box::new(OllamaVisionService::new(model)),
        _ => Box::new(OpenAIVisionService::new(api_key, model, provider)),
    }
}

