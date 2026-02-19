# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Lotaria is a desktop pet that sits on your screen, periodically captures the screen, uses vision AI to analyze user activity, and roasts the user with a speech bubble + TTS audio.

## Current State

- **UI**: pywebview transparent frameless window — pixel art character with speech bubble, eyes track cursor
- **Vision**: Multi-provider via LiteLLM — Gemini (default), OpenAI, Anthropic, OpenRouter; local Qwen3-VL option
- **TTS**: Multi-provider — Gemini TTS (default, via google-genai SDK), OpenAI TTS (via LiteLLM); local Piper option
- **History**: Last 20 roasts saved to `.temp/history.json` for context/callbacks
- **Storage**: Images + audio saved to `.temp/`, auto-cleanup after 24h
- **Roast Style**: Savage comedy roast - brutal, specific, references previous observations
- **Settings**: In-app settings modal for API keys, provider/model selection

## Tech Stack

- **Desktop**: pywebview with Qt backend (PyQt6 + QtWebEngine) — transparent, frameless, always-on-top
- **Screen Capture**: `mss` (multi-screen shot)
- **Vision**: LiteLLM for multi-provider support (Gemini, OpenAI, Anthropic, OpenRouter). Local Qwen3-VL-2B-Instruct available as an option (requires CUDA GPU).
- **TTS**: Gemini TTS via `google-genai` SDK (default). OpenAI TTS via `litellm.speech()`. Local Piper available as an option.
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

| Provider | Vision | TTS | API Key Env Var |
|----------|--------|-----|-----------------|
| Google Gemini | gemini-2.0-flash, 2.5-flash, 2.5-pro | gemini-2.5-flash-preview-tts | `GEMINI_API_KEY` |
| OpenAI | gpt-4o, gpt-4o-mini | tts-1, tts-1-hd | `OPENAI_API_KEY` |
| Anthropic | claude-sonnet-4 | - | `ANTHROPIC_API_KEY` |
| OpenRouter | Various (Gemini, Claude, GPT-4o) | - | `OPENROUTER_API_KEY` |
| Local | Qwen3-VL-2B | Piper | - |

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
├── state.py        # Config, history, constants, PROVIDERS dict, roast prompt, cleanup
├── capture.py      # ScreenCaptureService (mss)
├── vision.py       # Vision service: LiteLLM (multi-provider) + Local (Qwen3-VL)
└── tts.py          # TTS service: Gemini (google-genai) + LiteLLM (OpenAI etc.) + Local (Piper)
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
