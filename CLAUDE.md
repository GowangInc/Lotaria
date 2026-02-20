# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Lotaria is a desktop pet that sits on your screen, periodically captures the screen, uses vision AI to analyze user activity, and roasts the user with a speech bubble + TTS audio.

## Current State

- **UI**: pywebview transparent frameless window — pixel art character with speech bubble, eyes track cursor
- **Vision**: Multi-provider via LiteLLM — Gemini (default), OpenAI, Anthropic, OpenRouter; local Ollama, LM Studio, and Qwen-VL options
- **TTS**: Multi-provider — Gemini TTS (default), OpenAI TTS (via LiteLLM); local Kokoro (ONNX), Piper, and KittenTTS options
- **History**: Last 20 roasts saved to `.temp/history.json` for context/callbacks
- **Storage**: Images + audio saved to `.temp/`, auto-cleanup after 24h
- **Model Support**: Automated downloader for local weights using HF/ModelScope
- **Roast Style**: Savage comedy roast - brutal, specific, references previous observations
- **Settings**: In-app settings modal with dynamic Ollama model detection and voice filtering

## Tech Stack

- **Desktop**: pywebview with Qt backend (PyQt6 + QtWebEngine) — transparent, frameless, always-on-top
- **Screen Capture**: `mss` (multi-screen shot)
- **Vision**: LiteLLM for multi-provider support (Gemini, OpenAI, Anthropic). Local vision via **Ollama**, **LM Studio**, or direct Transformers (Qwen3-VL).
- **TTS**: Gemini TTS via `google-genai` SDK (default). Local neural TTS via **Kokoro-ONNX** (fast), **Piper**, or **KittenTTS**.
- **Model Hosting**: Ollama is the preferred local vision host. Direct local vision requires CUDA.
- **UI**: Single HTML file with inline CSS/JS (no build step)
- **State**: In-memory with JSON file persistence
- **Python**: 3.10+ (note: on 3.14+ use Qt backend since pythonnet/EdgeChromium is unavailable)

## IMPORTANT: Do NOT Use

- **Browser SpeechSynthesis** - Sounds horrible, do not use for TTS
- **edge-tts** - Not wanted by user
- **FastAPI / React** - Removed; app is now a pywebview desktop pet
- **CPU-only torch** - If using local vision model, it must run on GPU (CUDA). Do not use `device_map='cpu'`

## Running

```bash
python -m venv .venv
.venv\Scripts\activate        # Windows
pip install -r requirements.txt
cp .env.example .env          # Add your API keys (or enter them in-app via Settings)
python app.py
```

### Optional: Local models

Local vision/TTS models can be enabled via Settings. They are **not required** for default operation. If you want to use them:

```bash
# CUDA PyTorch (required for local vision model)
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu126
# Local TTS
pip install piper-tts
```

## Supported Providers

| Provider | Type | Vision | TTS |
|----------|------|--------|-----|
| Google Gemini | API | Yes | Yes (Live & Standard) |
| OpenAI | API | Yes | Yes |
| Anthropic | API | Yes | - |
| Ollama | Local | Yes (auto-detected) | - |
| LM Studio | Local | Yes | - |
| Direct (HF) | Local | Qwen3-VL | Kokoro, Piper, KittenTTS |

API keys can be entered in-app via right-click > Settings, or set as environment variables / in `.env`.

## Known Issues / Setup Notes

- **Python 3.14+**: `pythonnet` has no wheels, so pywebview's EdgeChromium backend is unavailable. The app forces `gui="qt"` in `app.py` and uses PyQt6 + QtWebEngine instead.
- **Local vision model** (optional): Requires CUDA GPU, ~4GB VRAM, and PyTorch installed from the CUDA index. The correct HuggingFace model ID is `Qwen/Qwen3-VL-2B-Instruct`.
- **google-genai**: Required for Gemini TTS (which uses the generate_content API with audio modality, not the standard OpenAI-compatible speech endpoint).
- **litellm**: Used for multi-provider vision and OpenAI-compatible TTS.

## Architecture

### File Structure

```
app.py              # Entry point: pywebview window, wiring, auto-start
bridge.py           # LotariaBridge class (js_api for pywebview)
monitor.py          # MonitoringThread (background capture+analysis)
services/
├── __init__.py
├── state.py        # Config, history, constants, PROVIDERS dict, roast prompt
├── downloader.py   # ModelDownloader (HF/ModelScope asset fetcher)
├── capture.py      # ScreenCaptureService (mss)
├── vision.py       # Vision: LiteLLM, Ollama, LM Studio, Qwen-VL
└── tts.py          # TTS: Gemini, LiteLLM, Kokoro-ONNX, Piper, KittenTTS
ui/
└── index.html      # Single-file HTML: character, speech bubble, settings modal, context menu
```

### Communication

- **Python -> JS**: `window.evaluate_js('deliverRoast(jsonData)')` pushes roasts
- **JS -> Python**: `window.pywebview.api.methodName()` (async, returns Promise)

### Bridge API (exposed to JS)

| Method | Purpose |
|--------|---------|
| `roast_now()` | Capture + vision + TTS, returns result dict |
| `toggle_monitoring()` | Start/stop monitor thread |
| `get_config()` | Return full config dict (api_keys masked) |
| `set_config(key, value)` | Update one setting, persist |
| `get_providers()` | Return PROVIDERS dict for settings UI |
| `get_api_keys()` | Return masked API keys |
| `save_api_key(provider, key)` | Save an API key, set env var |
| `set_vision_config(provider, model)` | Update vision provider + model |
| `set_tts_config(provider, model, voice)` | Update TTS provider + model + voice |
| `quit()` | Stop monitor, destroy window |

### Configuration

- API keys: From environment/`.env` or entered in-app (in-app keys override env vars)
- Default scan interval: 300 seconds (5 minutes)
- History: 20 entries max, persisted to `.temp/history.json`
- Config persisted to `.temp/config.json`
- Config keys: `interval`, `vision_provider`, `vision_model`, `tts_provider`, `tts_model`, `tts_voice`, `api_keys`, `speech_bubble_enabled`, `audio_enabled`

### UI Interaction

- **Right-click**: Context menu with controls (roast, monitoring, settings, interval, toggles)
- **Settings modal**: API key management, vision/TTS provider and model selection
- **Drag**: Character is draggable via pywebview drag region
- **Eye tracking**: Character eyes follow the cursor position
- **Animations**: Idle bobbing + eye blink, roasting shake, thinking pulse
