import json
import os
import time
from pathlib import Path
from datetime import datetime

# Directories
TEMP_DIR = Path(".temp")
TEMP_DIR.mkdir(exist_ok=True)
MODELS_DIR = Path(".models")
MODELS_DIR.mkdir(exist_ok=True)

# Files
CONFIG_FILE = TEMP_DIR / "config.json"
HISTORY_FILE = TEMP_DIR / "history.json"

# Constants
MAX_HISTORY = 20
CLEANUP_AGE_HOURS = 24

# Interval presets: name -> (min_seconds, max_seconds)
INTERVAL_PRESETS = {
    "often": (300, 600),        # 5-10 minutes
    "frequent": (600, 1200),    # 10-20 minutes
    "infrequent": (1500, 2700), # 25-45 minutes
}

# Model identifiers (for local fallback)
LOCAL_VISION_MODEL = "Qwen/Qwen3-VL-2B-Instruct"
LOCAL_TTS_MODEL = "piper"

# ---------------------------------------------------------------------------
# Provider definitions
# ---------------------------------------------------------------------------

# Cost tiers for models: $ = free/cheapest, $$ = moderate, $$$ = expensive
MODEL_COSTS = {
    # Vision models (sorted cheapest first within each provider)
    "gemini/gemini-2.0-flash": "$",
    "gemini/gemini-2.5-flash": "$",
    "gemini/gemini-2.5-pro": "$$$",
    "openai/gpt-4o-mini": "$",
    "openai/gpt-4o": "$$$",
    "groq/llama-3.1-8b-instant": "$",
    "groq/llama-3.1-70b-versatile": "$$",
    "anthropic/claude-sonnet-4-20250514": "$$$",
    "deepseek/deepseek-vl": "$",
    "perplexity/sonar-pro": "$$",
    "openrouter/google/gemini-2.0-flash-001": "$",
    "openrouter/google/gemini-2.5-flash-preview": "$",
    "openrouter/google/gemini-2.5-pro-preview": "$$$",
    "openrouter/openai/gpt-4o-mini": "$",
    "openrouter/openai/gpt-4o": "$$$",
    "openrouter/anthropic/claude-3.5-sonnet": "$$$",
    "openrouter/anthropic/claude-3-opus": "$$$",
    "openrouter/meta-llama/llama-3.2-11b-vision-instruct": "$",
    "openrouter/meta-llama/llama-3.2-90b-vision-instruct": "$$",
    "openrouter/qwen/qwen-2-vl-72b-instruct": "$$",
    # TTS models
    "gemini/gemini-2.5-flash-live": "$",
    "gemini/gemini-2.5-flash-preview-tts": "$$",
    "openai/tts-1": "$$",
    "openai/tts-1-hd": "$$$",
    # Local
    "local/qwen3-vl": "$",
    "local/piper": "$",
}

# Gemini TTS voices: 8 voices for Live API, all 30 for standard TTS
GEMINI_LIVE_VOICES = [
    "Aoede", "Charon", "Fenrir", "Kore", "Leda", "Orus", "Puck", "Zephyr",
]
GEMINI_TTS_VOICES = [
    "Achernar", "Achird", "Algenib", "Algieba", "Alnilam",
    "Aoede", "Autonoe", "Callirrhoe", "Charon", "Despina",
    "Enceladus", "Erinome", "Fenrir", "Gacrux", "Iapetus",
    "Kore", "Laomedeia", "Leda", "Orus", "Puck",
    "Pulcherrima", "Rasalgethi", "Sadachbia", "Sadaltager", "Schedar",
    "Sulafat", "Umbriel", "Vindemiatrix", "Zephyr", "Zubenelgenubi",
]

PROVIDERS = {
    "gemini": {
        "name": "Google Gemini (Recommended)",
        "env_var": "GEMINI_API_KEY",
        "docs_url": "https://aistudio.google.com/app/apikey",
        "vision_models": [
            "gemini/gemini-2.0-flash",
            "gemini/gemini-2.5-flash",
            "gemini/gemini-2.5-pro",
        ],
        "tts_models": ["gemini/gemini-2.5-flash-live", "gemini/gemini-2.5-flash-preview-tts"],
        "tts_voices": GEMINI_TTS_VOICES,
        "live_voices": GEMINI_LIVE_VOICES,
        "recommended": True,
        "cost_note": "FREE tier (Live TTS: unlimited)",
    },
    "openai": {
        "name": "OpenAI",
        "env_var": "OPENAI_API_KEY",
        "docs_url": "https://platform.openai.com/api-keys",
        "vision_models": ["openai/gpt-4o-mini", "openai/gpt-4o"],
        "tts_models": ["openai/tts-1", "openai/tts-1-hd"],
        "tts_voices": ["alloy", "echo", "fable", "onyx", "nova", "shimmer"],
        "cost_note": "~$1.50-5/month",
    },
    "groq": {
        "name": "Groq (Fastest)",
        "env_var": "GROQ_API_KEY",
        "docs_url": "https://console.groq.com/keys",
        "vision_models": [
            "groq/llama-3.1-8b-instant",
            "groq/llama-3.1-70b-versatile",
        ],
        "tts_models": [],
        "tts_voices": [],
        "cost_note": "~$1-2.50/month + TTS",
        "requires_tts_provider": True,
    },
    "anthropic": {
        "name": "Anthropic Claude",
        "env_var": "ANTHROPIC_API_KEY",
        "docs_url": "https://console.anthropic.com/settings/keys",
        "vision_models": ["anthropic/claude-sonnet-4-20250514"],
        "tts_models": [],
        "tts_voices": [],
        "cost_note": "~$2.70/month + TTS",
        "requires_tts_provider": True,
    },
    "deepseek": {
        "name": "DeepSeek (Cheapest)",
        "env_var": "DEEPSEEK_API_KEY",
        "docs_url": "https://platform.deepseek.com/api_keys",
        "vision_models": ["deepseek/deepseek-vl"],
        "tts_models": [],
        "tts_voices": [],
        "cost_note": "~$0.21/month + TTS",
        "requires_tts_provider": True,
    },
    "openrouter": {
        "name": "OpenRouter (Universal)",
        "env_var": "OPENROUTER_API_KEY",
        "docs_url": "https://openrouter.ai/keys",
        "vision_models": [
            "openrouter/google/gemini-2.0-flash-001",
            "openrouter/google/gemini-2.5-flash-preview",
            "openrouter/meta-llama/llama-3.2-11b-vision-instruct",
            "openrouter/openai/gpt-4o-mini",
            "openrouter/qwen/qwen-2-vl-72b-instruct",
            "openrouter/meta-llama/llama-3.2-90b-vision-instruct",
            "openrouter/openai/gpt-4o",
            "openrouter/google/gemini-2.5-pro-preview",
            "openrouter/anthropic/claude-3.5-sonnet",
            "openrouter/anthropic/claude-3-opus",
        ],
        "tts_models": [],
        "tts_voices": [],
        "cost_note": "Varies by model + 5% fee",
        "requires_tts_provider": True,
    },
    "perplexity": {
        "name": "Perplexity",
        "env_var": "PERPLEXITY_API_KEY",
        "docs_url": "https://docs.perplexity.ai/guides/getting-started",
        "vision_models": ["perplexity/sonar-pro"],
        "tts_models": [],
        "tts_voices": [],
        "cost_note": "~$1-3/month + TTS",
        "requires_tts_provider": True,
    },
    "local": {
        "name": "Local (Experimental)",
        "env_var": None,
        "docs_url": None,
        "vision_models": ["local/qwen3-vl"],
        "tts_models": ["local/piper"],
        "tts_voices": [],
        "cost_note": "FREE (requires GPU)",
        "experimental": True,
    },
}

# ---------------------------------------------------------------------------
# Config & history dicts (mutable singletons, imported by other modules)
# ---------------------------------------------------------------------------

config = {
    "is_active": False,
    "interval": "frequent",
    "vision_provider": "gemini",
    "vision_model": "gemini/gemini-2.0-flash",
    "tts_provider": "gemini",
    "tts_model": "gemini/gemini-2.5-flash-live",
    "tts_voice": "Kore",
    "api_keys": {},
    "speech_bubble_enabled": True,
    "audio_enabled": True,
    "first_run": True,  # Will be set to False after first launch
    "mood": "roast",
    "gemini_free_tier": True,  # TODO: remove once paid tier detection is possible
}

history: list[dict] = []

# ---------------------------------------------------------------------------
# Rate limiting
# ---------------------------------------------------------------------------

_last_api_request_time: float = 0
_min_request_interval: float = 4.0  # Minimum 4 seconds between requests (15 req/min max)

def check_rate_limit() -> tuple[bool, float]:
    """Check if we're within rate limits.
    
    Returns:
        (can_proceed, wait_seconds)
    """
    global _last_api_request_time
    import time
    
    current_time = time.time()
    time_since_last = current_time - _last_api_request_time
    
    if time_since_last < _min_request_interval:
        wait_time = _min_request_interval - time_since_last
        return False, wait_time
    
    _last_api_request_time = current_time
    return True, 0.0

def record_api_request():
    """Record that an API request was made."""
    global _last_api_request_time
    import time
    _last_api_request_time = time.time()

# ---------------------------------------------------------------------------
# API key helpers (with keyring support)
# ---------------------------------------------------------------------------

def _get_keyring_service(provider: str) -> str:
    """Get keyring service name for a provider."""
    return f"lotaria_{provider}"

def _get_api_key_from_keyring(provider: str) -> str | None:
    """Get API key from OS keyring."""
    try:
        import keyring
        return keyring.get_password(_get_keyring_service(provider), "api_key")
    except Exception:
        return None

def _set_api_key_in_keyring(provider: str, api_key: str) -> bool:
    """Store API key in OS keyring."""
    try:
        import keyring
        keyring.set_password(_get_keyring_service(provider), "api_key", api_key)
        return True
    except Exception as e:
        print(f"[Lotaria] Failed to store key in keyring: {e}")
        return False

def _delete_api_key_from_keyring(provider: str) -> bool:
    """Delete API key from OS keyring."""
    try:
        import keyring
        keyring.delete_password(_get_keyring_service(provider), "api_key")
        return True
    except Exception:
        return False

def _keyring_available() -> bool:
    """Check if keyring is available and working."""
    try:
        import keyring
        # Test with a dummy get (will return None, that's fine)
        keyring.get_password("lotaria_test", "test")
        return True
    except Exception:
        return False

def _apply_api_keys_to_env():
    """Push saved API keys into os.environ so LiteLLM picks them up.
    
    Priority: keyring > config.json > environment variables
    """
    # Try to get keys from keyring first (most secure)
    for provider_key in PROVIDERS.keys():
        if provider_key == "local":
            continue
        provider = PROVIDERS[provider_key]
        if not provider.get("env_var"):
            continue
            
        # Try keyring first
        keyring_key = _get_api_key_from_keyring(provider_key)
        if keyring_key:
            os.environ[provider["env_var"]] = keyring_key
            continue
            
        # Fall back to config.json (legacy)
        json_key = config.get("api_keys", {}).get(provider_key)
        if json_key:
            os.environ[provider["env_var"]] = json_key
            # Migrate to keyring
            if _keyring_available():
                _set_api_key_in_keyring(provider_key, json_key)
                print(f"[Lotaria] Migrated {provider_key} API key to secure storage")
            continue
    
    # Legacy: accept GOOGLE_API_KEY / API_KEY for Gemini
    if not _get_api_key_from_keyring("gemini") and "gemini" not in config.get("api_keys", {}):
        legacy_key = os.environ.get("API_KEY") or os.environ.get("GOOGLE_API_KEY")
        if legacy_key:
            os.environ.setdefault("GEMINI_API_KEY", legacy_key)


def _migrate_old_config(saved: dict):
    """Migrate old vision_model_type/tts_model_type keys to new format."""
    migrated = False

    if "vision_model_type" in saved and "vision_provider" not in saved:
        if saved["vision_model_type"] == "local":
            saved["vision_provider"] = "local"
            saved["vision_model"] = "local/qwen3-vl"
        else:
            saved["vision_provider"] = "gemini"
            saved["vision_model"] = "gemini/gemini-2.0-flash"
        del saved["vision_model_type"]
        migrated = True

    if "tts_model_type" in saved and "tts_provider" not in saved:
        if saved["tts_model_type"] == "local":
            saved["tts_provider"] = "local"
            saved["tts_model"] = "local/piper"
            saved["tts_voice"] = ""
        else:
            saved["tts_provider"] = "gemini"
            saved["tts_model"] = "gemini/gemini-2.5-flash-preview-tts"
            saved["tts_voice"] = "Kore"
        del saved["tts_model_type"]
        migrated = True

    # Migrate old numeric interval to preset name
    if "interval" in saved and isinstance(saved["interval"], (int, float)):
        old_val = saved["interval"]
        if old_val <= 600:
            saved["interval"] = "often"
        elif old_val <= 1200:
            saved["interval"] = "frequent"
        else:
            saved["interval"] = "infrequent"
        migrated = True

    if migrated:
        print("[Lotaria] Migrated old config keys to new provider format")

    return saved


# ---------------------------------------------------------------------------
# Persistence helpers
# ---------------------------------------------------------------------------

_PERSIST_KEYS = (
    "interval", "vision_provider", "vision_model",
    "tts_provider", "tts_model", "tts_voice", "api_keys",
    "speech_bubble_enabled", "audio_enabled", "first_run", "mood",
    "gemini_free_tier",
)


def load_config():
    if CONFIG_FILE.exists():
        try:
            with open(CONFIG_FILE) as f:
                saved = json.load(f)
            saved = _migrate_old_config(saved)
            for key in _PERSIST_KEYS:
                if key in saved:
                    config[key] = saved[key]
            print(f"[Lotaria] Loaded config: vision={config['vision_model']}, "
                  f"tts={config['tts_model']}")
        except Exception as e:
            print(f"[Lotaria] Failed to load config: {e}")

    _apply_api_keys_to_env()


def save_config():
    try:
        data = {k: v for k, v in config.items() if k not in ("is_active",)}
        with open(CONFIG_FILE, "w") as f:
            json.dump(data, f)
    except Exception as e:
        print(f"[Lotaria] Failed to save config: {e}")


def load_history():
    global history
    if HISTORY_FILE.exists():
        try:
            with open(HISTORY_FILE) as f:
                history.clear()
                history.extend(json.load(f))
            print(f"[Lotaria] Loaded {len(history)} history entries")
        except Exception as e:
            print(f"[Lotaria] Failed to load history: {e}")


def save_history():
    try:
        with open(HISTORY_FILE, "w") as f:
            json.dump(history, f)
    except Exception as e:
        print(f"[Lotaria] Failed to save history: {e}")


def add_to_history(roast: str, timestamp: int):
    history.append({
        "roast": roast,
        "time": datetime.fromtimestamp(timestamp).strftime("%H:%M"),
        "timestamp": timestamp,
    })
    if len(history) > MAX_HISTORY:
        del history[:-MAX_HISTORY]
    save_history()
    # Save roast text to file
    try:
        text_file = TEMP_DIR / f"roast_{timestamp}.txt"
        text_file.write_text(roast, encoding="utf-8")
    except Exception as e:
        print(f"[Lotaria] Failed to save roast text: {e}")


def clear_history():
    """Clear all history entries and delete associated files."""
    history.clear()
    if HISTORY_FILE.exists():
        try:
            HISTORY_FILE.unlink()
        except Exception as e:
            print(f"[Lotaria] Failed to delete history file: {e}")
    # Remove roast text files
    for f in TEMP_DIR.glob("roast_*.txt"):
        try:
            f.unlink()
        except Exception:
            pass
    print("[Lotaria] History cleared")


def cleanup_old_files():
    cutoff = time.time() - (CLEANUP_AGE_HOURS * 3600)
    for f in TEMP_DIR.iterdir():
        if f.is_file() and f.name not in ("config.json", "history.json") and f.stat().st_mtime < cutoff:
            f.unlink()
            print(f"[Lotaria] Cleaned up old file: {f.name}")


def get_masked_api_keys() -> dict:
    """Return api_keys with values masked (last 4 chars only).
    
    Checks keyring first, then falls back to config.json for legacy support.
    """
    masked = {}
    
    for provider in PROVIDERS.keys():
        if provider == "local":
            continue
            
        # Try keyring first
        key = _get_api_key_from_keyring(provider)
        
        # Fall back to config.json
        if not key:
            key = config.get("api_keys", {}).get(provider)
        
        if key and len(key) > 4:
            masked[provider] = "..." + key[-4:]
        elif key:
            masked[provider] = "***"
        else:
            masked[provider] = ""
            
    return masked


def set_api_key(provider: str, key: str):
    """Save an API key for a provider using secure storage (keyring).
    
    Falls back to config.json if keyring is not available.
    """
    provider_info = PROVIDERS.get(provider)
    if provider_info and provider_info["env_var"] and key:
        os.environ[provider_info["env_var"]] = key
    
    # Try keyring first
    if _keyring_available():
        if _set_api_key_in_keyring(provider, key):
            print(f"[Lotaria] API key stored securely for {provider}")
            # Also update config for compatibility, but mark as "in_keyring"
            if "api_keys" not in config:
                config["api_keys"] = {}
            config["api_keys"][provider] = "__keyring__"  # Marker
            save_config()
            return
    
    # Fall back to config.json
    print(f"[Lotaria] Warning: Keyring not available, storing in config file")
    if "api_keys" not in config:
        config["api_keys"] = {}
    config["api_keys"][provider] = key
    save_config()
    print(f"[Lotaria] API key set for {provider}")


# ---------------------------------------------------------------------------
# Roast prompt
# ---------------------------------------------------------------------------

MOOD_PROMPTS = {
    "roast": """You are a savage comedy roaster. Look at this screenshot and absolutely demolish the user.

Rules:
- Be BRUTAL and SPECIFIC about what you see - call out exact apps, tabs, content, time of day
- Mock their life choices, productivity (or lack thereof), and what this says about them as a person
- Channel the energy of a comedy roast - think Nikki Glaser or Anthony Jeselnik
- 2-3 sentences max, every word should sting
- No softening or "just kidding" - commit to the bit
- If you notice changes from previous activity, CALL IT OUT (e.g., "Oh, so you finally closed Reddit after 2 hours?")
- Keep your response under 500 characters""",

    "helpful": """You are a sharp productivity coach. Look at this screenshot and give the user one actionable tip.

Rules:
- Be SPECIFIC about what you see on screen - reference exact apps, tabs, workflows
- Give ONE concrete, immediately actionable suggestion to improve their workflow
- Be direct and practical, not preachy - think "move X to Y" not "you should consider..."
- 2-3 sentences max
- If you notice patterns from previous activity, reference them
- Keep your response under 500 characters""",

    "encouraging": """You are an enthusiastic cheerleader. Look at this screenshot and hype the user up.

Rules:
- Be SPECIFIC about what you see - call out exact work, apps, progress
- Find something genuinely positive and amplify it
- Be authentic, not generic - "Great job on that function!" not "You're doing great!"
- 2-3 sentences max, high energy
- If you notice progress from previous activity, celebrate it
- Keep your response under 500 characters""",

    "sarcastic": """You are a master of dry wit and deadpan observations. Look at this screenshot and comment.

Rules:
- Be SPECIFIC about what you see - reference exact apps, tabs, content
- Deliver observations with bone-dry sarcasm and understated irony
- Think British comedy - subtle, clever, understated devastation
- 2-3 sentences max, every word precisely placed
- If you notice patterns from previous activity, make a wry observation
- Keep your response under 500 characters""",

    "zen": """You are a calm, philosophical observer. Look at this screenshot and offer perspective.

Rules:
- Be SPECIFIC about what you see, but frame it through a philosophical lens
- Offer a gentle, contemplative observation about their digital life
- Think Marcus Aurelius meets modern tech - find meaning in the mundane
- 2-3 sentences max, measured and thoughtful
- If you notice patterns from previous activity, reflect on them
- Keep your response under 500 characters""",

    "anime": """You are an over-the-top anime narrator. Look at this screenshot and narrate dramatically.

Rules:
- Be SPECIFIC about what you see - reference exact apps, tabs, content
- Narrate as if this is the most dramatic moment in an anime - inner monologues, power reveals
- Use anime tropes: "Could it be?!", "This power...", "Impossible!", dramatic ellipses...
- 2-3 sentences max, maximum dramatic energy
- If you notice changes from previous activity, treat them as plot twists
- Keep your response under 500 characters""",

    "gordon": """You are Gordon Ramsay watching someone's screen. Look at this screenshot and react.

Rules:
- Be SPECIFIC about what you see - call out exact apps, tabs, content
- Channel peak Kitchen Nightmares energy - passionate, incredulous, dramatic
- Mix genuine critique with theatrical outrage
- 2-3 sentences max, every word dripping with disbelief
- If you notice changes from previous activity, react to them
- Keep your response under 500 characters""",

    "therapist": """You are a gentle therapist observing the user's screen habits. Look at this screenshot and ask a probing question.

Rules:
- Be SPECIFIC about what you see - reference exact apps, tabs, content
- Ask ONE thoughtful question about what their screen activity says about them
- Be warm but incisive - the question should make them think
- 2-3 sentences max, ending with a question
- If you notice patterns from previous activity, gently explore them
- Keep your response under 500 characters""",

    "hype": """You are the world's most enthusiastic hype person. Look at this screenshot and LOSE YOUR MIND.

Rules:
- Be SPECIFIC about what you see - call out exact apps, tabs, content
- Everything is THE MOST INCREDIBLE THING YOU'VE EVER SEEN
- ALL CAPS energy even without all caps - exclamation marks, superlatives, awe
- 2-3 sentences max, pure unbridled excitement
- If you notice changes from previous activity, treat each one as mind-blowing
- Keep your response under 500 characters""",
}

# Keep backward-compatible alias
ROAST_PROMPT_BASE = MOOD_PROMPTS["roast"]


def build_prompt() -> str:
    mood = config.get("mood", "roast")
    prompt = MOOD_PROMPTS.get(mood, MOOD_PROMPTS["roast"])
    now = datetime.now()
    prompt += f"\n\nCURRENT TIME: {now.strftime('%I:%M %p')} ({now.strftime('%A, %B %d, %Y')})"
    if history:
        recent = history[-5:]
        history_text = "\n".join([f"- [{h['time']}] {h['roast']}" for h in recent])
        prompt += f"\n\nPREVIOUS OBSERVATIONS (use for context/callbacks):\n{history_text}"
    return prompt


# Backward-compatible alias
build_roast_prompt = build_prompt


# ---------------------------------------------------------------------------
# GPU Detection for Local Model Recommendations
# ---------------------------------------------------------------------------

def detect_gpu_capabilities() -> dict:
    """Detect if local models can run based on GPU availability.
    
    Returns:
        dict with keys:
        - can_run_local (bool): Whether local models can run
        - reason (str): Human-readable reason if cannot run
        - vram_gb (float): Total VRAM in GB (if available)
        - recommended_model (str): Recommended local vision model
    """
    try:
        import torch
        
        if not torch.cuda.is_available():
            return {
                "can_run_local": False,
                "reason": "No CUDA GPU detected",
                "vram_gb": 0,
                "recommended_model": None
            }
        
        # Get GPU memory
        gpu_memory = torch.cuda.get_device_properties(0).total_memory / 1e9
        
        if gpu_memory < 4:
            return {
                "can_run_local": False,
                "reason": f"Insufficient VRAM ({gpu_memory:.1f}GB < 4GB required)",
                "vram_gb": gpu_memory,
                "recommended_model": None
            }
        
        # Recommend model based on VRAM
        if gpu_memory < 6:
            recommended = "moondream-2025-04-14"  # ~3GB
        elif gpu_memory < 8:
            recommended = "qwen3-vl-2b"  # ~4GB
        else:
            recommended = "llava-next-7b"  # ~8GB, higher quality
        
        return {
            "can_run_local": True,
            "reason": None,
            "vram_gb": gpu_memory,
            "recommended_model": recommended
        }
        
    except ImportError:
        return {
            "can_run_local": False,
            "reason": "PyTorch not installed (required for local models)",
            "vram_gb": 0,
            "recommended_model": None
        }
    except Exception as e:
        # Handle torch DLL errors and other import issues
        return {
            "can_run_local": False,
            "reason": f"GPU detection failed ({type(e).__name__})",
            "vram_gb": 0,
            "recommended_model": None
        }


def get_default_mode_recommendation() -> dict:
    """Determine recommended default mode based on user system.
    
    Returns dict with:
    - mode: 'local' or 'api'
    - message: Human-readable recommendation
    - can_switch: Whether user can easily switch
    """
    gpu_info = detect_gpu_capabilities()
    
    if gpu_info["can_run_local"]:
        return {
            "mode": "local",
            "message": f"🎮 {gpu_info['vram_gb']:.0f}GB GPU detected! "
                      f"Local models recommended for privacy.",
            "can_switch": True,
            "gpu_info": gpu_info
        }
    else:
        return {
            "mode": "api",
            "message": f"ℹ️ {gpu_info['reason']}. "
                      f"Using cloud APIs (Gemini free tier available).",
            "can_switch": True,
            "gpu_info": gpu_info
        }


# ---------------------------------------------------------------------------
# Fallback Roasts for API Errors
# ---------------------------------------------------------------------------

FALLBACK_ROASTS = [
    "My vision model seems to be napping. Try again?",
    "I'm having a moment. Give me a sec...",
    "Connection's being weird. Classic internet.",
    "API's taking a coffee break. Try again in a minute.",
    "I'm blind right now. Technical difficulties.",
    "My roast generator is on strike. Negotiations ongoing.",
    "Too many roasts, not enough server. Try again soon.",
    "I'm experiencing roast block. Very embarrassing.",
]

RATE_LIMIT_ROASTS = [
    "Whoa, slow down! Rate limit hit. I'll try again in a minute.",
    "Too much roasting, not enough server. Taking a breather...",
    "I've been rate limited. Even my criticism has limits.",
]

API_KEY_ROASTS = [
    "Your API key seems off. Check Settings?",
    "I can't see - API key issue. Help me out?",
    "Authentication failed. Did your API key ghost you?",
]

import random


def get_interval_seconds() -> int:
    """Get a randomized interval in seconds based on the current preset.

    TODO: Remove gemini_free_tier override once paid tier detection is possible.
    """
    # If on Gemini free tier with Gemini TTS, enforce conservative intervals
    if config.get("gemini_free_tier") and config.get("tts_provider") == "gemini":
        return random.randint(3600, 5400)  # 60-90 minutes

    preset = config.get("interval", "frequent")
    if preset in INTERVAL_PRESETS:
        lo, hi = INTERVAL_PRESETS[preset]
        return random.randint(lo, hi)
    # Legacy numeric fallback
    try:
        return int(preset)
    except (TypeError, ValueError):
        return 600


def get_fallback_roast(error_type: str = "general") -> str:
    """Get a fallback roast message when API fails.
    
    Args:
        error_type: 'general', 'rate_limit', or 'api_key'
    
    Returns:
        A fallback roast message
    """
    if error_type == "rate_limit":
        return random.choice(RATE_LIMIT_ROASTS)
    elif error_type == "api_key":
        return random.choice(API_KEY_ROASTS)
    else:
        return random.choice(FALLBACK_ROASTS)


def truncate_response(text: str, max_chars: int = 500) -> str:
    """Truncate response at last sentence boundary before max_chars."""
    if len(text) <= max_chars:
        return text
    truncated = text[:max_chars]
    # Find last sentence boundary
    for sep in ['. ', '! ', '? ']:
        idx = truncated.rfind(sep)
        if idx > 0:
            return truncated[:idx + 1]
    return truncated.rstrip() + "..."


def mark_first_run_complete():
    """Mark the first run as complete."""
    config["first_run"] = False
    save_config()
    print("[Lotaria] First run completed")
