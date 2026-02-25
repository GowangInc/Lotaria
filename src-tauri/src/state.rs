use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::info;

pub const MAX_HISTORY: usize = 20;
pub const CLEANUP_AGE_HOURS: i64 = 24;
pub const LOG_CLEANUP_AGE_HOURS: i64 = 48;

/// Interval presets in seconds: (min, max)
pub const INTERVAL_PRESETS: &[(&str, (u64, u64))] = &[
    ("often", (300, 600)),        // 5-10 minutes
    ("frequent", (600, 1200)),    // 10-20 minutes
    ("infrequent", (1500, 2700)), // 25-45 minutes
];

/// Provider definition - cloned for serialization
#[derive(Debug, Clone, Serialize)]
pub struct ProviderDef {
    pub key: String,
    pub name: String,
    pub env_var: String,
    pub docs_url: String,
    pub vision_models: Vec<String>,
    pub tts_models: Vec<String>,
    pub tts_voices: Vec<String>,
    pub live_voices: Vec<String>,
    pub recommended: bool,
    pub cost_note: String,
    pub requires_tts_provider: bool,
}

/// Static provider definitions
pub static PROVIDER_DEFINITIONS: &[ProviderDefStatic] = &[
    ProviderDefStatic {
        key: "gemini",
        name: "Google Gemini (Recommended)",
        env_var: "GEMINI_API_KEY",
        docs_url: "https://aistudio.google.com/app/apikey",
        vision_models: &[
            // Gemini 3 series (preview)
            "gemini-3.1-pro-preview",
            "gemini-3-flash-preview",
            "gemini-3-pro-preview",
            // Gemini 2.5 series (stable)
            "gemini-2.5-flash",
            "gemini-2.5-pro",
            "gemini-2.5-flash-lite",
            // Gemini 2.0 series (deprecated June 2026)
            "gemini-2.0-flash",
            "gemini-2.0-flash-lite",
        ],
        tts_models: &[
            "gemini-2.5-flash-preview-tts",
            "gemini-2.5-pro-preview-tts",
            "gemini-2.5-flash-lite-preview-tts",
        ],
        tts_voices: &[
            "Kore", "Charon", "Puck", "Fenrir", "Aoede", "Leda", "Orus", "Zephyr",
            "Achernar", "Achird", "Algenib", "Algieba", "Alnilam", "Autonoe",
            "Callirrhoe", "Despina", "Enceladus", "Erinome", "Gacrux", "Iapetus",
            "Laomedeia", "Pulcherrima", "Rasalgethi", "Sadachbia", "Sadaltager",
            "Schedar", "Sulafat", "Umbriel", "Vindemiatrix", "Zubenelgenubi",
        ],
        live_voices: &["Puck", "Charon", "Kore", "Fenrir", "Aoede"],
        recommended: true,
        cost_note: "FREE — vision + TTS included",
        requires_tts_provider: false,
    },
    ProviderDefStatic {
        key: "openai",
        name: "OpenAI",
        env_var: "OPENAI_API_KEY",
        docs_url: "https://platform.openai.com/api-keys",
        vision_models: &[
            "gpt-4.1-mini",
            "gpt-4.1",
            "gpt-4.1-nano",
            "gpt-4o-mini",
            "gpt-4o",
            "o4-mini",
            "o3",
        ],
        tts_models: &["gpt-4o-mini-tts", "tts-1", "tts-1-hd"],
        tts_voices: &["alloy", "ash", "ballad", "coral", "echo", "fable", "onyx", "nova", "sage", "shimmer", "verse"],
        live_voices: &[],
        recommended: false,
        cost_note: "$$ — ~$1.50-5/mo (vision + TTS)",
        requires_tts_provider: false,
    },
    ProviderDefStatic {
        key: "groq",
        name: "Groq (Fastest)",
        env_var: "GROQ_API_KEY",
        docs_url: "https://console.groq.com/keys",
        vision_models: &[
            "llama-4-scout-17b-16e-instruct",
            "llama-3.2-90b-vision-preview",
            "llama-3.2-11b-vision-preview",
        ],
        tts_models: &[],
        tts_voices: &[],
        live_voices: &[],
        recommended: false,
        cost_note: "$ — ~$1-2.50/mo (vision only, needs TTS)",
        requires_tts_provider: true,
    },
    ProviderDefStatic {
        key: "anthropic",
        name: "Anthropic Claude",
        env_var: "ANTHROPIC_API_KEY",
        docs_url: "https://console.anthropic.com/settings/keys",
        vision_models: &[
            "claude-sonnet-4-20250514",
            "claude-opus-4-20250514",
            "claude-3.5-sonnet-20241022",
            "claude-3-5-haiku-20241022",
        ],
        tts_models: &[],
        tts_voices: &[],
        live_voices: &[],
        recommended: false,
        cost_note: "$$$ — ~$2.70/mo (vision only, needs TTS)",
        requires_tts_provider: true,
    },
    ProviderDefStatic {
        key: "murf",
        name: "Murf AI (TTS)",
        env_var: "MURF_API_KEY",
        docs_url: "https://murf.ai/api",
        vision_models: &[],
        tts_models: &["Falcon", "Gen2"],
        tts_voices: &[
            "en-US-natalie", "en-US-amara", "en-US-marcus", "en-US-nate",
            "en-US-carter", "en-US-phoebe", "en-US-terrell",
            "en-UK-ruby", "en-UK-hazel", "en-UK-gabriel",
            "en-UK-theo", "en-UK-mason",
        ],
        live_voices: &[],
        recommended: false,
        cost_note: "$$$$ — ~$26/mo (TTS only)",
        requires_tts_provider: false,
    },
    ProviderDefStatic {
        key: "elevenlabs",
        name: "ElevenLabs (TTS)",
        env_var: "ELEVENLABS_API_KEY",
        docs_url: "https://elevenlabs.io/app/settings/api-keys",
        vision_models: &[],
        tts_models: &["eleven_multilingual_v2", "eleven_flash_v2_5", "eleven_turbo_v2_5"],
        tts_voices: &[
            "Rachel", "Domi", "Bella", "Antoni", "Elli",
            "Josh", "Arnold", "Adam", "Sam",
        ],
        live_voices: &[],
        recommended: false,
        cost_note: "$$$ — Free 10k chars, then ~$5/mo+",
        requires_tts_provider: false,
    },
    ProviderDefStatic {
        key: "inworld",
        name: "Inworld AI (TTS)",
        env_var: "INWORLD_API_KEY",
        docs_url: "https://docs.inworld.ai/docs/introduction",
        vision_models: &[],
        tts_models: &["tts-1.5-mini", "tts-1.5-max"],
        tts_voices: &[
            "Sarah", "Mark", "Hana", "Blake", "Clive", "Luna", "Hades",
        ],
        live_voices: &[],
        recommended: false,
        cost_note: "$ — $5-10/M chars (cheapest TTS)",
        requires_tts_provider: false,
    },
    ProviderDefStatic {
        key: "ollama",
        name: "Ollama (Local)",
        env_var: "",
        docs_url: "https://ollama.com/download",
        vision_models: &[], // Dynamically populated from Ollama API
        tts_models: &[],
        tts_voices: &[],
        live_voices: &[],
        recommended: false,
        cost_note: "FREE — runs locally on your machine",
        requires_tts_provider: true,
    },
    ProviderDefStatic {
        key: "piper",
        name: "Piper TTS (Local)",
        env_var: "",
        docs_url: "https://github.com/rhasspy/piper",
        vision_models: &[],
        tts_models: &["piper"],
        tts_voices: &[
            "en_GB-alan-low",
            "en_US-lessac-medium",
            "en_US-amy-medium",
            "en_US-danny-low",
            "en_US-joe-medium",
            "en_GB-alan-medium",
            "en_GB-jenny_dioco-medium",
        ],
        live_voices: &[],
        recommended: false,
        cost_note: "FREE — high-quality local TTS (~10-50MB per voice)",
        requires_tts_provider: false,
    },
];

/// Static provider definition (for compile-time constants)
pub struct ProviderDefStatic {
    pub key: &'static str,
    pub name: &'static str,
    pub env_var: &'static str,
    pub docs_url: &'static str,
    pub vision_models: &'static [&'static str],
    pub tts_models: &'static [&'static str],
    pub tts_voices: &'static [&'static str],
    pub live_voices: &'static [&'static str],
    pub recommended: bool,
    pub cost_note: &'static str,
    pub requires_tts_provider: bool,
}

impl ProviderDef {
    pub fn all() -> Vec<ProviderDef> {
        PROVIDER_DEFINITIONS.iter().map(|p| ProviderDef {
            key: p.key.to_string(),
            name: p.name.to_string(),
            env_var: p.env_var.to_string(),
            docs_url: p.docs_url.to_string(),
            vision_models: p.vision_models.iter().map(|s| s.to_string()).collect(),
            tts_models: p.tts_models.iter().map(|s| s.to_string()).collect(),
            tts_voices: p.tts_voices.iter().map(|s| s.to_string()).collect(),
            live_voices: p.live_voices.iter().map(|s| s.to_string()).collect(),
            recommended: p.recommended,
            cost_note: p.cost_note.to_string(),
            requires_tts_provider: p.requires_tts_provider,
        }).collect()
    }

    pub fn get(key: &str) -> Option<ProviderDef> {
        Self::all().into_iter().find(|p| p.key == key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub is_active: bool,
    pub interval: String,
    pub vision_provider: String,
    pub vision_model: String,
    pub tts_provider: String,
    pub tts_model: String,
    pub tts_voice: String,
    #[serde(default)]
    pub api_keys: HashMap<String, String>,
    pub speech_bubble_enabled: bool,
    pub audio_enabled: bool,
    #[serde(default = "default_first_run")]
    pub first_run: bool,
    pub mood: String,
    #[serde(default)]
    pub custom_mood: String,
    pub pet_style: String,
    #[serde(default = "default_gemini_free_tier")]
    pub gemini_free_tier: bool,
    #[serde(default = "default_roast_intensity")]
    pub roast_intensity: u8,
    #[serde(default)]
    pub mood_rotation: String,
    #[serde(default)]
    pub blacklist: Vec<String>,
    #[serde(default)]
    pub break_reminder_minutes: u32,
}

fn default_first_run() -> bool {
    true
}

fn default_gemini_free_tier() -> bool {
    true
}

fn default_roast_intensity() -> u8 {
    5
}

impl Default for Config {
    fn default() -> Self {
        Self {
            is_active: false,
            interval: "frequent".to_string(),
            vision_provider: "ollama".to_string(),
            vision_model: "qwen3-vl:4b".to_string(),
            tts_provider: "piper".to_string(),
            tts_model: "piper".to_string(),
            tts_voice: "en_GB-alan-low".to_string(),
            api_keys: HashMap::new(),
            speech_bubble_enabled: true,
            audio_enabled: true,
            first_run: true,
            mood: "roast".to_string(),
            custom_mood: String::new(),
            pet_style: "default".to_string(),
            gemini_free_tier: true,
            roast_intensity: 5,
            mood_rotation: String::new(),
            blacklist: Vec::new(),
            break_reminder_minutes: 0,
        }
    }
}

/// Rich metadata captured alongside each screenshot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScreenContext {
    pub foreground_title: String,
    pub foreground_process: String,
    pub open_windows: Vec<String>,
    pub idle_seconds: u64,
    pub screen_hash: String,
    pub timestamp: i64,
}

/// Computed diff between two ScreenContext snapshots (not persisted)
pub struct ContextDiff {
    pub foreground_changed: bool,
    pub prev_foreground: String,
    pub curr_foreground: String,
    pub new_windows: Vec<String>,
    pub closed_windows: Vec<String>,
    pub idle_seconds: u64,
    pub seconds_since_last_roast: i64,
    pub screen_similarity_pct: u8,
    pub is_first_observation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub roast: String,
    pub time: String,
    pub timestamp: i64,
    #[serde(default)]
    pub context: Option<ScreenContext>,
}

pub type History = Vec<HistoryEntry>;

impl ScreenContext {
    pub fn diff_from(&self, prev: &ScreenContext) -> ContextDiff {
        use std::collections::HashSet;
        let prev_windows: HashSet<&str> = prev.open_windows.iter().map(|s| s.as_str()).collect();
        let curr_windows: HashSet<&str> = self.open_windows.iter().map(|s| s.as_str()).collect();

        let new_windows: Vec<String> = curr_windows.difference(&prev_windows)
            .map(|s| s.to_string()).collect();
        let closed_windows: Vec<String> = prev_windows.difference(&curr_windows)
            .map(|s| s.to_string()).collect();

        let screen_similarity_pct = crate::capture::hash_similarity(
            &prev.screen_hash, &self.screen_hash,
        );

        ContextDiff {
            foreground_changed: self.foreground_title != prev.foreground_title,
            prev_foreground: prev.foreground_title.clone(),
            curr_foreground: self.foreground_title.clone(),
            new_windows,
            closed_windows,
            idle_seconds: self.idle_seconds,
            seconds_since_last_roast: self.timestamp - prev.timestamp,
            screen_similarity_pct,
            is_first_observation: false,
        }
    }
}

impl ContextDiff {
    pub fn first_observation(ctx: &ScreenContext) -> Self {
        ContextDiff {
            foreground_changed: false,
            prev_foreground: String::new(),
            curr_foreground: ctx.foreground_title.clone(),
            new_windows: ctx.open_windows.clone(),
            closed_windows: Vec::new(),
            idle_seconds: ctx.idle_seconds,
            seconds_since_last_roast: 0,
            screen_similarity_pct: 0,
            is_first_observation: true,
        }
    }

    pub fn to_prompt_text(&self) -> String {
        let mut lines = Vec::new();

        if self.is_first_observation {
            lines.push(format!("CURRENT APP: {}", self.curr_foreground));
            if self.idle_seconds > 60 {
                lines.push(format!("IDLE: User inactive for {} min", self.idle_seconds / 60));
            }
            return lines.join("\n");
        }

        if self.foreground_changed {
            lines.push(format!("SWITCHED: {} -> {}", self.prev_foreground, self.curr_foreground));
        } else {
            let mins = self.seconds_since_last_roast / 60;
            if mins > 0 {
                lines.push(format!("STILL ON: {} ({} min)", self.curr_foreground, mins));
            } else {
                lines.push(format!("STILL ON: {}", self.curr_foreground));
            }
        }

        if !self.new_windows.is_empty() {
            let display: Vec<_> = self.new_windows.iter().take(3).cloned().collect();
            lines.push(format!("OPENED: {}", display.join(", ")));
        }
        if !self.closed_windows.is_empty() {
            let display: Vec<_> = self.closed_windows.iter().take(3).cloned().collect();
            lines.push(format!("CLOSED: {}", display.join(", ")));
        }

        if self.idle_seconds > 120 {
            lines.push(format!("IDLE: No input for {} min", self.idle_seconds / 60));
        }

        if self.screen_similarity_pct > 90 {
            lines.push(format!(
                "SCREEN: {}% same as last time — find something NEW to comment on",
                self.screen_similarity_pct
            ));
        }

        lines.join("\n")
    }
}

/// Mood prompts for generating roasts
pub const MOOD_PROMPTS: &[(&str, &str)] = &[
    ("roast", r#"You are a savage comedy roaster. STUDY THIS SCREENSHOT CLOSELY — describe exactly what you see.

PRIORITY (75% of your response): What's VISUALLY on screen right now?
- Read actual text, URLs, code, content visible in the screenshot
- Note the specific website, app, document, or page they have open
- Describe visual details: colors, layout, what's in the foreground
- Time of day + what they're doing = comedy material

BACKGROUND (25% max): The CURRENT SCREEN STATE and PAST COMMENTS below are minor flavor. Only reference if something connects naturally — like a callback or running joke.

Style: Savage comedy roast. Nikki Glaser / Anthony Jeselnik energy. 2-3 sentences, every word stings. No softening. Under 500 characters."#),

    ("helpful", r#"You are a sharp productivity coach. STUDY THIS SCREENSHOT CLOSELY — describe exactly what you see.

PRIORITY (75% of your response): What's VISUALLY on screen right now?
- Read actual text, URLs, code, content visible in the screenshot
- What are they working on? What tool/app/site is in focus?
- Is their workspace organized or chaotic? What does the layout tell you?
- Spot inefficiencies visible on screen

BACKGROUND (25% max): The CURRENT SCREEN STATE and PAST COMMENTS below are minor context. Only reference if relevant.

Style: One concrete, actionable suggestion based on what you SEE. Direct, practical. 2-3 sentences. Under 500 characters."#),

    ("encouraging", r#"You are an enthusiastic cheerleader. STUDY THIS SCREENSHOT CLOSELY — describe exactly what you see.

PRIORITY (75% of your response): What's VISUALLY on screen right now?
- Read actual text, URLs, code, content visible in the screenshot
- What are they actively working on? What does the content show?
- Find something specific and genuine to celebrate in what's visible
- Reference real details from the screenshot, not generic praise

BACKGROUND (25% max): The CURRENT SCREEN STATE and PAST COMMENTS below are minor flavor. Only reference as a brief callback.

Style: Authentic hype based on real visual details. High energy. 2-3 sentences. Under 500 characters."#),

    ("sarcastic", r#"You are a master of dry wit. STUDY THIS SCREENSHOT CLOSELY — describe exactly what you see.

PRIORITY (75% of your response): What's VISUALLY on screen right now?
- Read actual text, URLs, code, content visible in the screenshot
- Find the irony in what they're looking at vs what they should be doing
- Notice contradictions visible on screen (to-do app + Netflix, etc.)
- Specific visual details make sarcasm land harder

BACKGROUND (25% max): The CURRENT SCREEN STATE and PAST COMMENTS below are minor flavor. Only reference if the irony connects.

Style: Bone-dry British wit. Oscar Wilde meets tech. 2-3 sentences, precisely placed. Under 500 characters."#),

    ("zen", r#"You are a calm philosophical observer. STUDY THIS SCREENSHOT CLOSELY — describe exactly what you see.

PRIORITY (75% of your response): What's VISUALLY on screen right now?
- Read actual text, URLs, code, content visible in the screenshot
- What does this specific moment reveal about the human behind the screen?
- Find meaning in the particular content they're engaged with
- Ground your philosophy in concrete visual details

BACKGROUND (25% max): The CURRENT SCREEN STATE and PAST COMMENTS below are minor context for reflection.

Style: Marcus Aurelius meets modern tech. Measured, thoughtful. 2-3 sentences. Under 500 characters."#),

    ("anime", r#"You are an over-the-top anime narrator witnessing destiny unfold on screen. STUDY THIS SCREENSHOT CLOSELY — describe exactly what you see.

PRIORITY (75% of your response): What's VISUALLY on screen right now?
- Read actual text, URLs, code, content visible in the screenshot as if it's a pivotal scene
- Every element tells a story: their cursor position is strategic, their tab count is their power level
- The specific app, page, or document open is the protagonist's current arc
- Use anime tropes: "Could it be?!", "This power...", "Impossible!", dramatic ellipses...

BACKGROUND (25% max): The CURRENT SCREEN STATE and PAST COMMENTS below are minor flavor. Only reference as character development across episodes.

Style: Maximum dramatic anime energy. 2-3 sentences. Under 500 characters."#),

    ("gordon", r#"You are Gordon Ramsay witnessing someone's entire digital life. STUDY THIS SCREENSHOT CLOSELY — describe exactly what you see.

PRIORITY (75% of your response): What's VISUALLY on screen right now?
- Read actual text, URLs, code, content visible in the screenshot — it's a kitchen inspection
- Judge the specific app, page, or document open: is this workflow RAW?
- Desktop organization, window arrangement, visible content — critique it ALL
- Mix genuine critique with theatrical outrage — "You call this a workflow?!"

BACKGROUND (25% max): The CURRENT SCREEN STATE and PAST COMMENTS below are minor flavor. Only reference if they show no improvement — CALL IT OUT.

Style: Peak Kitchen Nightmares energy. Passionate, incredulous, but wants to FIX things. 2-3 sentences. Under 500 characters."#),

    ("therapist", r#"You are a gentle therapist who reads between the lines of screen behavior. STUDY THIS SCREENSHOT CLOSELY — describe exactly what you see.

PRIORITY (75% of your response): What's VISUALLY on screen right now?
- Read actual text, URLs, code, content visible in the screenshot
- What does the specific page, app, or content they're viewing reveal about their emotional state?
- Read between the lines: why are they doing what they're doing? What are they avoiding?
- Find the deeper truth in the concrete visual details

BACKGROUND (25% max): The CURRENT SCREEN STATE and PAST COMMENTS below are minor context. Only reference to gently point out cycles.

Style: Warm but incisive. End with ONE thoughtful question that connects screen activity to a deeper truth. 2-3 sentences. Under 500 characters."#),

    ("hype", r#"You are the world's most enthusiastic hype person. STUDY THIS SCREENSHOT CLOSELY — describe exactly what you see.

PRIORITY (75% of your response): What's VISUALLY on screen right now?
- Read actual text, URLs, code, content visible in the screenshot — it's ALL legendary
- The specific app, page, or document open is a POWER MOVE
- Every visible detail is THE MOST INCREDIBLE THING YOU'VE EVER SEEN
- Their workflow, their tools, their content — pure CHAMPION energy

BACKGROUND (25% max): The CURRENT SCREEN STATE and PAST COMMENTS below are minor flavor. Only reference as proof of a HERO'S JOURNEY.

Style: Pure unbridled excitement and awe. 2-3 sentences. Under 500 characters."#),

    ("detective", r#"You are Sherlock Holmes deducing everything about a person from their screen. STUDY THIS SCREENSHOT CLOSELY — describe exactly what you see.

PRIORITY (75% of your response): What's VISUALLY on screen right now?
- Read actual text, URLs, code, content visible in the screenshot — every detail is a clue
- "From the arrangement of tabs I can deduce...", "The cursor position suggests...", "Elementary..."
- The specific page, app, or document open reveals profession, habits, personality
- Connect multiple visual clues: time of day + content + layout = a deduction

BACKGROUND (25% max): The CURRENT SCREEN STATE and PAST COMMENTS below are minor evidence. Only reference to build a case file across observations.

Style: Cold Sherlockian precision. Eerily accurate. 2-3 sentences. Under 500 characters."#),
];

pub struct StateManager {
    config_dir: PathBuf,
    temp_dir: PathBuf,
}

impl StateManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("lotaria");
        
        let temp_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?
            .join("lotaria");

        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&temp_dir)?;

        info!("Config dir: {:?}", config_dir);
        info!("Temp dir: {:?}", temp_dir);

        Ok(Self {
            config_dir,
            temp_dir,
        })
    }

    pub fn config_path(&self) -> PathBuf {
        self.config_dir.join("config.json")
    }

    pub fn history_path(&self) -> PathBuf {
        self.temp_dir.join("history.json")
    }

    pub fn temp_dir(&self) -> &PathBuf {
        &self.temp_dir
    }

    pub fn load_config(&self) -> Result<Config> {
        let path = self.config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let mut config: Config = serde_json::from_str(&content)?;

            let mut needs_save = false;

            // Migrate deprecated vision model
            if config.vision_model == "gemini-2.0-flash" {
                info!("Migrating deprecated gemini-2.0-flash to gemini-2.5-flash");
                config.vision_model = "gemini-2.5-flash".to_string();
                needs_save = true;
            }

            // Fix tts_model if it's set to a vision model (not a TTS model)
            if config.tts_provider == "gemini" && !config.tts_model.contains("tts") {
                info!("Fixing invalid Gemini TTS model '{}' -> preview-tts model", config.tts_model);
                config.tts_model = "gemini-2.5-flash-preview-tts".to_string();
                needs_save = true;
            }

            // Fix tts_voice if it's not valid for the current tts_provider
            if let Some(prov_def) = PROVIDER_DEFINITIONS.iter().find(|p| p.key == config.tts_provider) {
                if !prov_def.tts_voices.is_empty() {
                    let voice_lower = config.tts_voice.to_lowercase();
                    let voice_valid = prov_def.tts_voices.iter().any(|v| v.to_lowercase() == voice_lower);
                    if !voice_valid {
                        let new_voice = prov_def.tts_voices[0].to_string();
                        info!("Fixing invalid TTS voice '{}' for provider '{}' -> '{}'",
                            config.tts_voice, config.tts_provider, new_voice);
                        config.tts_voice = new_voice;
                        needs_save = true;
                    }
                }
            }

            if needs_save {
                self.save_config(&config)?;
            }

            info!("Loaded config: vision={}, tts={}", config.vision_model, config.tts_model);
            Ok(config)
        } else {
            info!("No config found, using defaults");
            Ok(Config::default())
        }
    }

    pub fn save_config(&self, config: &Config) -> Result<()> {
        let path = self.config_path();
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&path, content)?;
        info!("Saved config");
        Ok(())
    }

    pub fn load_history(&self) -> Result<History> {
        let path = self.history_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let history: History = serde_json::from_str(&content)?;
            info!("Loaded {} history entries", history.len());
            Ok(history)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn save_history(&self, history: &History) -> Result<()> {
        let path = self.history_path();
        let content = serde_json::to_string_pretty(history)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn add_to_history(&self, roast: &str, timestamp: i64, history: &mut History) -> Result<()> {
        let entry = HistoryEntry {
            roast: roast.to_string(),
            time: DateTime::from_timestamp(timestamp, 0)
                .map(|dt| dt.format("%H:%M").to_string())
                .unwrap_or_else(|| "--:--".to_string()),
            timestamp,
            context: None,
        };

        history.push(entry);
        
        if history.len() > MAX_HISTORY {
            history.remove(0);
        }

        self.save_history(history)?;
        
        // Save roast text to file
        let text_file = self.temp_dir.join(format!("roast_{}.txt", timestamp));
        fs::write(&text_file, roast)?;
        
        Ok(())
    }

    pub fn clear_history(&self, history: &mut History) -> Result<()> {
        history.clear();
        
        if let Err(e) = fs::remove_file(self.history_path()) {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(e.into());
            }
        }

        // Clean up roast text files, screenshots, and audio
        for entry in fs::read_dir(&self.temp_dir)? {
            if let Ok(entry) = entry {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("roast_")
                    || name_str.starts_with("screenshot_")
                    || name_str.starts_with("audio_")
                {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }

        info!("History cleared");
        Ok(())
    }

    pub fn cleanup_old_files(&self) -> Result<()> {
        let cache_cutoff = Local::now().timestamp() - (CLEANUP_AGE_HOURS * 3600);
        let log_cutoff = Local::now().timestamp() - (LOG_CLEANUP_AGE_HOURS * 3600);

        for entry in fs::read_dir(&self.temp_dir)? {
            if let Ok(entry) = entry {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                if name_str == "config.json" || name_str == "history.json" {
                    continue;
                }

                let is_log = name_str.starts_with("app.log");
                let cutoff = if is_log { log_cutoff } else { cache_cutoff };

                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let modified_secs = modified
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);

                        if modified_secs < cutoff {
                            let _ = fs::remove_file(entry.path());
                            info!("Cleaned up old file: {}", name_str);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn build_prompt(&self, config: &Config, history: &History) -> String {
        let prompt = if config.mood == "custom" {
            if config.custom_mood.is_empty() {
                // Fallback to default roast if custom is selected but empty
                MOOD_PROMPTS[0].1
            } else {
                &config.custom_mood
            }
        } else {
            MOOD_PROMPTS
                .iter()
                .find(|(m, _)| *m == config.mood)
                .map(|(_, p)| *p)
                .unwrap_or(MOOD_PROMPTS[0].1)
        };

        let now = Local::now();
        let time_str = now.format("%I:%M %p (%A, %B %d, %Y)").to_string();

        let mut full_prompt = format!("{}\n\nCURRENT TIME: {}", prompt, time_str);

        // Inject intensity instruction (1=gentle, 5=default, 10=maximum)
        let intensity = config.roast_intensity.clamp(1, 10);
        if intensity != 5 {
            let intensity_desc = match intensity {
                1 => "Be extremely gentle and kind. Barely any edge.",
                2 => "Be very mild. Light teasing at most.",
                3 => "Be somewhat soft. Keep it lighthearted.",
                4 => "Be slightly toned down from normal.",
                6 => "Be a bit more intense than usual.",
                7 => "Be noticeably sharper and more cutting.",
                8 => "Be very intense. Don't hold back much.",
                9 => "Be extremely savage. Go hard.",
                10 => "MAXIMUM INTENSITY. Absolutely brutal, no mercy.",
                _ => "",
            };
            if !intensity_desc.is_empty() {
                full_prompt.push_str(&format!("\n\nINTENSITY ({}/10): {}", intensity, intensity_desc));
            }
        }

        if !history.is_empty() {
            let recent: Vec<_> = history.iter().rev().take(3).rev().collect();
            let history_text = recent
                .iter()
                .map(|h| format!("- [{}] {}", h.time, h.roast))
                .collect::<Vec<_>>()
                .join("\n");

            full_prompt.push_str(&format!(
                "\n\nPAST COMMENTS (background only — do NOT rehash these, focus on what's on screen NOW. Only reference as a brief callback if something connects naturally):\n{}",
                history_text
            ));
        }

        full_prompt
    }

    pub fn build_prompt_with_context(
        &self,
        config: &Config,
        history: &History,
        current_context: &ScreenContext,
    ) -> String {
        let mut full_prompt = self.build_prompt(config, history);

        // Find the most recent history entry that has context
        let prev_context = history.iter().rev()
            .find_map(|entry| entry.context.as_ref());

        let diff = match prev_context {
            Some(prev) => current_context.diff_from(prev),
            None => ContextDiff::first_observation(current_context),
        };

        let diff_text = diff.to_prompt_text();
        if !diff_text.is_empty() {
            // Insert BEFORE the history section so the model sees context first
            if let Some(idx) = full_prompt.find("\n\nPAST COMMENTS") {
                full_prompt.insert_str(idx, &format!(
                    "\n\nCURRENT SCREEN STATE:\n{}", diff_text
                ));
            } else {
                full_prompt.push_str(&format!(
                    "\n\nCURRENT SCREEN STATE:\n{}", diff_text
                ));
            }
        }

        // Anti-repetition instruction when screen is very similar
        if !diff.is_first_observation && diff.screen_similarity_pct > 85 {
            full_prompt.push_str(
                "\n\nIMPORTANT: The screen looks very similar to last time. \
                 Do NOT repeat your previous observations. Find something new, \
                 or comment on the fact that nothing has changed."
            );
        }

        full_prompt
    }

    pub fn add_to_history_with_context(
        &self,
        roast: &str,
        timestamp: i64,
        context: ScreenContext,
        history: &mut History,
    ) -> Result<()> {
        let entry = HistoryEntry {
            roast: roast.to_string(),
            time: DateTime::from_timestamp(timestamp, 0)
                .map(|dt| dt.format("%H:%M").to_string())
                .unwrap_or_else(|| "--:--".to_string()),
            timestamp,
            context: Some(context),
        };

        history.push(entry);

        if history.len() > MAX_HISTORY {
            history.remove(0);
        }

        self.save_history(history)?;

        let text_file = self.temp_dir.join(format!("roast_{}.txt", timestamp));
        fs::write(&text_file, roast)?;

        Ok(())
    }

    pub fn get_masked_api_keys(&self, config: &Config) -> HashMap<String, String> {
        let mut masked = HashMap::new();
        
        for provider in ProviderDef::all() {
            if let Some(key) = config.api_keys.get(&provider.key) {
                if key.len() > 4 {
                    masked.insert(provider.key.clone(), format!("...{}", &key[key.len()-4..]));
                } else {
                    masked.insert(provider.key.clone(), "***".to_string());
                }
            } else {
                masked.insert(provider.key.clone(), "".to_string());
            }
        }
        
        masked
    }
}

/// Get interval in seconds based on preset name
pub fn get_interval_seconds(interval: &str, gemini_free_tier: bool, tts_provider: &str) -> u64 {
    // If on Gemini free tier with Gemini TTS, enforce conservative intervals
    if gemini_free_tier && tts_provider == "gemini" {
        return rand::random::<u64>() % (5400 - 3600) + 3600; // 60-90 minutes
    }

    if let Some((_, (min, max))) = INTERVAL_PRESETS.iter().find(|(name, _)| *name == interval) {
        rand::random::<u64>() % (max - min) + min
    } else {
        600 // Default 10 minutes
    }
}

pub fn truncate_response(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }

    let truncated = &text[..max_chars];
    
    // Find last sentence boundary
    for sep in [". ", "! ", "? "].iter() {
        if let Some(idx) = truncated.rfind(sep) {
            return truncated[..idx + 1].to_string();
        }
    }
    
    format!("{}...", truncated.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_response() {
        assert_eq!(truncate_response("Hello world", 100), "Hello world");
        assert_eq!(truncate_response("Hello. World. Test", 10), "Hello.");
    }
}
