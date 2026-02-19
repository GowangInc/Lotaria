import json
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

# Model identifiers
LOCAL_VISION_MODEL = "Qwen/Qwen3-VL-2B-Instruct"
LOCAL_TTS_MODEL = "piper"
API_VISION_MODEL = "gemini-2.0-flash"
API_TTS_MODEL = "gemini-2.5-flash-preview-tts"

# ---------------------------------------------------------------------------
# Config & history dicts (mutable singletons, imported by other modules)
# ---------------------------------------------------------------------------

config = {
    "is_active": False,
    "interval": 300,
    "vision_model_type": "local",   # "local" | "api"
    "tts_model_type": "local",      # "local" | "api"
    "speech_bubble_enabled": True,
    "audio_enabled": True,
}

history: list[dict] = []

# ---------------------------------------------------------------------------
# Persistence helpers
# ---------------------------------------------------------------------------

def load_config():
    if CONFIG_FILE.exists():
        try:
            with open(CONFIG_FILE) as f:
                saved = json.load(f)
            for key in ("interval", "vision_model_type", "tts_model_type",
                        "speech_bubble_enabled", "audio_enabled"):
                if key in saved:
                    config[key] = saved[key]
            print(f"[Lotaria] Loaded config: vision={config['vision_model_type']}, tts={config['tts_model_type']}")
        except Exception as e:
            print(f"[Lotaria] Failed to load config: {e}")


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
