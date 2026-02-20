# Lotaria - Kimi Agent Project Guide

## Project Overview

Lotaria is a desktop pet that sits on your screen as a transparent, always-on-top pixel art character. It periodically captures your screen, uses vision AI to analyze what you're doing, and delivers savage comedy roasts via speech bubble and text-to-speech audio.

**Core Tech Stack:**
- **Desktop**: pywebview with Qt backend (PyQt6 + QtWebEngine) — transparent, frameless, always-on-top
- **Screen Capture**: `mss` (multi-screen shot)
- **Vision**: LiteLLM for multi-provider support + Local Qwen3-VL option
- **TTS**: Gemini TTS via `google-genai` SDK, OpenAI TTS via LiteLLM, Local Piper
- **UI**: Single HTML file with inline CSS/JS (no build step)
- **Python**: 3.10+ (3.14+ requires Qt backend since pythonnet is unavailable)

---

## 1. Best Default Options (Works Out of the Box)

### Recommended Default Configuration

```python
# Default config in services/state.py
config = {
    "is_active": False,           # Start inactive, let user enable
    "interval": 300,              # 5 minutes between roasts
    "vision_provider": "gemini",   # Most reliable vision + TTS combo
    "vision_model": "gemini/gemini-2.0-flash",  # Fast, cheap, good vision
    "tts_provider": "gemini",      # Matches vision provider
    "tts_model": "gemini/gemini-2.5-flash-preview-tts",
    "tts_voice": "Kore",          # Default Gemini voice
    "api_keys": {},
    "speech_bubble_enabled": True,
    "audio_enabled": True,
}
```

### Why These Defaults?

| Setting | Choice | Reasoning |
|---------|--------|-----------|
| **Vision Provider** | Gemini | Best price/performance for vision; has native TTS |
| **Vision Model** | gemini-2.0-flash | Cheaper than 2.5-pro, faster, still excellent vision |
| **TTS Provider** | Gemini | Matched with vision; high quality voices |
| **TTS Voice** | Kore | Most natural-sounding default |
| **Interval** | 300s (5min) | Not too spammy, not too boring |
| **Start Active** | False | Respect user privacy; let them opt-in |

---

## 2. Python Environment Management

### Setup Script (Recommended)

Create `setup.bat` for Windows:

```batch
@echo off
echo Setting up Lotaria...

:: Check Python version
python --version >nul 2>&1
if errorlevel 1 (
    echo Error: Python not found. Please install Python 3.10 or higher.
    exit /b 1
)

:: Create virtual environment
if not exist ".venv" (
    echo Creating virtual environment...
    python -m venv .venv
)

:: Activate and install
call .venv\Scripts\activate.bat
echo Installing dependencies...
pip install -r requirements.txt

:: Check for .env
if not exist ".env" (
    copy .env.example .env
    echo.
    echo Please edit .env and add your API keys, then run: python app.py
) else (
    echo.
    echo Setup complete! Run: python app.py
)

pause
```

### Common Python Issues & Solutions

| Issue | Cause | Solution |
|-------|-------|----------|
| `pythonnet` fails on Python 3.14+ | No wheels for 3.14 | Use Qt backend (already forced in app.py with `gui="qt"`) |
| PyQt6 installation fails | Missing MSVC runtime | Install Visual C++ Redistributable |
| CUDA out of memory | Local model too big | Switch to API provider in settings |
| Piper TTS not found | Not in PATH | Use `pip install piper-tts` and ensure .venv is activated |
| Import errors | Wrong Python version | Ensure Python 3.10+ and virtual env is activated |
| Screen capture fails | Permissions | Run as administrator (Windows) |
| Audio not playing | Missing codec | Install ffmpeg or use WAV format |

### Virtual Environment Check

Always verify venv is active before running:

```python
# Add to top of app.py for diagnostics
import sys
print(f"Python: {sys.executable}")
print(f"Version: {sys.version}")
```

---

## 3. Best Local Models

### Vision Model: Qwen3-VL-2B-Instruct

```bash
# Requirements (already in requirements.txt)
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu126
transformers>=4.40.0
accelerate>=0.30.0
qwen-vl-utils>=0.0.8
```

**Specs:**
- Size: ~4GB VRAM required
- Speed: ~2-3 seconds per image on RTX 3060
- Quality: Good for UI detection, decent for activity recognition
- Model ID: `Qwen/Qwen3-VL-2B-Instruct`

**When to use:**
- ✅ You have an NVIDIA GPU with 6GB+ VRAM
- ✅ Privacy is paramount (no data leaves machine)
- ✅ You want zero ongoing costs
- ❌ You want the highest quality roasts (API models are better)
- ❌ You have AMD GPU or CPU-only (too slow)

### TTS Model: Piper

```bash
pip install piper-tts
```

**Specs:**
- Size: ~100MB per voice
- Speed: Real-time on CPU
- Quality: Acceptable, robotic but clear
- Best voice: `en_US-lessac-medium`

**When to use:**
- ✅ You want completely offline operation
- ✅ API TTS costs are a concern
- ❌ You want natural-sounding speech (Gemini/OpenAI TTS is much better)

---

## 4. Cheapest API Options (Acceptable Quality)

### Cost Comparison (per 1K requests)

| Provider | Vision Model | Cost/1K | TTS Model | Cost/1M chars | Total/Month* |
|----------|--------------|---------|-----------|---------------|--------------|
| **Gemini** | gemini-2.0-flash | ~$0.35 | gemini-2.5-flash-tts | ~$0.50 | **~$3-5** |
| OpenAI | gpt-4o-mini | ~$0.30 | tts-1 | ~$15.00 | ~$10-15 |
| OpenRouter | gemini-2.0-flash | ~$0.35 | (use Gemini) | ~$0.50 | ~$3-5 |
| Anthropic | claude-sonnet-4 | ~$3.00 | (no TTS) | - | ~$15-20 |

*Assuming 10 roasts/day, 30 days, ~500 tokens vision + ~100 chars TTS per roast

### Recommended Budget Setup

```python
# Cheapest viable configuration
config = {
    "vision_provider": "gemini",
    "vision_model": "gemini/gemini-2.0-flash",
    "tts_provider": "gemini",
    "tts_model": "gemini/gemini-2.5-flash-preview-tts",
    "tts_voice": "Kore",
}
```

### Free Tier Options

| Provider | Free Tier | Limitations |
|----------|-----------|-------------|
| Gemini | 1,500 req/day | Rate limited, sufficient for personal use |
| OpenRouter | $0.50 credit | Runs out quickly, good for testing |

---

## 5. Local vs API: Default Recommendation

### Recommendation: API-First with Local Fallback

**Default: Gemini API** for these reasons:

1. **Zero setup friction** - Works immediately with just an API key
2. **Better quality** - API vision models are significantly better at understanding context
3. **Lower resource usage** - No GPU required, works on laptops
4. **Faster** - No model loading time
5. **Cheaper than expected** - ~$3-5/month for normal usage

**Local as Fallback:**
- Offer local models in settings for privacy-conscious users
- Detect GPU availability and suggest local if present
- Clear messaging: "Local mode requires NVIDIA GPU, 6GB VRAM"

### Implementation

```python
# In settings UI, show recommendation badges
VISION_PROVIDERS = {
    "gemini": {"name": "Google Gemini", "badge": "recommended"},
    "openai": {"name": "OpenAI", "badge": "premium"},
    "local": {"name": "Local (Qwen3-VL)", "badge": "privacy"},
}
```

---

## 6. Right-Click Context Menu Focus

### Current Menu Structure

```
├─ 🔥 Roast Now
├─ ⏯️ Monitoring: [Start/Stop]
├─ ⏱️ Scan Interval ▶
│  ├─ 1 minute
│  ├─ 5 minutes ✓
│  ├─ 10 minutes
│  └─ 30 minutes
├─ ───────────────
├─ 💬 Speech Bubble: [On/Off]
├─ 🔊 Audio: [On/Off]
├─ ───────────────
├─ ⚙️ Settings...
└─ ❌ Quit
```

### Recommended Improvements

1. **Add "Last Roast" Section**
   - Show timestamp of last roast
   - Click to repeat it (with TTS)

2. **Add "Personality" Toggle**
   - Gentle (encouraging, not mean)
   - Balanced (mild roasts)
   - Savage (default, brutal)

3. **Quick Provider Switch**
   - Submenu to quickly change vision/TTS without opening full settings

4. **Keyboard Shortcuts Display**
   - Show shortcuts in menu items (e.g., "Roast Now (Ctrl+R)")

5. **Status Section at Top**
   - Current provider
   - API status indicator
   - Next roast countdown

### Context Menu Code Location

```javascript
// ui/index.html - around line 200+
// Look for #ctx-menu and ctx-item classes
```

---

## 7. API Services to Include

### Tier 1: Core (Always Include)

| Provider | Vision | TTS | Why |
|----------|--------|-----|-----|
| **Gemini** | ✅ | ✅ | Best price/performance, native TTS |
| **OpenAI** | ✅ | ✅ | Industry standard, high quality |

### Tier 2: Extended (Include if space permits)

| Provider | Vision | TTS | Why |
|----------|--------|-----|-----|
| **Anthropic** | ✅ | ❌ | Excellent vision quality, no TTS |
| **OpenRouter** | ✅ | ❌ | Unified API, good for trying models |

### Tier 3: Niche (Optional)

| Provider | Vision | TTS | Why |
|----------|--------|-----|-----|
| **Groq** | ✅ | ❌ | Extremely fast, cheap |
| **Ollama** | ✅ | ✅ | Local API wrapper (if user has Ollama) |

### Provider Configuration Template

```python
PROVIDERS = {
    "gemini": {
        "name": "Google Gemini",
        "env_var": "GEMINI_API_KEY",
        "vision_models": [
            "gemini/gemini-2.0-flash",      # Recommended default
            "gemini/gemini-2.5-flash",      # Better quality, slightly more expensive
            "gemini/gemini-2.5-pro",        # Best quality
        ],
        "tts_models": ["gemini/gemini-2.5-flash-preview-tts"],
        "tts_voices": ["Kore", "Puck", "Charon", "Fenrir", "Aoede"],
        "docs_url": "https://aistudio.google.com/app/apikey",
    },
    "openai": {
        "name": "OpenAI",
        "env_var": "OPENAI_API_KEY",
        "vision_models": [
            "openai/gpt-4o-mini",           # Cheapest
            "openai/gpt-4o",                # Better quality
        ],
        "tts_models": ["openai/tts-1", "openai/tts-1-hd"],
        "tts_voices": ["alloy", "echo", "fable", "onyx", "nova", "shimmer"],
        "docs_url": "https://platform.openai.com/api-keys",
    },
    # ... etc
}
```

---

## 8. Alternative Front Ends

### Option 1: System Tray Application (Recommended Alternative)

Instead of a desktop pet, run as a system tray icon:

```python
# Using pystray + PIL
import pystray
from PIL import Image

def create_tray_icon():
    image = Image.open("icon.png")
    menu = pystray.Menu(
        pystray.MenuItem("Roast Now", on_roast),
        pystray.MenuItem("Settings", on_settings),
        pystray.MenuItem("Quit", on_quit),
    )
    icon = pystray.Icon("Lotaria", image, "Lotaria", menu)
    return icon
```

**Pros:**
- No window management issues
- Works on all platforms consistently
- Less intrusive

**Cons:**
- Less personality/character
- No eye tracking animations

### Option 2: Web Dashboard

Simple Flask/FastAPI server with web UI:

```python
from flask import Flask, render_template
import webview

app = Flask(__name__)

@app.route("/")
def dashboard():
    return render_template("dashboard.html", history=history)

# Run in background, open browser or embed
```

**Pros:**
- Accessible from phone/tablet
- Easier to customize UI
- Can add charts/stats

**Cons:**
- Requires browser
- Not "always on top"

### Option 3: Terminal/TUI

Using `rich` or `textual` for a terminal UI:

```python
from textual.app import App
from textual.widgets import Static, Button

class LotariaTUI(App):
    def compose(self):
        yield Static("Lotaria", id="header")
        yield Button("Roast Me", id="roast")
        yield Static("", id="output")
```

**Pros:**
- Extremely lightweight
- No GUI dependencies
- Great for remote/SSH usage

**Cons:**
- No images/animations
- No TTS (could use system `say` command)

### Option 4: Discord/Slack Bot

Same core logic, different delivery:

```python
import discord

@bot.command()
async def roast(ctx):
    screenshot = capture_screen()
    analysis = vision.analyze(screenshot)
    await ctx.send(f"🔥 {analysis}")
```

**Pros:**
- Share roasts with friends
- Natural social interaction
- Can roast multiple people

**Cons:**
- Requires Discord/Slack
- No local TTS

---

## 9. Additional Recommendations

### Security

```python
# API key handling - already implemented well
# Additional recommendations:

1. Mask keys in logs (already done)
2. Store in OS keyring instead of JSON file
   ```bash
   pip install keyring
   ```
3. Add key validation on save (test API call)
```

### Performance

```python
# Add these optimizations:

1. Screenshot caching (don't capture if screen hasn't changed)
2. Vision result caching (same screenshot = same roast)
3. Lazy model loading (only load local models when selected)
4. Audio preloading (keep TTS model warm)
```

### Privacy

```python
# Add privacy mode:

PRIVACY_CONFIG = {
    "blur_sensitive": True,      # Detect and blur passwords/keys in screenshots
    "local_only": False,         # Force local models only
    "no_storage": False,         # Don't save screenshots/history
    "auto_purge": 3600,          # Auto-delete after 1 hour
}
```

### Accessibility

```python
# Improvements for accessibility:

1. High contrast mode for speech bubbles
2. Larger text option
3. Screen reader support (ARIA labels)
4. Keyboard navigation for context menu
5. Reduced motion option (disable animations)
```

### Analytics (Optional)

```python
# Anonymous usage stats (opt-in only)

ANALYTICS = {
    "roasts_triggered": 0,
    "provider_usage": {},
    "avg_response_time": 0,
    "crash_reports": [],
}
```

---

## 10. Development Checklist

When working on Lotaria, ensure:

- [ ] Python 3.10+ with virtual environment
- [ ] Qt backend works (PyQt6 installed)
- [ ] API keys in `.env` or entered in-app
- [ ] `.temp/` directory is gitignored
- [ ] No browser TTS (use Gemini/OpenAI/Piper only)
- [ ] No CPU-only torch for local models
- [ ] Right-click context menu is intuitive
- [ ] Settings persist correctly
- [ ] Audio plays without external codecs
- [ ] Window hides before screenshot (avoid self-capture)
- [ ] History doesn't grow unbounded (max 20 entries)

---

## Quick Reference

### File Locations

| File | Purpose |
|------|---------|
| `app.py` | Entry point, window setup |
| `bridge.py` | JS ↔ Python API |
| `monitor.py` | Background roast thread |
| `services/state.py` | Config, history, constants |
| `services/capture.py` | Screen capture |
| `services/vision.py` | Vision AI providers |
| `services/tts.py` | Text-to-speech providers |
| `ui/index.html` | UI (HTML/CSS/JS) |

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `GEMINI_API_KEY` | Google Gemini API |
| `OPENAI_API_KEY` | OpenAI API |
| `ANTHROPIC_API_KEY` | Anthropic API |
| `OPENROUTER_API_KEY` | OpenRouter API |

### Default Ports/Paths

| Item | Value |
|------|-------|
| Config file | `.temp/config.json` |
| History file | `.temp/history.json` |
| Screenshots | `.temp/screenshot_*.png` |
| Audio files | `.temp/audio_*.wav` |
| Window size | 400x350 |
| Default position | Bottom-right of primary screen |

---

## Resources

- **Gemini API**: https://aistudio.google.com/app/apikey
- **OpenAI API**: https://platform.openai.com/api-keys
- **OpenRouter**: https://openrouter.ai/keys
- **Qwen3-VL**: https://huggingface.co/Qwen/Qwen3-VL-2B-Instruct
- **Piper TTS**: https://github.com/rhasspy/piper
- **pywebview**: https://pywebview.flowrl.com/
