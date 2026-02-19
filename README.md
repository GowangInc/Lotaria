# Lotaria

A desktop pet that watches your screen and roasts you.

Lotaria sits on your desktop as a transparent, always-on-top pixel art character. It periodically captures your screen, uses vision AI to analyze what you're doing, and delivers savage comedy roasts via speech bubble and text-to-speech audio.

## Features

- **Screen-aware roasts** — captures your screen and generates context-specific commentary
- **Vision AI** — local Qwen3-VL-2B-Instruct (GPU) or Gemini 2.0 Flash (API)
- **Text-to-speech** — local Piper TTS or Gemini TTS (API)
- **Roast history** — remembers last 20 roasts for callbacks and continuity
- **Desktop pet UI** — draggable pixel art character with idle animations, speech bubbles, and a right-click context menu
- **Configurable** — adjust scan interval, toggle vision/TTS providers, enable/disable speech bubbles and audio

## Requirements

- Python 3.10+
- Windows (pywebview with Qt backend)
- NVIDIA GPU with CUDA (for local vision model)
- Google API key (only if using API-based vision/TTS)

## Setup

```bash
python -m venv .venv
.venv\Scripts\activate

pip install -r requirements.txt

# PyTorch MUST be the CUDA build, not CPU-only:
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu126

# Only needed if using Gemini API models:
cp .env.example .env
# Edit .env and add your Google API key
```

## Usage

```bash
python app.py
```

- **Right-click** the character for the context menu (roast now, toggle monitoring, settings, quit)
- **Drag** the character to reposition it on screen
- Monitoring auto-starts and roasts you every 5 minutes by default

## Project Structure

```
app.py              # Entry point: pywebview window setup and auto-start
bridge.py           # JS API bridge (pywebview <-> Python)
monitor.py          # Background monitoring thread
services/
├── state.py        # Config, history, roast prompt, temp file cleanup
├── capture.py      # Screen capture (mss)
├── vision.py       # Vision analysis (local Qwen3-VL / Gemini API)
└── tts.py          # Text-to-speech (local Piper / Gemini API)
ui/
└── index.html      # Single-file UI: character, speech bubble, context menu
```

## Configuration

All settings are adjustable via the right-click context menu:

| Setting | Default | Description |
|---------|---------|-------------|
| Scan interval | 300s | Time between automatic roasts |
| Vision model | Local | Local (Qwen3-VL) or API (Gemini) |
| TTS model | Local | Local (Piper) or API (Gemini) |
| Speech bubble | On | Show/hide the speech bubble |
| Audio | On | Enable/disable TTS playback |

## Notes

- Temporary files (screenshots, audio) are stored in `.temp/` and auto-cleaned after 24 hours
- On Python 3.14+, the app uses the Qt backend since pythonnet/EdgeChromium is unavailable
- The local vision model requires ~4GB VRAM
