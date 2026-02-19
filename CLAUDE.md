# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Lotaria is a desktop pet that sits on your screen, periodically captures the screen, uses vision AI to analyze user activity, and roasts the user with a speech bubble + TTS audio.

## Current State

- **UI**: pywebview transparent frameless window — pixel art character with speech bubble
- **Vision**: Local (Qwen3-VL-2B-Instruct) default, Gemini API fallback
- **TTS**: Local (Piper) default, Gemini TTS API fallback
- **History**: Last 20 roasts saved to `.temp/history.json` for context/callbacks
- **Storage**: Images + audio saved to `.temp/`, auto-cleanup after 24h
- **Roast Style**: Savage comedy roast - brutal, specific, references previous observations

## Tech Stack

- **Desktop**: pywebview with Qt backend (PyQt6 + QtWebEngine) — transparent, frameless, always-on-top
- **Screen Capture**: `mss` (multi-screen shot)
- **Vision**: Qwen3-VL-2B-Instruct (local, GPU required) / Gemini 2.0 Flash (API)
- **TTS**: Piper (local) / Gemini TTS (API)
- **UI**: Single HTML file with inline CSS/JS (no build step)
- **State**: In-memory with JSON file persistence
- **Python**: 3.10+ (note: on 3.14+ use Qt backend since pythonnet/EdgeChromium is unavailable)

## IMPORTANT: Do NOT Use

- **Browser SpeechSynthesis** - Sounds horrible, do not use for TTS
- **edge-tts** - Not wanted by user
- **FastAPI / React** - Removed; app is now a pywebview desktop pet
- **CPU-only torch** - Vision model must run on GPU (CUDA). Do not use `device_map='cpu'`

## Running

```bash
python -m venv .venv
.venv\Scripts\activate        # Windows
pip install -r requirements.txt
# PyTorch must be CUDA build, not CPU-only:
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu126
cp .env.example .env          # Add your API_KEY (only needed for API models)
python app.py
```

## Known Issues / Setup Notes

- **Python 3.14+**: `pythonnet` has no wheels, so pywebview's EdgeChromium backend is unavailable. The app forces `gui="qt"` in `app.py` and uses PyQt6 + QtWebEngine instead.
- **PyTorch CUDA**: The default pip torch install may be CPU-only. You must install from the CUDA index (`--index-url https://download.pytorch.org/whl/cu126`) for the local vision model to work on GPU.
- **torchvision**: Required by the Qwen3-VL processor (fast image processor). Without it, the processor falls back to a slow path that may error.
- **Model name**: The correct HuggingFace model ID is `Qwen/Qwen3-VL-2B-Instruct` (not `Qwen3-VL-2B`).

## Architecture

### File Structure

```
app.py              # Entry point: pywebview window, wiring, auto-start
bridge.py           # LotariaBridge class (js_api for pywebview)
monitor.py          # MonitoringThread (background capture+analysis)
services/
├── __init__.py
├── state.py        # Config, history, constants, roast prompt, cleanup
├── capture.py      # ScreenCaptureService (mss)
├── vision.py       # Base + Local (Qwen3-VL) + API (Gemini) vision
└── tts.py          # Base + Local (Piper) + API (Gemini) TTS
ui/
└── index.html      # Single-file HTML: character, speech bubble, context menu
```

### Communication

- **Python -> JS**: `window.evaluate_js('deliverRoast(jsonData)')` pushes roasts
- **JS -> Python**: `window.pywebview.api.methodName()` (async, returns Promise)

### Bridge API (exposed to JS)

| Method | Purpose |
|--------|---------|
| `roast_now()` | Capture + vision + TTS, returns result dict |
| `toggle_monitoring()` | Start/stop monitor thread |
| `get_config()` | Return full config dict |
| `set_config(key, value)` | Update one setting, persist |
| `quit()` | Stop monitor, destroy window |

### Configuration

- `API_KEY` / `GOOGLE_API_KEY`: Google GenAI key from environment or `.env` (only needed for API models)
- Default scan interval: 300 seconds (5 minutes)
- History: 20 entries max, persisted to `.temp/history.json`
- Config keys: `interval`, `vision_model_type`, `tts_model_type`, `speech_bubble_enabled`, `audio_enabled`

### UI Interaction

- **Right-click**: Context menu with all controls
- **Drag**: Character is draggable via pywebview drag region
- **Animations**: Idle bobbing + eye blink, roasting shake, thinking pulse
