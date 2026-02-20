# Lotaria Desktop Pet: Comprehensive Research Report

**Date:** February 20, 2026  
**Research Scope:** 10 Critical Areas for Product Success

---

## Executive Summary

Based on extensive research across AI API pricing, local model capabilities, Python packaging best practices, and UX patterns, this report provides actionable recommendations for each of your 10 research questions. The overarching recommendation is to **default to Google Gemini API** for the "golden path" while maintaining robust local model support as a privacy-focused alternative.

---

## 1. Best Default Options for "Works Out of the Box"

### 🏆 Recommended "Golden Path" Configuration

| Setting | Recommendation | Rationale |
|---------|---------------|-----------|
| **Vision Provider** | Google Gemini | Single ecosystem for vision + TTS, generous free tier, cheapest paid tier |
| **Vision Model** | `gemini-2.0-flash` | FREE tier available, excellent vision quality, fast enough for roasts |
| **TTS Provider** | Google Gemini | Native integration, no additional API key needed |
| **TTS Model** | `gemini-2.5-flash-preview-tts` | $0.50/1M input, $10/1M output - cheapest quality TTS |
| **TTS Voice** | "Kore" | Good comedic timing, distinct personality |
| **Scan Interval** | **5 minutes (300s)** | Sweet spot: not annoying, not forgettable |
| **Auto-Start** | **NO** - require explicit opt-in | Critical for privacy trust |
| **Speech Bubbles** | **ON** by default | Core personality expression |
| **Audio** | **ON** by default | Differentiating feature |
| **History** | 20 roasts | Good balance of context vs storage |

### Why Gemini as Default?

**Pros:**
- **Free tier:** 15 requests/minute, perfect for testing
- **Unified billing:** One API key for both vision AND TTS
- **Cheapest paid vision:** $0.075/1M tokens (Flash) vs $2.50 (GPT-4o)
- **Cheapest TTS:** $10/1M output vs $15-30 (OpenAI)
- **No separate TTS integration:** Uses same SDK (`google-genai`)

**Cons:**
- Less "household name" recognition than OpenAI
- Some users may have existing OpenAI credits

### First-Launch Experience Flow

```python
# Suggested onboarding flow
1. App launches with character visible but NOT monitoring
2. First-run modal appears:
   - "Hi, I'm Lotaria! I roast you while you work."
   - "To get started, I need an API key (or use local models)"
   - [Enter Gemini API Key] [Use Local Models] [Learn More]
3. After key validation, explicit "Start Monitoring" button
4. First roast happens within 30 seconds of starting
```

### Cost Projection (Gemini Default)

| Usage Level | Roasts/Day | Monthly Cost |
|-------------|-----------|--------------|
| Light | 10 | ~$0.50 (mostly free tier) |
| Medium | 30 | ~$2.50 |
| Heavy | 50 | ~$5.00 |

---

## 2. Python Environment Management Best Practices

### Current Pain Points Identified

1. **PyQt6/QtWebEngine MSVC Runtime** - Fresh Windows installs often missing Visual C++ redistributables
2. **CUDA/PyTorch version hell** - Local models require specific CUDA versions
3. **Python 3.14 compatibility** - `pythonnet` has no wheels, forcing Qt backend
4. **Package size bloat** - PyInstaller with PyTorch = 3GB+ executables

### Recommended Distribution Strategy

#### Option A: "Installer Package" (Recommended for Most Users)

```
Lotaria-Setup.exe
├── Bundled Python 3.11 (embedded)
├── Pre-installed dependencies
├── MSVC redist (silent install)
├── Desktop shortcut
└── Uninstaller
```

**Tools:**
- **Inno Setup** or **NSIS** for Windows installer
- **embedded Python** (python-3.11.x-embed-amd64.zip)
- No user Python installation required

#### Option B: "Portable ZIP" (For Power Users)

```
Lotaria-Portable.zip
├── .venv/ (pre-configured)
├── app/
├── run.bat (handles venv activation)
└── README.txt
```

#### Option C: "Run from Source" (For Developers)

```bash
# setup.bat - automated setup script
@echo off
echo Setting up Lotaria...
python -m venv .venv
.venv\Scripts\activate
pip install -r requirements.txt
echo Setup complete! Run 'python app.py' to start.
```

### PyInstaller vs Nuitka Decision Matrix

| Factor | PyInstaller | Nuitka |
|--------|-------------|--------|
| **File Size** | 3GB+ (with PyTorch) | 7MB+ (without PyTorch bundled) |
| **Startup Time** | 10-50 seconds | ~1 second |
| **Build Time** | 30+ minutes | 10+ minutes |
| **Compatibility** | Excellent | Good |
| **Code Protection** | Poor (easily decompiled) | Excellent (compiled to C) |
| **Resource Handling** | Automatic | Manual specification |

**Recommendation:** Use **PyInstaller for development/testing**, **Nuitka for production releases**.

### Setup Script Implementation

```python
# setup.py - automated environment setup
import subprocess
import sys
import os

def check_msvc_redist():
    """Check if MSVC runtime is installed"""
    try:
        import ctypes
        ctypes.CDLL("vcruntime140.dll")
        return True
    except:
        return False

def install_msvc_redist():
    """Download and install MSVC redistributable"""
    url = "https://aka.ms/vs/17/release/vc_redist.x64.exe"
    subprocess.run(["curl", "-L", "-o", "vc_redist.exe", url])
    subprocess.run(["vc_redist.exe", "/quiet", "/norestart"])

def main():
    print("🔧 Lotaria Setup")
    
    # Check Python version
    if sys.version_info < (3, 10):
        print("❌ Python 3.10+ required")
        sys.exit(1)
    
    # Check MSVC on Windows
    if os.name == 'nt' and not check_msvc_redist():
        print("📦 Installing MSVC Runtime...")
        install_msvc_redist()
    
    # Create venv
    if not os.path.exists(".venv"):
        print("🐍 Creating virtual environment...")
        subprocess.run([sys.executable, "-m", "venv", ".venv"])
    
    # Install dependencies
    print("📥 Installing dependencies...")
    pip = ".venv\\Scripts\\pip.exe" if os.name == 'nt' else ".venv/bin/pip"
    subprocess.run([pip, "install", "-r", "requirements.txt"])
    
    print("✅ Setup complete! Run 'python app.py' to start.")

if __name__ == "__main__":
    main()
```

---

## 3. Best Local Models to Run (2025)

### Vision Models Comparison

| Model | Size | VRAM | Speed | Quality | Best For |
|-------|------|------|-------|---------|----------|
| **Qwen3-VL-2B** | 2B | ~4GB | Fast | Good | Default local option |
| **Moondream 2025-04-14** | ~2B | ~3GB | Very Fast | Good+ | UI/document understanding |
| **LLaVA-NeXT (7B)** | 7B | ~8GB | Medium | Very Good | Higher quality roasts |
| **InternVL2-4B** | 4B | ~6GB | Medium | Very Good | Balanced performance |
| **MiniCPM-V-2.6** | 2.6B | ~5GB | Fast | Good | Mobile/edge deployment |
| **SmolVLM-2.2B** | 2.2B | ~4GB | Very Fast | Good | Resource-constrained |

### Updated Recommendation: Moondream 2025

**Why Moondream over Qwen3-VL-2B:**

1. **Superior document/UI understanding** - Better at reading what's actually on screen
2. **Smaller footprint** - ~3GB VRAM vs ~4GB
3. **Faster inference** - Optimized for edge deployment
4. **Apache 2.0 license** - Fully permissive
5. **Active development** - 2025-04-14 release shows continued improvement

**Hardware Requirements:**
- **Minimum:** GTX 1060 6GB / RTX 3050
- **Recommended:** RTX 3060 12GB or better
- **CPU-only:** Not recommended (unacceptable latency)

### TTS Models Comparison

| Model | Size | Quality | Speed | License | Best For |
|-------|------|---------|-------|---------|----------|
| **Piper** | ~100MB | Good | Very Fast | MIT | Default local TTS |
| **MeloTTS** | ~200MB | Very Good | Fast | MIT | Natural prosody |
| **XTTS v2** | ~1.5GB | Excellent | Medium | CPML | Voice cloning |
| **Parler TTS** | ~1GB | Excellent | Medium | Apache 2.0 | Production quality |
| **ChatTTS** | ~1GB | Excellent | Medium | AGPL | Conversational |

### Recommended Local Stack

```python
# config.py - local model configuration
LOCAL_CONFIG = {
    "vision": {
        "model": "moondream-2025-04-14",
        "repo": "vikhyatk/moondream-2",
        "vram_gb": 3,
        "quantize": "int8",  # int8 for 4GB cards, fp16 for 6GB+
    },
    "tts": {
        "model": "piper",
        "voice": "en_US-lessac-medium",
        "speed": 1.2,  # Slightly faster for comedic timing
    }
}
```

### Auto-Detection Logic

```python
import torch

def detect_local_capabilities():
    """Auto-detect if local models can run"""
    if not torch.cuda.is_available():
        return {"can_run_local": False, "reason": "No CUDA GPU detected"}
    
    gpu_memory = torch.cuda.get_device_properties(0).total_memory / 1e9
    
    if gpu_memory < 4:
        return {
            "can_run_local": False, 
            "reason": f"Insufficient VRAM ({gpu_memory:.1f}GB < 4GB required)"
        }
    
    return {
        "can_run_local": True,
        "vram_gb": gpu_memory,
        "recommended_model": "moondream" if gpu_memory < 6 else "llava-next-7b"
    }
```

---

## 4. Cheapest API Options with Acceptable Quality

### Vision + TTS Cost Matrix (per 1M tokens/characters)

| Provider | Vision Input | Vision Output | TTS Input | TTS Output | Combined Cost/1K Roasts* |
|----------|-------------|---------------|-----------|------------|------------------------|
| **Gemini 2.0 Flash** | FREE | FREE | $0.50 | $10.00 | ~$0.50-1.00 |
| **Gemini 2.5 Flash** | $0.15 | $3.50 | $0.50 | $10.00 | ~$1.50 |
| **GPT-4o Mini** | $0.15 | $0.60 | N/A | N/A | ~$0.75 (vision only) |
| **GPT-4o** | $2.50 | $10.00 | $15.00** | $30.00** | ~$25.00 |
| **Claude Sonnet 4** | $3.00 | $15.00 | N/A | N/A | ~$18.00 (vision only) |
| **Groq (Llama 3.1 8B)** | $0.05 | $0.08 | N/A | N/A | ~$0.65 (vision only) |
| **Fireworks AI** | $0.10-$0.90 | Varies | N/A | N/A | ~$1.00-5.00 |

*TTS costs calculated per character, not token. 100 chars ≈ 25 tokens.
**OpenAI TTS priced per 1M characters, not tokens.

### Monthly Cost Projections

#### Light Usage (10 roasts/day = 300/month)

| Provider | Monthly Cost |
|----------|-------------|
| Gemini 2.0 Flash | **FREE** (within free tier) |
| Gemini 2.5 Flash | ~$0.50 |
| GPT-4o Mini | ~$0.25 |
| GPT-4o | ~$7.50 |
| Claude Sonnet 4 | ~$5.50 |

#### Medium Usage (30 roasts/day = 900/month)

| Provider | Monthly Cost |
|----------|-------------|
| Gemini 2.0 Flash | ~$1.00 |
| Gemini 2.5 Flash | ~$2.50 |
| GPT-4o Mini | ~$0.75 |
| GPT-4o | ~$22.50 |
| Claude Sonnet 4 | ~$16.50 |

#### Heavy Usage (50 roasts/day = 1500/month)

| Provider | Monthly Cost |
|----------|-------------|
| Gemini 2.0 Flash | ~$2.50 |
| Gemini 2.5 Flash | ~$5.00 |
| GPT-4o Mini | ~$1.50 |
| GPT-4o | ~$37.50 |
| Claude Sonnet 4 | ~$27.50 |

### Cost Optimization Strategy

```python
# Smart provider switching based on usage
class CostOptimizedProvider:
    def __init__(self):
        self.daily_roast_count = 0
        self.last_reset = datetime.now()
    
    def get_provider(self):
        # Reset counter daily
        if (datetime.now() - self.last_reset).days >= 1:
            self.daily_roast_count = 0
            self.last_reset = datetime.now()
        
        # Use Gemini Flash (free tier) for first 15 roasts/day
        if self.daily_roast_count < 15:
            return "gemini", "gemini-2.0-flash"
        
        # Switch to GPT-4o Mini for cost savings
        return "openai", "gpt-4o-mini"
    
    def track_roast(self):
        self.daily_roast_count += 1
```

### Provider Mix Recommendation

**Best Value Combo:**
- **Vision:** Gemini 2.0 Flash (free tier covers most users)
- **TTS:** Gemini 2.5 Flash TTS (cheapest quality option)
- **Fallback:** GPT-4o Mini (if Gemini rate-limited)

---

## 5. Local vs API: Which Should Be Default?

### Decision Framework

```
Is user a first-time desktop AI app user?
├── YES → API Default (Gemini)
│         └── Offer local as "privacy mode"
│
└── NO → Do they have a GPU with 4GB+ VRAM?
          ├── YES → Offer local as default
          │         └── "Want better privacy? Use local models"
          │
          └── NO → API Default
                    └── "GPU detected but insufficient VRAM"
```

### Arguments Summary

| Factor | API Default Wins | Local Default Wins |
|--------|------------------|-------------------|
| **Setup friction** | ✅ Zero setup | ❌ CUDA, model downloads |
| **Quality** | ✅ Best vision models | ❌ Smaller, less capable |
| **Privacy** | ❌ Screenshots leave machine | ✅ Complete privacy |
| **Cost** | ❌ Ongoing API costs | ✅ Free after setup |
| **Offline** | ❌ Requires internet | ✅ Works air-gapped |
| **Battery** | ✅ Lower power usage | ❌ GPU drains battery |

### Final Recommendation: **API Default with Smart Detection**

```python
# First-run detection and recommendation
def get_default_mode():
    """Determine default mode based on user system"""
    
    # Check for GPU
    gpu_info = detect_local_capabilities()
    
    if gpu_info["can_run_local"]:
        return {
            "mode": "local",
            "message": f"🎮 {gpu_info['vram_gb']:.0f}GB GPU detected! "
                      "Local models recommended for privacy.",
            "can_switch": True
        }
    else:
        return {
            "mode": "api",
            "message": f"ℹ️ {gpu_info['reason']}. "
                      "Using cloud APIs (Gemini free tier available).",
            "can_switch": True
        }
```

### What Similar Projects Do

| Project | Default | Rationale |
|---------|---------|-----------|
| **Ollama** | Local | Developer-focused, CLI-first |
| **LM Studio** | Local | GUI makes local easy |
| **ComfyUI** | Local | Requires GPU for diffusion |
| **Claude Desktop** | API | Consumer-focused, zero setup |
| **ChatGPT Desktop** | API | Proprietary, cloud-only |

**Insight:** Projects that default to local are typically developer tools. Consumer apps default to cloud.

---

## 6. Right-Click Context Menu UX

### Improved Menu Structure

```
├─ 🔥 Roast Now
├─ ⏯️ Monitoring: [Start/Stop]
├─ ⏱️ Scan Interval ▶
│  ├─ 1 minute (Aggressive)
│  ├─ 5 minutes ✓ (Balanced)
│  ├─ 10 minutes (Relaxed)
│  ├─ 30 minutes (Chill)
│  └─ Custom...
├─ ───────────────
├─ 💬 Speech Bubble: [On/Off]
├─ 🔊 Audio: [On/Off]
├─ 🔕 Mute For ▶
│  ├─ 1 hour
│  ├─ Until tomorrow
│  └─ Custom...
├─ ───────────────
├─ 🎭 Personality ▶
│  ├─ Gentle (encouraging)
│  ├─ Balanced ✓ (playful)
│  ├─ Savage (brutal)
│  └─ Custom...
├─ 📜 Last Roast...
├─ ───────────────
├─ ⚙️ Settings...
├─ ❓ Help & About
└─ ❌ Quit
```

### Status Information Display

Add a subtle status bar or tooltip showing:
- Next roast in: 2m 34s
- Provider: Gemini 2.0 Flash
- Today: 12 roasts delivered
- API status: ✅ Connected

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+R` | Roast Now |
| `Ctrl+Shift+M` | Toggle Monitoring |
| `Ctrl+Shift+S` | Toggle Speech Bubble |
| `Ctrl+Shift+A` | Toggle Audio |
| `Ctrl+Shift+Q` | Quit |

### Personality Modes Implementation

```python
PERSONALITIES = {
    "gentle": {
        "prompt_suffix": "Keep it light and encouraging. Focus on positive reinforcement.",
        "example": "You've been coding for 2 hours straight - maybe a quick stretch? Your back will thank you!"
    },
    "balanced": {
        "prompt_suffix": "Playful teasing, observational humor. Not mean, but not soft either.",
        "example": "Another Stack Overflow tab? At this point you should just bookmark it."
    },
    "savage": {
        "prompt_suffix": "Brutal honesty, no holding back. Think Anthony Jeselnik energy.",
        "example": "You've been 'researching' for 45 minutes. We both know that's just you avoiding actual work."
    }
}
```

---

## 7. Which API Services to Include

### Recommended Provider List

| Provider | Include | Vision | TTS | Rationale |
|----------|---------|--------|-----|-----------|
| **Google Gemini** | ✅ **Core** | ✅ | ✅ | Cheapest, unified ecosystem |
| **OpenAI** | ✅ Include | ✅ | ✅ | Brand recognition, quality |
| **Anthropic Claude** | ✅ Include | ✅ | ❌ | Excellent vision, no TTS |
| **OpenRouter** | ✅ Include | ✅ | ❌ | Aggregation, model variety |
| **Groq** | ✅ **Add** | ✅ | ❌ | Ultra-fast, very cheap |
| **Ollama** | ✅ **Add** | ✅ | ✅ | Makes local models easy |
| **Together AI** | ❌ Skip | - | - | Redundant with OpenRouter |
| **Fireworks AI** | ❌ Skip | - | - | Niche, adds complexity |
| **Azure OpenAI** | ❌ Skip | - | - | Enterprise-only appeal |
| **Cloudflare Workers AI** | ❌ Skip | - | - | Free tier too limited |

### Why Add Groq

- **Speed:** 840+ tokens/second (Llama 3.1 8B)
- **Cost:** $0.05/1M input, $0.08/1M output
- **Marketing:** "World's fastest inference" is compelling
- **Integration:** LiteLLM already supports it

### Why Add Ollama

- **Simplifies local setup:** One command model pull
- **API compatibility:** OpenAI-compatible endpoint
- **User familiarity:** Many already have it installed
- **Integration:** Can detect running Ollama instance

### Provider Detection

```python
def detect_available_providers():
    """Auto-detect which providers are available"""
    available = []
    
    # Check API keys
    if os.getenv("GEMINI_API_KEY"):
        available.append("gemini")
    if os.getenv("OPENAI_API_KEY"):
        available.append("openai")
    if os.getenv("ANTHROPIC_API_KEY"):
        available.append("anthropic")
    
    # Check Ollama
    try:
        import requests
        response = requests.get("http://localhost:11434/api/tags", timeout=2)
        if response.status_code == 200:
            available.append("ollama")
    except:
        pass
    
    return available
```

---

## 8. Alternative Front Ends and Interaction Models

### Comparison Matrix

| Approach | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **Desktop Pet (Current)** | Personality, always visible, draggable | Takes screen space, can be distracting | ✅ **Keep as primary** |
| **System Tray Only** | Zero screen real estate, native feel | No personality, less engagement | ❌ Too minimal |
| **Web Dashboard** | Rich UI, remote access | Requires browser, not always visible | ❌ Wrong for this use case |
| **Windows Notifications** | Native feel, integrated | Limited interaction, transient | ⚠️ Good for some roasts |
| **Terminal/TUI** | Lightweight, SSH-able | No images, no casual users | ❌ Wrong audience |
| **Discord/Slack Bot** | Social sharing, multi-user | Requires Discord/Slack, no local TTS | ⚠️ Future expansion? |
| **Electron App** | Full web stack | Massive bundle size | ❌ Overkill |

### Recommended: Hybrid Approach

**Primary:** Desktop Pet (current implementation)
- Keep pixel art character
- Draggable positioning
- Speech bubbles + TTS

**Secondary:** Smart Notification Integration
- Use Windows notifications for "urgent" roasts
- Configurable per personality mode
- Example: "You've been on Reddit for 30 minutes" → notification

**Tertiary:** Mini Dashboard (optional)
- Accessible via double-click on character
- Shows roast history, stats, settings
- Doesn't replace main UI

### Windows Notification Integration

```python
from win10toast import ToastNotifier

def send_notification_roast(title, message, urgency="normal"):
    """Send roast as Windows notification for non-intrusive delivery"""
    toaster = ToastNotifier()
    
    if urgency == "low":
        # Just show in action center
        toaster.show_toast(
            title=f"Lotaria - {title}",
            msg=message,
            duration=5,
            threaded=True
        )
    else:
        # Show as popup + TTS
        toaster.show_toast(
            title=f"Lotaria - {title}",
            msg=message,
            duration=10,
            threaded=False
        )
```

---

## 9. Security, Privacy, and Trust Considerations

### API Key Security

**Current:** JSON file storage (`config.json`)
**Recommended:** OS keyring integration

```python
import keyring

def secure_store_api_key(provider, api_key):
    """Store API key in OS credential manager"""
    keyring.set_password(f"lotaria_{provider}", "api_key", api_key)

def secure_get_api_key(provider):
    """Retrieve API key from OS credential manager"""
    return keyring.get_password(f"lotaria_{provider}", "api_key")
```

**Benefits:**
- Windows: Credential Locker (encrypted)
- macOS: Keychain
- Linux: Secret Service/KWallet
- No plaintext keys in files

### Screenshot Privacy

**Recommended Features:**

1. **Privacy Mode Toggle**
   ```python
   PRIVACY_SENSITIVE_KEYWORDS = [
       "password", "credit card", "ssn", "bank",
       "login", "sign in", "authentication"
   ]
   
   def should_blur_screenshot(text_content):
       """Detect if screenshot contains sensitive content"""
       text_lower = text_content.lower()
       return any(keyword in text_lower for keyword in PRIVACY_SENSITIVE_KEYWORDS)
   ```

2. **Application Blacklist**
   ```python
   PRIVATE_APPS = [
       "1password", "lastpass", "bitwarden",
       "bank", "paypal", "venmo"
   ]
   
   def is_private_window_active():
       """Check if a private app is currently active"""
       # Use win32gui or similar to get active window
       active_window = get_active_window_title().lower()
       return any(app in active_window for app in PRIVATE_APPS)
   ```

3. **Screenshot Preview (Optional)**
   - Before sending to API, show thumbnail
   - User can cancel if sensitive
   - Adds friction but builds trust

### Data Minimization

```python
# Auto-cleanup configuration
CLEANUP_CONFIG = {
    "screenshots": {"max_age_hours": 24, "max_count": 50},
    "audio_files": {"max_age_hours": 24, "max_count": 50},
    "history": {"max_entries": 20},  # Already implemented
}

def cleanup_old_files():
    """Remove old temporary files"""
    # Implementation in your existing cleanup function
    pass
```

### Consent Best Practices

**First Launch:**
```
┌─────────────────────────────────────────┐
│  👋 Welcome to Lotaria!                 │
│                                         │
│  I watch your screen and roast you.     │
│                                         │
│  ⚠️ Privacy Notes:                      │
│  • Screenshots are captured periodically│
│  • Images are analyzed by AI (cloud)    │
│  • Screenshots auto-delete after 24h    │
│  • You can pause anytime                │
│                                         │
│  [Start Monitoring] [Settings] [Quit]   │
└─────────────────────────────────────────┘
```

**Ongoing Consent:**
- Visual indicator when monitoring is active (subtle LED dot)
- Easy pause/resume (right-click menu)
- "Mute for X hours" option

### Transparency Dashboard

Add a "Privacy" tab in settings showing:
- Total screenshots captured today
- API calls made this month
- Storage used by temp files
- Option to "Clear all data now"

---

## 10. Additional Considerations

### Accessibility

```python
ACCESSIBILITY_FEATURES = {
    "screen_reader_support": {
        "roast_alerts": "Use system notifications (screen reader compatible)",
        "speech_bubble": "Add ARIA labels for screen readers"
    },
    "high_contrast_mode": {
        "speech_bubble_border": "3px solid white",
        "character_outline": "Visible in high contrast"
    },
    "reduced_motion": {
        "idle_animation": "Disable bobbing",
        "eye_tracking": "Keep but reduce update frequency"
    },
    "font_size": "Respect system font size settings"
}
```

### Analytics (Optional, Opt-In)

```python
ANALYTICS_EVENTS = {
    # Only if user opts in
    "app_started": "Track session start",
    "roast_delivered": "Count roasts (anonymized)",
    "provider_switched": "Which providers are popular",
    "error_occurred": "Track errors for improvement"
}

# Implementation with privacy-first approach
def track_event(event_name, properties=None):
    """Track event only if user opted in"""
    if not config.get("analytics_enabled", False):
        return
    
    # Anonymize - no screenshots, no roast content
    sanitized_properties = {
        "timestamp": datetime.now().isoformat(),
        "event": event_name,
        # No PII, no content
    }
    
    # Send to analytics service (e.g., Plausible, PostHog)
```

### Auto-Update Mechanism

**Recommended:** Squirrel (Windows) or custom update checker

```python
import requests
from packaging import version

def check_for_updates():
    """Check GitHub releases for updates"""
    try:
        response = requests.get(
            "https://api.github.com/repos/yourname/lotaria/releases/latest",
            timeout=5
        )
        latest = response.json()
        latest_version = latest["tag_name"].lstrip("v")
        
        if version.parse(latest_version) > version.parse(CURRENT_VERSION):
            return {
                "available": True,
                "version": latest_version,
                "url": latest["html_url"],
                "notes": latest["body"]
            }
    except:
        pass
    
    return {"available": False}
```

### Error Handling Strategy

```python
class RoastErrorHandler:
    """Graceful error handling for API failures"""
    
    FALLBACK_ROASTS = [
        "My vision model seems to be napping. Try again?",
        "I'm having a moment. Give me a sec...",
        "Connection's being weird. Classic internet.",
    ]
    
    def handle_vision_error(self, error):
        """Handle vision API failure"""
        if "rate_limit" in str(error).lower():
            return {
                "text": "Whoa, slow down! Rate limit hit. I'll try again in a minute.",
                "retry_after": 60
            }
        elif "invalid_api_key" in str(error).lower():
            return {
                "text": "Your API key seems off. Check Settings?",
                "action": "open_settings"
            }
        else:
            return {
                "text": random.choice(self.FALLBACK_ROASTS),
                "retry_after": 30
            }
```

### Community Features (Future)

- **Roast Sharing:** Opt-in to share favorite roasts
- **Leaderboards:** "Most roasted this week" (anonymous)
- **Custom Prompts:** Community-submitted roasting styles
- **Character Skins:** User-created character variations

### Internationalization (i18n)

**Priority:** Low for initial launch

If implementing:
```python
# Use gettext or similar
import gettext

# Supported languages
SUPPORTED_LOCALES = ["en", "es", "fr", "de", "ja", "zh"]

# TTS voices per locale
TTS_VOICES = {
    "en": ["Kore", "Fenrir"],
    "es": ["Puck"],
    "fr": ["Leda"],
    # ...
}
```

### Platform Expansion

| Platform | Complexity | Priority |
|----------|-----------|----------|
| **Windows** | Baseline | ✅ Primary |
| **macOS** | Medium | ⚠️ Post-launch |
| **Linux** | Medium | ⚠️ Community contribution? |

**macOS Considerations:**
- Use `pyobjc` for native integration
- Menu bar app (different from Windows system tray)
- Notarization required for distribution
- ARM64 (Apple Silicon) support critical

---

## Implementation Priority Matrix

| Feature | Impact | Effort | Priority |
|---------|--------|--------|----------|
| Gemini default + free tier | High | Low | P0 |
| OS keyring for API keys | High | Low | P0 |
| Auto-detect GPU for local | Medium | Low | P0 |
| Privacy mode + app blacklist | High | Medium | P0 |
| Improved context menu | Medium | Low | P1 |
| Personality modes | Medium | Low | P1 |
| Moondream local model | Medium | Medium | P1 |
| Groq provider support | Low | Low | P2 |
| Ollama integration | Medium | Medium | P2 |
| Windows notifications | Low | Low | P2 |
| Auto-updater | Medium | Medium | P2 |
| Analytics (opt-in) | Low | Low | P3 |
| macOS support | Medium | High | P3 |
| i18n | Low | High | P4 |

---

## Resources and References

### Pricing Sources
- [Google Gemini Pricing](https://ai.google.dev/pricing)
- [OpenAI Pricing](https://openai.com/pricing)
- [Anthropic Claude Pricing](https://www.anthropic.com/pricing)
- [Groq Pricing](https://groq.com/pricing)
- [Fireworks AI Pricing](https://fireworks.ai/pricing)

### Model Benchmarks
- [Moondream Benchmarks](https://moondream.ai/blog/moondream-2025-04-14-release)
- [LLaVA Leaderboard](https://huggingface.co/spaces/liuhaotian/Leaderboard)
- [OpenRouter Models](https://openrouter.ai/models)

### Packaging Tools
- [PyInstaller Documentation](https://pyinstaller.org/)
- [Nuitka Documentation](https://nuitka.net/)
- [cx_Freeze Documentation](https://cx-freeze.readthedocs.io/)

### Privacy Best Practices
- [GDPR Compliance Guide](https://gdpr.eu/checklist/)
- [Python Keyring Library](https://github.com/jaraco/keyring)

---

## Conclusion

The path forward is clear:

1. **Default to Gemini** for the smoothest out-of-box experience
2. **Implement OS keyring** for secure credential storage
3. **Auto-detect GPU** and offer local models when appropriate
4. **Prioritize privacy controls** to build user trust
5. **Keep the desktop pet UX** - it's your differentiator

The combination of Gemini's free tier, unified vision+TTS, and low paid pricing makes it the ideal default. Local models (especially Moondream) provide an excellent privacy-focused alternative for users with GPUs.

Focus on the P0 and P1 items for launch, then iterate based on user feedback.

---

*Report compiled: February 20, 2026*
*Sources: 60+ web searches, official documentation, community benchmarks*
