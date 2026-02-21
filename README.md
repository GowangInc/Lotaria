# Lotaria (Tauri Edition)

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
- **Multi-provider AI** — Gemini (recommended), OpenAI, Anthropic, Groq, DeepSeek
- **Neural TTS** — Gemini Live API (free unlimited tier) or OpenAI TTS
- **Tiny footprint** — entire app under 15MB
- **Cross-platform** — Windows, macOS, Linux
- **Native performance** — Rust backend with Web frontend

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
│   │   ├── state.rs     # Config, history, providers
│   │   ├── capture.rs   # Screen capture (xcap)
│   │   ├── vision.rs    # Vision API clients
│   │   ├── tts.rs       # TTS API clients + audio
│   │   └── commands.rs  # Tauri commands
│   ├── capabilities/    # Tauri 2.0 capabilities
│   ├── icons/           # App icons
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                 # Frontend
│   ├── index.html       # UI
│   └── main.ts          # TypeScript app logic
├── dist/                # Built frontend (generated)
├── package.json
├── vite.config.ts
├── tsconfig.json
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

| Provider | Vision | TTS | Notes |
|----------|--------|-----|-------|
| **Google Gemini** | ✅ | ✅ | **Recommended** - Free tier, unlimited Live TTS |
| **OpenAI** | ✅ | ✅ | gpt-4o, tts-1 |
| **Anthropic Claude** | ✅ | ❌ | Requires separate TTS provider |
| **Groq** | ✅ | ❌ | Very fast inference |
| **DeepSeek** | ✅ | ❌ | Cheapest option |

## Configuration

Configuration is stored in:
- **Windows**: `%APPDATA%\lotaria\config.json`
- **macOS**: `~/Library/Application Support/lotaria/config.json`
- **Linux**: `~/.config/lotaria/config.json`

Temporary files (screenshots, audio) are stored in cache directories and auto-cleaned after 24 hours.

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
