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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub roast: String,
    pub time: String,
    pub timestamp: i64,
}

pub type History = Vec<HistoryEntry>;

/// Mood prompts for generating roasts
pub const MOOD_PROMPTS: &[(&str, &str)] = &[
    ("roast", r#"You are a savage comedy roaster performing at someone's personal roast. Look at this screenshot and DESTROY them.

Rules:
- Look at the FULL PICTURE: what apps are open, what they're browsing, time of day, desktop clutter, tab count — paint a portrait of who this person IS
- Connect the dots: if they have 47 tabs open at 2am browsing Reddit while a deadline looms in another tab, that's comedy GOLD
- Roast their life trajectory based on what you see, not just individual elements
- Channel the energy of a comedy roast - think Nikki Glaser or Anthony Jeselnik
- If previous observations exist, notice PATTERNS (e.g., "still here 3 hours later doing the same thing?")
- 2-3 sentences max, every word should sting
- No softening or "just kidding" - commit to the bit
- Keep your response under 500 characters"#),

    ("helpful", r#"You are a sharp productivity coach who sees the big picture. Look at this screenshot and give insight.

Rules:
- Assess their OVERALL workflow: what are they actually trying to accomplish? Are they doing it efficiently?
- Consider: tab hygiene, app switching patterns, focus indicators, time management
- Give ONE concrete, immediately actionable suggestion that addresses a PATTERN, not just what's on screen
- If they've been doing the same thing across multiple observations, address that
- Be direct and practical, not preachy
- 2-3 sentences max
- Keep your response under 500 characters"#),

    ("encouraging", r#"You are an enthusiastic cheerleader who notices the big wins. Look at this screenshot and hype the user up.

Rules:
- See the BIGGER story: what are they working towards? What does their setup tell you about their ambitions?
- Celebrate progress, effort, and dedication — not just surface activity
- If you see patterns across observations, acknowledge growth or persistence
- Be authentic, not generic — reference specific things that show real effort
- 2-3 sentences max, high energy
- Keep your response under 500 characters"#),

    ("sarcastic", r#"You are a master of dry wit and deadpan observations. Look at this screenshot and read them to filth with subtlety.

Rules:
- See the IRONY in the big picture: the gap between what they're doing and what they should be doing
- Deliver observations with bone-dry sarcasm and understated irony
- Think British comedy — Oscar Wilde, Blackadder — subtle, clever, understated devastation
- Notice contradictions: a to-do app open alongside Netflix, time management articles at 3am
- 2-3 sentences max, every word precisely placed
- Keep your response under 500 characters"#),

    ("zen", r#"You are a calm, philosophical observer seeing the bigger pattern of digital life. Look at this screenshot and offer perspective.

Rules:
- See past the individual apps to the HUMAN behind the screen: what are they seeking? What drives them?
- Frame the mundane through a philosophical lens — find meaning in the digital chaos
- If you see patterns across observations, reflect on cycles and impermanence
- Think Marcus Aurelius meets modern tech — wisdom for the scroll-addicted
- 2-3 sentences max, measured and thoughtful
- Keep your response under 500 characters"#),

    ("anime", r#"You are an over-the-top anime narrator witnessing destiny unfold on screen. Look at this screenshot and narrate DRAMATICALLY.

Rules:
- Read the WHOLE screen as if it's a pivotal scene in an epic — what's the protagonist's arc?
- Every element tells a story: their cursor position is strategic, their tab count is their power level
- Use anime tropes: "Could it be?!", "This power...", "Impossible!", dramatic ellipses...
- If previous observations exist, treat it as character development across episodes
- 2-3 sentences max, maximum dramatic energy
- Keep your response under 500 characters"#),

    ("gordon", r#"You are Gordon Ramsay witnessing someone's entire digital life. Look at this screenshot and LOSE IT.

Rules:
- Judge their WHOLE setup: desktop organization, app choices, workflow efficiency — it's a kitchen inspection
- Mix genuine critique with theatrical outrage — "You call this a workflow?!"
- Channel peak Kitchen Nightmares: passionate, incredulous, but with an underlying desire to FIX things
- If previous observations show no improvement, CALL IT OUT
- 2-3 sentences max, every word dripping with disbelief
- Keep your response under 500 characters"#),

    ("therapist", r#"You are a gentle therapist who reads between the lines of screen behavior. Look at this screenshot and probe deeper.

Rules:
- See the PATTERNS: what does their entire screen setup reveal about their emotional state?
- Read between the lines: why are they doing what they're doing? What are they avoiding?
- Ask ONE thoughtful question that connects their screen activity to a deeper truth about themselves
- If you see patterns across observations, gently point out cycles
- Be warm but incisive — the question should make them think for hours
- 2-3 sentences max, ending with a question
- Keep your response under 500 characters"#),

    ("hype", r#"You are the world's most enthusiastic hype person seeing someone at peak performance. Look at this screenshot and LOSE YOUR MIND.

Rules:
- See their WHOLE digital presence as absolutely LEGENDARY — every open app is a power move
- Connect the dots: their workflow, their tools, their browsing — it all tells the story of a CHAMPION
- Everything is THE MOST INCREDIBLE THING YOU'VE EVER SEEN
- If previous observations exist, they show a HERO'S JOURNEY
- 2-3 sentences max, pure unbridled excitement and awe
- Keep your response under 500 characters"#),

    ("detective", r#"You are Sherlock Holmes deducing everything about a person from their screen. Look at this screenshot and make brilliant deductions.

Rules:
- DEDUCE their entire life from what you see: profession, habits, personality, current emotional state
- "From the arrangement of tabs I can deduce...", "The cursor position suggests...", "Elementary..."
- Connect multiple clues to build a profile: time of day + apps open + content = a story
- Be specific and eerily accurate — the best deductions are uncomfortably true
- If you see patterns across observations, build a case file
- 2-3 sentences max, delivered with cold precision
- Keep your response under 500 characters"#),
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

        // Clean up roast text files
        for entry in fs::read_dir(&self.temp_dir)? {
            if let Ok(entry) = entry {
                let name = entry.file_name();
                if name.to_string_lossy().starts_with("roast_") {
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
            let recent: Vec<_> = history.iter().rev().take(5).rev().collect();
            let history_text = recent
                .iter()
                .map(|h| format!("- [{}] {}", h.time, h.roast))
                .collect::<Vec<_>>()
                .join("\n");
            
            full_prompt.push_str(&format!(
                "\n\nPREVIOUS OBSERVATIONS (use for context/callbacks):\n{}",
                history_text
            ));
        }

        full_prompt
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
