# Lotaria

A desktop pet that watches your screen and roasts you.

Lotaria sits on your desktop as a transparent, always-on-top pixel art character. It periodically captures your screen, uses vision AI to analyze what you're doing, and delivers savage comedy roasts via speech bubble and text-to-speech audio.

## Features

- **Screen-aware roasts** — captures your screen and generates context-specific commentary
- **Multi-provider AI** — supports Gemini, OpenAI, Claude, and OpenRouter.
- **Local AI Support** — integrate with **Ollama** or **LM Studio** for local vision.
- **Neural TTS** — high-quality local voices via **Kokoro (ONNX)**, **Piper**, and **KittenTTS**.
- **In-app settings** — manage API keys and local providers directly.
- **Auto-Downloader** — automatically fetches required model assets from HuggingFace/ModelScope.

## Requirements

- Python 3.10+
- Windows (pywebview with Qt backend)
- Ollama (optional, for easiest local vision)

## Setup

```bash
python -m venv .venv
.venv\Scripts\activate

# Install core and local dependencies
pip install -r requirements.txt
```

## Local Model Configuration

Lotaria is built to be "Local First" if you have the hardware.

### 1. Vision (Eyes)
The recommended way to run local vision is via **Ollama**:
1. Install [Ollama](https://ollama.com).
2. Run `ollama pull moondream` or `ollama pull llama3-v1.5-lava`.
3. Lotaria will automatically detect your pulled models in the settings.

Alternatively, use **LM Studio** by enabling the local server on `localhost:1234`.

### 2. Text-to-Speech (Voice)
Lotaria includes high-quality neural TTS that runs directly in Python:
- **Kokoro-82M (ONNX)**: Near-professional quality, very fast (Default fallback).
- **Piper**: Extremely lightweight and stable.
- **KittenTTS**: Highly expressive models designed for local CPUs.

The app will automatically download the necessary voice weights from HuggingFace on first use.

## Usage

```bash
python app.py
```

- **Right-click** for the menu (Roast Now, Toggle Monitoring, Settings, Quit).
- **Settings** — pick your "Eyes" (Vision) and "Voice" (TTS).
- **Drag** — move the pet anywhere on your screen.

## Supported Providers

| Provider | Type | Vision Support | TTS Support |
|----------|------|----------------|-------------|
| **Google Gemini** | API | Yes | Yes (Live & Standard) |
| **OpenAI** | API | Yes | Yes |
| **Anthropic** | API | Yes | - |
| **Ollama** | Local | Yes | - |
| **LM Studio** | Local | Yes | - |
| **Direct (HF)** | Local | - | Yes (Kokoro/Piper/Kitten) |

## Project Structure

```
app.py              # Main app & UI window
bridge.py           # Python ↔ JS Communication
services/
├── state.py        # Config & Provider Definitions
├── downloader.py   # Automated Model Downloader
├── vision.py       # Vision Analysis (Cloud & Local)
└── tts.py          # Text-to-Speech Engines
ui/
└── index.html      # UI, Animations & Styles
```

## Hardware Note

Local vision models (like Qwen-VL or Llama-Vision) typically require a GPU with **4GB+ VRAM**. Local TTS (Kokoro/Piper) runs excellently on **CPU**.

## Notes

- Temporary files (screenshots, audio) are stored in `.temp/` and auto-cleaned after 24 hours
- On Python 3.14+, the app uses the Qt backend since pythonnet/EdgeChromium is unavailable
- API keys entered in-app are persisted to `.temp/config.json` and override environment variables
