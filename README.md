# Lotaria
A desktop pet that watches your screen and roasts you — rebuilt with **Tauri + Rust + TypeScript**.

## Why Tauri?

| Metric | Python/pywebview | Tauri |
|--------|-----------------|-------|
| Bundle size | ~100-200MB | ~5-15MB |
| Memory usage | 150-300MB | 30-80MB |
| Cold start | 3-5 seconds | <1 second |
| Distribution | Complex | Single executable |
| Security | Good | Excellent |

## Features

- **Screen-aware roasts** — captures screen and generates context-specific commentary
- **Multi-provider AI** — Gemini (recommended), OpenAI, Anthropic, Groq, Ollama (local)
- **Local & Cloud TTS** — Piper TTS (bundled, offline), Gemini TTS (free), OpenAI, Murf AI, ElevenLabs, Inworld AI
- **Custom moods** — roast, helpful, encouraging, sarcastic, zen, anime, gordon, therapist, detective, hype, or create your own with AI improvement
- **Intensity control** — slider from gentle (1) to brutal (10) adjusts how hard the pet goes
- **Sound effects** — blip on roast start, chime on completion (procedurally generated, no bundled files)
- **Global hotkey** — `Ctrl+Shift+R` triggers an instant roast from anywhere
- **System tray** — tray icon with quick actions: roast, monitor, settings, quit
- **Right-click menu** — quick access to roast, mute, change mood, settings without opening full panel
- **Mood rotation** — optionally randomize personality each roast for variety
- **Scheduled personalities** — automatic mood changes by time of day (encouraging mornings, helpful midday, sarcastic afternoons, roast evenings, zen late night)
- **App blacklist** — skip roasts when specific apps/windows are in the foreground
- **Break reminders** — configurable reminders to take a break after continuous screen time
- **Pet click reactions** — click the pet for a poke animation and random quip
- **10 pet designs** — cat, ghost, robot, blob, owl, alien, pumpkin, cloud, octopus, or classic box
- **Smooth animations** — avatar collapses during screenshot, expands on return
- **Context memory** — remembers past roasts to call out patterns
- **Offline capable** — Use Ollama for vision + Piper for TTS with zero API costs
- **Tiny footprint** — entire app under 15MB (plus optional voice models)
- **Cross-platform** — Windows, macOS, Linux (Windows only for now)
- **Native performance** — Rust backend with Web frontend
- **Privacy first** — auto-delete screenshots after 24h, log files after 48h, pause anytime

## Prerequisites

1. **Rust** (1.75+): Install from [rustup.rs](https://rustup.rs/)
2. **Node.js** (18+): Install from [nodejs.org](https://nodejs.org/)

## Development Setup

```bash
# 1. Clone/navigate to the project
cd lotaria

# 2. Install Node dependencies
npm install

# 3. Run in development mode (Vite dev server + Tauri)
npm run dev
```

## Building for Production

```bash
# Build the frontend first
npm run build

# Then build the Tauri app
cargo tauri build

# Output:
# - Windows: src-tauri/target/release/bundle/msi/*.msi
# - Windows: src-tauri/target/release/bundle/nsis/*.exe
```

## Project Structure

```
lotaria/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── lib.rs       # Module exports
│   │   ├── state.rs     # Config, history, providers, moods
│   │   ├── capture.rs   # Screen capture (xcap)
│   │   ├── vision.rs    # Vision API clients (Gemini, OpenAI, Ollama)
│   │   ├── tts.rs       # TTS services (Piper, Gemini, OpenAI, etc.)
│   │   └── commands.rs  # Tauri commands (roast, improve_mood)
│   ├── binaries/        # Bundled Piper TTS binary + dependencies
│   ├── models/          # Bundled voice models (en_GB-alan-low)
│   ├── capabilities/    # Tauri 2.0 capabilities
│   ├── icons/           # App icons
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── PIPER_SETUP.md   # Instructions for bundling Piper
├── src/                 # Frontend
│   ├── index.html       # UI (10 pet styles, custom mood UI)
│   └── main.ts          # TypeScript app logic
├── dist/                # Built frontend (generated)
├── package.json
├── vite.config.ts
├── tsconfig.json
├── CLAUDE.md            # Developer guide
└── README.md
```

## Setup

1. **Get a Gemini API key** (recommended - free tier):
   - Visit [aistudio.google.com/app/apikey](https://aistudio.google.com/app/apikey)
   - Create a free API key
   - Enter it in the app's welcome screen

2. **Alternative providers**:
   - OpenAI: [platform.openai.com](https://platform.openai.com)
   - Anthropic: [console.anthropic.com](https://console.anthropic.com)
   - Groq: [console.groq.com](https://console.groq.com)

## Supported Providers

### Local (100% Free, Offline)
| Provider | Cost | Notes |
|----------|------|-------|
| **Ollama + Piper** ⭐ | FREE | Default setup, zero API costs, runs offline |

### Vision + TTS (All-in-one)
| Provider | Cost | Notes |
|----------|------|-------|
| **Google Gemini** | FREE | 30 voices, vision + TTS included |
| **OpenAI** | $$ | 11 voices, gpt-4o + TTS |

### Vision Only (Need separate TTS)
| Provider | Cost | Notes |
|----------|------|-------|
| **Groq** | $ | Fastest inference (~$1-2.50/mo) |
| **Anthropic Claude** | $$$ | Sonnet/Opus models (~$2.70/mo) |

### TTS Only (Need separate vision)
| Provider | Cost | Notes |
|----------|------|-------|
| **Piper TTS** | FREE | Bundled offline TTS, 7 voices included |
| **Inworld AI** | $ | Cheapest cloud TTS ($5-10/M chars), 7 voices |
| **ElevenLabs** | $$$ | Premium voices, free 10k chars/mo |
| **Murf AI** | $$$$ | Studio quality ($26/mo), Falcon/Gen2 |

## Custom Moods

Create your own personality prompts:

1. Open **Settings → Mood**
2. Select **"Custom"** from dropdown
3. Write your custom prompt (or start with a basic idea)
4. Click **"✨ Improve with AI"** to enhance your prompt
5. Save and enjoy your personalized roasts

Built-in moods: roast (savage), helpful (productivity coach), encouraging (cheerleader), sarcastic (dry wit), zen (mindfulness), anime (kawaii energy), gordon (chef brutality), therapist (emotional support), detective (investigative), hype (maximum energy).

## Settings

### Frequency

Control how often the pet analyzes your screen:

1. Open **Settings → Frequency**
2. Select monitoring interval:
   - **Often** — Every 5-10 minutes
   - **Frequent** — Every 10-20 minutes (default)
   - **Infrequent** — Every 25-45 minutes

Note: If using Gemini free tier with Gemini TTS, intervals are automatically extended to 60-90 minutes to avoid rate limits.

### API Keys

Add or update API keys for any provider in **Settings → API Keys**.

## Pet Styles

Choose from 10 uniquely animated designs in Settings:

- **📺 Retro Computer** - CRT scanlines, power LED, screen flicker
- **🐱 Cat** - Fuzzy with whiskers, ear twitches, slit pupils, breathing
- **👻 Ghost** - Ethereal glow, flowing trail, spooky mouth, floating
- **🤖 Robot** - Mechanical panels, rivets, beeping antenna, processing
- **🫧 Blob** - Gooey drips, morphing shape, color shifts, jiggling
- **🦉 Owl** - Feather patterns, ear tufts, wise blinking, hooting
- **👽 Alien** - Energy field, pulsing glow, scanning antenna, otherworldly
- **🎃 Pumpkin** - Carved face, inner glow, flickering candle, stem
- **☁️ Cloud** - Fluffy puffs, rain drops, lightning flash, drifting
- **🐙 Octopus** - Wavy tentacles, suction cups, water ripples, swimming

## How to Use

### Interacting with the Pet

The pet lives in a transparent, frameless window that stays always-on-top. The app uses **click-through** technology so you can interact with windows underneath:

| Area | Interaction |
|------|-------------|
| **Over the pet** | Click-through disabled — you can drag, right-click, or interact with the pet |
| **Empty space** | Click-through enabled — clicks pass through to windows underneath |

**Controls:**
- **Left-click + drag** — Move the pet anywhere on screen
- **Left-click** — Poke the pet for a random quip and bounce animation
- **Right-click** — Open context menu (Roast Now, Monitoring, Mute, Change Mood, Settings, Quit)
- **Click outside menu** — Close context menu and return to click-through mode

The pet automatically detects when your cursor is over it (within the 100×100px avatar area) and switches between interactive and pass-through modes.

## Architecture

### Rust Backend
- **State Management**: JSON-based config and history persistence
- **Screen Capture**: Cross-platform via `xcap` crate
- **Vision APIs**: Direct HTTP calls via `reqwest`
- **TTS APIs**: Gemini (`google-genai` style) and OpenAI-compatible
- **Audio**: Playback via `rodio`

### TypeScript Frontend
- **Build Tool**: Vite
- **Tauri API**: Invoke commands, listen to events
- **UI**: Vanilla HTML/CSS with custom styling
- **Window**: Frameless, transparent, always-on-top

## Key Dependencies

### Rust
- `tauri` - Desktop framework
- `xcap` - Screen capture
- `rodio` - Audio playback
- `reqwest` - HTTP client
- `tokio` - Async runtime

### Frontend
- `vite` - Build tool
- `@tauri-apps/api` - Tauri JavaScript API

## Development Tips

- Run `npm run build` before `cargo tauri build` to generate the `dist/` folder
- Use `cargo tauri dev` for hot-reload development
- Icons go in `src-tauri/icons/` (32x32.png, 128x128.png, icon.ico)

## License

MIT
