use anyhow::{anyhow, Result};
use reqwest::Client;
use rodio::{Decoder, OutputStream, Sink};
use serde::Deserialize;
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

        tracing::info!("TTS API call - model: {}, voice: {}, text_len: {}", 
            self.model_name(), self.voice, text.len());

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

        let status = response.status();
        let response_text: String = response.text().await?;
        
        tracing::info!("TTS API response status: {}", status);

        if !status.is_success() {
            tracing::error!("TTS API error: {}", response_text);
            return Err(anyhow!("Gemini TTS error: {}", response_text));
        }

        tracing::debug!("TTS raw response: {}", response_text);

        let gemini_response: GeminiAudioResponse = match serde_json::from_str(&response_text) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to parse TTS response: {}. Response: {}", e, response_text);
                return Err(anyhow!("Failed to parse TTS response: {}", e));
            }
        };
        
        // Extract audio data from response
        let audio_data = gemini_response
            .candidates
            .get(0)
            .and_then(|c| c.content.parts.get(0))
            .and_then(|p| p.inline_data.as_ref())
            .map(|d| decode_base64(&d.data))
            .ok_or_else(|| {
                tracing::error!("No audio data in TTS response");
                anyhow!("No audio data in response")
            })??;

        tracing::info!("TTS audio data received: {} bytes", audio_data.len());

        // Convert PCM to WAV if needed
        let audio_bytes = if let Some(mime) = gemini_response
            .candidates
            .get(0)
            .and_then(|c| c.content.parts.get(0))
            .and_then(|p| p.inline_data.as_ref())
            .map(|d| d.mime_type.clone()) 
        {
            tracing::info!("TTS audio mime type: {}", mime);
            if mime.contains("L16") || mime.contains("pcm") {
                pcm_to_wav(&audio_data, 24000)?
            } else {
                audio_data
            }
        } else {
            audio_data
        };

        tracing::info!("TTS final audio size: {} bytes", audio_bytes.len());
        Ok(audio_bytes)
    }
}

/// Gemini Live TTS service (unlimited free tier)
pub struct GeminiLiveTTSService {
    api_key: String,
    voice: String,
}

impl GeminiLiveTTSService {
    pub fn new(api_key: String, voice: String) -> Self {
        Self {
            api_key,
            voice,
        }
    }
}

#[async_trait::async_trait]
impl TTSService for GeminiLiveTTSService {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        // Fall back to standard Gemini TTS with the correct audio model
        let standard = GeminiTTSService::new(
            self.api_key.clone(),
            "gemini-2.5-flash-preview-tts".to_string(),
            self.voice.clone(),
        );
        standard.synthesize(text).await
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

/// Murf AI TTS service
pub struct MurfTTSService {
    api_key: String,
    model: String,
    voice: String,
    client: Client,
}

impl MurfTTSService {
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
impl TTSService for MurfTTSService {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        let url = "https://api.murf.ai/v1/speech/stream";

        tracing::info!("Murf TTS API call - model: {}, voice: {}, text_len: {}",
            self.model, self.voice, text.len());

        let body = json!({
            "text": text,
            "voiceId": self.voice,
            "model": self.model,
            "format": "WAV",
            "channelType": "MONO",
            "multiNativeLocale": "en-US",
            "sampleRate": 24000
        });

        let response = self
            .client
            .post(url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            tracing::error!("Murf TTS error ({}): {}", status, error_text);
            return Err(anyhow!("Murf TTS error: {}", error_text));
        }

        // Streaming endpoint returns audio bytes directly
        let audio_bytes = response.bytes().await?.to_vec();
        tracing::info!("Murf TTS audio: {} bytes", audio_bytes.len());

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
        tracing::info!("Starting audio playback thread, audio size: {} bytes", audio_bytes.len());
        std::thread::spawn(move || {
            tracing::info!("Audio playback thread started");
            if let Err(e) = Self::play(&audio_bytes) {
                tracing::error!("Audio playback error: {}", e);
            } else {
                tracing::info!("Audio playback completed successfully");
            }
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

/// ElevenLabs TTS service
pub struct ElevenLabsTTSService {
    api_key: String,
    model: String,
    voice: String,
    client: Client,
}

impl ElevenLabsTTSService {
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
impl TTSService for ElevenLabsTTSService {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        let url = format!(
            "https://api.elevenlabs.io/v1/text-to-speech/{}",
            self.voice
        );

        tracing::info!("ElevenLabs TTS API call - model: {}, voice: {}, text_len: {}",
            self.model, self.voice, text.len());

        let body = json!({
            "text": text,
            "model_id": self.model
        });

        let response = self
            .client
            .post(&url)
            .header("xi-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("Accept", "audio/mpeg")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            tracing::error!("ElevenLabs TTS error ({}): {}", status, error_text);
            return Err(anyhow!("ElevenLabs TTS error: {}", error_text));
        }

        let audio_bytes = response.bytes().await?.to_vec();
        tracing::info!("ElevenLabs TTS audio: {} bytes", audio_bytes.len());
        Ok(audio_bytes)
    }
}

/// Inworld AI TTS service
pub struct InworldTTSService {
    api_key: String,
    model: String,
    voice: String,
    client: Client,
}

impl InworldTTSService {
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
impl TTSService for InworldTTSService {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        let url = "https://api.inworld.ai/tts/v1/tts";

        tracing::info!("Inworld TTS API call - model: {}, voice: {}, text_len: {}",
            self.model, self.voice, text.len());

        let body = json!({
            "text": text,
            "voice_id": self.voice,
            "model": self.model,
            "response_format": "wav"
        });

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            tracing::error!("Inworld TTS error ({}): {}", status, error_text);
            return Err(anyhow!("Inworld TTS error: {}", error_text));
        }

        let audio_bytes = response.bytes().await?.to_vec();
        tracing::info!("Inworld TTS audio: {} bytes", audio_bytes.len());
        Ok(audio_bytes)
    }
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
        "murf" => Box::new(MurfTTSService::new(api_key, model, voice)),
        "elevenlabs" => Box::new(ElevenLabsTTSService::new(api_key, model, voice)),
        "inworld" => Box::new(InworldTTSService::new(api_key, model, voice)),
        "system-tts" => Box::new(SystemTTSService::new(voice)),
        "kokoro" => Box::new(KokoroTTSService::new(voice)),
        _ => Box::new(OpenAITTSService::new(api_key, model, voice)),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiAudioResponse {
    candidates: Vec<GeminiAudioCandidate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiAudioCandidate {
    content: GeminiAudioContent,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiAudioContent {
    parts: Vec<GeminiAudioPart>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GeminiAudioPart {
    inline_data: Option<GeminiInlineData>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GeminiInlineData {
    mime_type: String,
    data: String,
}

// Base64 decode helper
pub fn decode_base64(input: &str) -> anyhow::Result<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.decode(input).map_err(|e| anyhow::anyhow!("Base64 decode error: {}", e))
}

/// System TTS service (uses OS built-in TTS via Tauri plugin)
pub struct SystemTTSService {
    _voice: String,
}

impl SystemTTSService {
    pub fn new(voice: String) -> Self {
        Self { _voice: voice }
    }
}

#[async_trait::async_trait]
impl TTSService for SystemTTSService {
    async fn synthesize(&self, _text: &str) -> Result<Vec<u8>> {
        // System TTS doesn't return audio bytes, it plays directly
        // Return empty vec as placeholder
        tracing::info!("System TTS would speak: {}", _text);
        Ok(Vec::new())
    }
}

/// Kokoro-82M TTS service (local neural TTS)
pub struct KokoroTTSService {
    voice: String,
}

impl KokoroTTSService {
    pub fn new(voice: String) -> Self {
        Self { voice }
    }
}

#[async_trait::async_trait]
impl TTSService for KokoroTTSService {
    async fn synthesize(&self, _text: &str) -> Result<Vec<u8>> {
        tracing::info!("Kokoro TTS with voice: {}", self.voice);

        // Kokoro requires complex build dependencies (libclang)
        // Will implement via ONNX Runtime directly in future update
        Err(anyhow!("Kokoro TTS coming soon - use System TTS for now"))
    }
}
