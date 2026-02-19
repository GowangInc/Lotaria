# Lotaria

A desktop pet that watches your screen and roasts you.

Lotaria sits on your desktop as a transparent, always-on-top pixel art character. It periodically captures your screen, uses vision AI to analyze what you're doing, and delivers savage comedy roasts via speech bubble and text-to-speech audio.

## Features

- **Screen-aware roasts** — captures your screen and generates context-specific commentary
- **Multi-provider AI** — supports Gemini, OpenAI, Anthropic, and OpenRouter via LiteLLM
- **Text-to-speech** — Gemini TTS or OpenAI TTS reads roasts aloud
- **In-app settings** — enter API keys and configure providers directly in the app
- **Roast history** — remembers last 20 roasts for callbacks and continuity
- **Desktop pet UI** — draggable pixel art character with idle animations, eye tracking, speech bubbles, and a right-click context menu
- **Configurable** — adjust scan interval, toggle speech bubbles and audio, switch providers and models

## Requirements

- Python 3.10+
- Windows (pywebview with Qt backend)
- At least one API key (Gemini, OpenAI, Anthropic, or OpenRouter)

## Setup

```bash
python -m venv .venv
.venv\Scripts\activate

pip install -r requirements.txt

cp .env.example .env
# Edit .env and add your API key(s), or enter them in-app via Settings
```

## Usage

```bash
python app.py
```

- **Right-click** the character for the context menu (roast now, toggle monitoring, settings, quit)
- **Settings** — manage API keys, choose vision/TTS providers and models
- **Drag** the character to reposition it on screen
- Monitoring auto-starts and roasts you every 5 minutes by default

## Supported Providers

| Provider | Vision | TTS | API Key Env Var |
|----------|--------|-----|-----------------|
| Google Gemini | gemini-2.0-flash, 2.5-flash, 2.5-pro | gemini-2.5-flash-preview-tts | `GEMINI_API_KEY` |
| OpenAI | gpt-4o, gpt-4o-mini | tts-1, tts-1-hd | `OPENAI_API_KEY` |
| Anthropic | claude-sonnet-4 | - | `ANTHROPIC_API_KEY` |
| OpenRouter | Various (Gemini, Claude, GPT-4o) | - | `OPENROUTER_API_KEY` |
| Local | Qwen3-VL-2B | Piper | - |

API keys can be set as environment variables, in `.env`, or entered directly in the Settings modal.

## Optional: Local Models

By default Lotaria uses API providers. If you have an NVIDIA GPU, you can switch to local models via Settings. Local models require additional dependencies:

```bash
# CUDA PyTorch for local vision (Qwen3-VL-2B-Instruct, ~4GB VRAM)
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu126

# Local TTS (Piper)
pip install piper-tts
```

## Project Structure

```
app.py              # Entry point: pywebview window setup and auto-start
bridge.py           # JS API bridge (pywebview <-> Python)
monitor.py          # Background monitoring thread
services/
├── state.py        # Config, history, providers, roast prompt, temp file cleanup
├── capture.py      # Screen capture (mss)
├── vision.py       # Vision analysis (LiteLLM multi-provider / local Qwen3-VL)
└── tts.py          # Text-to-speech (Gemini API / LiteLLM / local Piper)
ui/
└── index.html      # Single-file UI: character, speech bubble, settings modal, context menu
```

## Configuration

Settings are adjustable via the right-click context menu and the Settings modal:

| Setting | Default | Description |
|---------|---------|-------------|
| Vision provider | Gemini | AI provider for screen analysis |
| Vision model | gemini-2.0-flash | Specific model for vision |
| TTS provider | Gemini | AI provider for text-to-speech |
| TTS model | gemini-2.5-flash-preview-tts | Specific model for TTS |
| TTS voice | Kore | Voice for TTS output |
| Scan interval | 300s | Time between automatic roasts |
| Speech bubble | On | Show/hide the speech bubble |
| Audio | On | Enable/disable TTS playback |

## Notes

- Temporary files (screenshots, audio) are stored in `.temp/` and auto-cleaned after 24 hours
- On Python 3.14+, the app uses the Qt backend since pythonnet/EdgeChromium is unavailable
- API keys entered in-app are persisted to `.temp/config.json` and override environment variables
