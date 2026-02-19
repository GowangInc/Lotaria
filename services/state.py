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

# Model identifiers (for local fallback)
LOCAL_VISION_MODEL = "Qwen/Qwen3-VL-2B-Instruct"
LOCAL_TTS_MODEL = "piper"

# ---------------------------------------------------------------------------
# Provider definitions
# ---------------------------------------------------------------------------

PROVIDERS = {
    "gemini": {
        "name": "Google Gemini",
        "env_var": "GEMINI_API_KEY",
        "vision_models": [
            "gemini/gemini-2.0-flash",
            "gemini/gemini-2.5-flash",
            "gemini/gemini-2.5-pro",
        ],
        "tts_models": ["gemini/gemini-2.5-flash-preview-tts"],
        "tts_voices": ["Kore", "Puck", "Charon", "Fenrir", "Aoede"],
    },
    "openai": {
        "name": "OpenAI",
        "env_var": "OPENAI_API_KEY",
        "vision_models": ["openai/gpt-4o", "openai/gpt-4o-mini"],
        "tts_models": ["openai/tts-1", "openai/tts-1-hd"],
        "tts_voices": ["alloy", "echo", "fable", "onyx", "nova", "shimmer"],
    },
    "anthropic": {
        "name": "Anthropic",
        "env_var": "ANTHROPIC_API_KEY",
        "vision_models": ["anthropic/claude-sonnet-4-20250514"],
        "tts_models": [],
        "tts_voices": [],
    },
    "openrouter": {
        "name": "OpenRouter",
        "env_var": "OPENROUTER_API_KEY",
        "vision_models": [
            "openrouter/google/gemini-2.0-flash-001",
            "openrouter/anthropic/claude-sonnet-4",
            "openrouter/openai/gpt-4o",
        ],
        "tts_models": [],
        "tts_voices": [],
    },
    "local": {
        "name": "Local",
        "env_var": None,
        "vision_models": ["local/qwen3-vl"],
        "tts_models": ["local/piper"],
        "tts_voices": [],
    },
}

# ---------------------------------------------------------------------------
# Config & history dicts (mutable singletons, imported by other modules)
# ---------------------------------------------------------------------------

config = {
    "is_active": False,
    "interval": 300,
    "vision_provider": "gemini",
    "vision_model": "gemini/gemini-2.0-flash",
    "tts_provider": "gemini",
    "tts_model": "gemini/gemini-2.5-flash-preview-tts",
    "tts_voice": "Kore",
    "api_keys": {},
    "speech_bubble_enabled": True,
    "audio_enabled": True,
}

history: list[dict] = []

# ---------------------------------------------------------------------------
# API key helpers
# ---------------------------------------------------------------------------

def _apply_api_keys_to_env():
    """Push saved API keys into os.environ so LiteLLM picks them up."""
    for provider_key, api_key in config.get("api_keys", {}).items():
        provider = PROVIDERS.get(provider_key)
        if provider and provider["env_var"] and api_key:
            os.environ[provider["env_var"]] = api_key

    # Also accept legacy GOOGLE_API_KEY / API_KEY for Gemini
    if "gemini" not in config.get("api_keys", {}):
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

    if migrated:
        print("[Lotaria] Migrated old config keys to new provider format")

    return saved


# ---------------------------------------------------------------------------
# Persistence helpers
# ---------------------------------------------------------------------------

_PERSIST_KEYS = (
    "interval", "vision_provider", "vision_model",
    "tts_provider", "tts_model", "tts_voice", "api_keys",
    "speech_bubble_enabled", "audio_enabled",
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
            print(f"[Lotaria] Loaded config: vision={config['vision_provider']}/{config['vision_model']}, "
                  f"tts={config['tts_provider']}/{config['tts_model']}")
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


def cleanup_old_files():
    cutoff = time.time() - (CLEANUP_AGE_HOURS * 3600)
    for f in TEMP_DIR.iterdir():
        if f.is_file() and f.name not in ("config.json", "history.json") and f.stat().st_mtime < cutoff:
            f.unlink()
            print(f"[Lotaria] Cleaned up old file: {f.name}")


def get_masked_api_keys() -> dict:
    """Return api_keys with values masked (last 4 chars only)."""
    masked = {}
    for provider, key in config.get("api_keys", {}).items():
        if key and len(key) > 4:
            masked[provider] = "..." + key[-4:]
        elif key:
            masked[provider] = "***"
        else:
            masked[provider] = ""
    return masked


def set_api_key(provider: str, key: str):
    """Save an API key for a provider and apply it to the environment."""
    if "api_keys" not in config:
        config["api_keys"] = {}
    config["api_keys"][provider] = key
    save_config()

    provider_info = PROVIDERS.get(provider)
    if provider_info and provider_info["env_var"] and key:
        os.environ[provider_info["env_var"]] = key
    print(f"[Lotaria] API key set for {provider}")


# ---------------------------------------------------------------------------
# Roast prompt
# ---------------------------------------------------------------------------

ROAST_PROMPT_BASE = """You are a savage comedy roaster. Look at this screenshot and absolutely demolish the user.

Rules:
- Be BRUTAL and SPECIFIC about what you see - call out exact apps, tabs, content, time of day
- Mock their life choices, productivity (or lack thereof), and what this says about them as a person
- Channel the energy of a comedy roast - think Nikki Glaser or Anthony Jeselnik
- 2-3 sentences max, every word should sting
- No softening or "just kidding" - commit to the bit
- If you notice changes from previous activity, CALL IT OUT (e.g., "Oh, so you finally closed Reddit after 2 hours?")"""


def build_roast_prompt() -> str:
    prompt = ROAST_PROMPT_BASE
    if history:
        recent = history[-5:]
        history_text = "\n".join([f"- [{h['time']}] {h['roast']}" for h in recent])
        prompt += f"\n\nPREVIOUS OBSERVATIONS (use for context/callbacks):\n{history_text}"
    return prompt
