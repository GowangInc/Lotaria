use anyhow::{anyhow, Result};
use reqwest::Client;
use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Cursor;

/// TTS service trait
#[async_trait::async_trait]
pub trait TTSService: Send + Sync {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>>;
}

/// Gemini TTS service (standard)
pub struct GeminiTTSService {
    api_key: String,
    model: String,
    voice: String,
    client: Client,
}

impl GeminiTTSService {
    pub fn new(api_key: String, model: String, voice: String) -> Self {
        Self {
            api_key,
            model,
            voice,
            client: Client::new(),
        }
    }

    fn model_name(&self) -> String {
        if self.model.starts_with("gemini-") {
            self.model.clone()
        } else {
            format!("gemini-{}", self.model)
        }
    }
}

#[async_trait::async_trait]
impl TTSService for GeminiTTSService {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model_name(),
            self.api_key
        );

        let body = json!({
            "contents": [{
                "parts": [{"text": text}]
            }],
            "generationConfig": {
                "responseModalities": ["AUDIO"],
                "speechConfig": {
                    "voiceConfig": {
                        "prebuiltVoiceConfig": {
                            "voiceName": self.voice
                        }
                    }
                }
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
            return Err(anyhow!("Gemini TTS error: {}", error_text));
        }

        let gemini_response: GeminiAudioResponse = response.json().await?;
        
        // Extract audio data from response
        let audio_data = gemini_response
            .candidates
            .get(0)
            .and_then(|c| c.content.parts.get(0))
            .and_then(|p| p.inline_data.as_ref())
            .map(|d| decode_base64(&d.data))
            .ok_or_else(|| anyhow!("No audio data in response"))??;

        // Convert PCM to WAV if needed
        let audio_bytes = if let Some(mime) = gemini_response
            .candidates
            .get(0)
            .and_then(|c| c.content.parts.get(0))
            .and_then(|p| p.inline_data.as_ref())
            .map(|d| d.mime_type.clone()) 
        {
            if mime.contains("L16") || mime.contains("pcm") {
                pcm_to_wav(&audio_data, 24000)?
            } else {
                audio_data
            }
        } else {
            audio_data
        };

        Ok(audio_bytes)
    }
}

/// Gemini Live TTS service (unlimited free tier)
pub struct GeminiLiveTTSService {
    api_key: String,
    voice: String,
    client: Client,
}

impl GeminiLiveTTSService {
    pub fn new(api_key: String, voice: String) -> Self {
        Self {
            api_key,
            voice,
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl TTSService for GeminiLiveTTSService {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        // For now, fall back to standard Gemini TTS
        // Full Live API implementation would use WebSocket
        let standard = GeminiTTSService::new(
            self.api_key.clone(),
            "gemini-2.5-flash-native-audio-preview-12-2025".to_string(),
            self.voice.clone(),
        );
        
        // Wrap the text to be read aloud
        let wrapped_text = format!("Read the following text aloud naturally: {}", text);
        standard.synthesize(&wrapped_text).await
    }
}

/// OpenAI TTS service
pub struct OpenAITTSService {
    api_key: String,
    model: String,
    voice: String,
    client: Client,
}

impl OpenAITTSService {
    pub fn new(api_key: String, model: String, voice: String) -> Self {
        Self {
            api_key,
            model,
            voice,
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl TTSService for OpenAITTSService {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        let url = "https://api.openai.com/v1/audio/speech";

        let body = json!({
            "model": self.model,
            "voice": self.voice,
            "input": text
        });

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI TTS error: {}", error_text));
        }

        let audio_bytes = response.bytes().await?.to_vec();
        Ok(audio_bytes)
    }
}

/// Audio player using rodio
pub struct AudioPlayer;

impl AudioPlayer {
    /// Play audio bytes (WAV or MP3)
    pub fn play(audio_bytes: &[u8]) -> Result<()> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        
        let cursor = Cursor::new(audio_bytes.to_vec());
        let source = Decoder::new(cursor)?;
        
        sink.append(source);
        sink.sleep_until_end();
        
        Ok(())
    }

    /// Play audio in a non-blocking way
    pub fn play_async(audio_bytes: Vec<u8>) -> Result<()> {
        std::thread::spawn(move || {
            let _ = Self::play(&audio_bytes);
        });
        Ok(())
    }
}

/// Convert PCM16 to WAV format
fn pcm_to_wav(pcm_data: &[u8], sample_rate: u32) -> Result<Vec<u8>> {
    use byteorder::{LittleEndian, WriteBytesExt};
    
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_size = pcm_data.len() as u32;
    let file_size = 36 + data_size;

    let mut wav = Vec::new();
    
    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.write_u32::<LittleEndian>(file_size)?;
    wav.extend_from_slice(b"WAVE");
    
    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.write_u32::<LittleEndian>(16)?; // Subchunk1Size
    wav.write_u16::<LittleEndian>(1)?; // AudioFormat (PCM)
    wav.write_u16::<LittleEndian>(num_channels)?;
    wav.write_u32::<LittleEndian>(sample_rate)?;
    wav.write_u32::<LittleEndian>(byte_rate)?;
    wav.write_u16::<LittleEndian>(block_align)?;
    wav.write_u16::<LittleEndian>(bits_per_sample)?;
    
    // data chunk
    wav.extend_from_slice(b"data");
    wav.write_u32::<LittleEndian>(data_size)?;
    wav.extend_from_slice(pcm_data);
    
    Ok(wav)
}

/// Factory function to create the appropriate TTS service
pub fn create_tts_service(provider: &str, api_key: String, model: String, voice: String) -> Box<dyn TTSService> {
    match provider {
        "gemini" => {
            if model.contains("live") {
                Box::new(GeminiLiveTTSService::new(api_key, voice))
            } else {
                Box::new(GeminiTTSService::new(api_key, model, voice))
            }
        }
        _ => Box::new(OpenAITTSService::new(api_key, model, voice)),
    }
}

#[derive(Debug, Deserialize)]
struct GeminiAudioResponse {
    candidates: Vec<GeminiAudioCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiAudioCandidate {
    content: GeminiAudioContent,
}

#[derive(Debug, Deserialize)]
struct GeminiAudioContent {
    parts: Vec<GeminiAudioPart>,
}

#[derive(Debug, Deserialize, Clone)]
struct GeminiAudioPart {
    inline_data: Option<GeminiInlineData>,
}

#[derive(Debug, Deserialize, Clone)]
struct GeminiInlineData {
    mime_type: String,
    data: String,
}

// Base64 decode helper
pub fn decode_base64(input: &str) -> anyhow::Result<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.decode(input).map_err(|e| anyhow::anyhow!("Base64 decode error: {}", e))
}
