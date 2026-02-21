# CLAUDE.md

This file provides guidance when working with the Lotaria Tauri codebase.

## Project Overview

Lotaria is a desktop pet that sits on your screen, periodically captures the screen, uses vision AI to analyze user activity, and roasts the user with a speech bubble + TTS audio.

**Architecture: Tauri 2.0 + Rust + TypeScript**

## Current State

- **UI**: Tauri frameless transparent window with Vite-built frontend
- **Vision**: API-only via HTTP clients — Gemini (default), OpenAI, Anthropic, Groq, DeepSeek
- **TTS**: API-only — Gemini TTS (default with unlimited Live API), OpenAI TTS
- **History**: Last 20 roasts saved to config dir for context/callbacks
- **Storage**: Images + audio saved to cache dir, auto-cleanup after 24h
- **Build**: Vite builds frontend to `dist/`, Tauri bundles into native app

## Tech Stack

- **Desktop**: Tauri 2.0 — transparent, frameless, always-on-top
- **Frontend Build**: Vite + TypeScript
- **Screen Capture**: `xcap` crate (cross-platform)
- **Vision**: Direct HTTP APIs (Gemini, OpenAI-compatible)
- **TTS**: Gemini TTS via `reqwest`, OpenAI TTS
- **Audio**: Playback via `rodio`
- **UI**: Single HTML file with TypeScript (no framework)
- **State**: JSON file persistence via `dirs` crate

## IMPORTANT: Do NOT Use

- **Browser SpeechSynthesis** - Sounds horrible
- **edge-tts** - Not wanted by user
- **Local models** - API-only for simplicity
- **Python** - Fully migrated to Rust

## Running

```bash
# Install dependencies
npm install

# Development (Vite dev server + Tauri)
npm run dev

# Production build
npm run build
cargo tauri build
```

## Supported Providers

| Provider | Vision | TTS | Notes |
|----------|--------|-----|-------|
| **Google Gemini** | ✅ | ✅ | **Recommended** - Free tier, unlimited Live TTS |
| **OpenAI** | ✅ | ✅ | gpt-4o, tts-1 |
| **Anthropic** | ✅ | ❌ | Claude models |
| **Groq** | ✅ | ❌ | Fast inference |
| **DeepSeek** | ✅ | ❌ | Cheapest |

API keys entered in-app are persisted to config and override environment variables.

## Project Structure

```
src-tauri/              # Rust backend
├── src/
│   ├── main.rs         # Entry point, window creation
│   ├── lib.rs          # Module exports
│   ├── state.rs        # Config, history, PROVIDERS, prompts
│   ├── capture.rs      # ScreenCapture (xcap)
│   ├── vision.rs       # Vision services (Gemini, OpenAI)
│   ├── tts.rs          # TTS services + AudioPlayer
│   └── commands.rs     # Tauri commands
├── capabilities/       # Tauri 2.0 capabilities
├── icons/              # App icons
├── Cargo.toml
└── tauri.conf.json

src/                    # Frontend
├── index.html          # UI markup
└── main.ts             # TypeScript logic

dist/                   # Built frontend (gitignored)
```

## Architecture

### Rust Backend

**State Management** (`state.rs`):
- `Config` - Serializable app configuration
- `History` - Vec of recent roasts
- `StateManager` - Handles persistence to config/cache dirs
- `ProviderDef` - Static provider definitions

**Screen Capture** (`capture.rs`):
- `ScreenCapture::capture_primary()` - Returns PNG bytes + base64
- Uses `xcap` for cross-platform monitor capture
- Saves to cache dir

**Vision** (`vision.rs`):
- `VisionService` trait - async analyze(image, prompt) -> text
- `GeminiVisionService` - Uses Gemini generateContent API
- `OpenAIVisionService` - OpenAI-compatible chat completions
- `create_vision_service()` - Factory function

**TTS** (`tts.rs`):
- `TTSService` trait - async synthesize(text) -> audio bytes
- `GeminiTTSService` - Standard Gemini TTS
- `GeminiLiveTTSService` - Live API (currently falls back)
- `OpenAITTSService` - OpenAI speech endpoint
- `AudioPlayer` - rodio-based playback

**Commands** (`commands.rs`):
- All Tauri command handlers
- `roast_now` - Main capture → analyze → TTS flow
- `toggle_monitoring` - Background interval task
- `AppState` - Shared state with tokio::sync::RwLock

### Frontend

**Build Flow**:
1. Vite builds `src/` → `dist/` (HTML + JS)
2. Tauri embeds `dist/` into binary
3. `tauri.conf.json` points to `../dist`

**Key Functions** (`main.ts`):
- `roast_now` - Calls backend, displays result, plays audio
- `deliverRoast` - Shows speech bubble with text
- `playAudio` - Decodes base64 and plays
- `toggleMonitoring` - Starts/stops background roasting
- `showSettings` - Opens settings modal

### Communication

- **Rust → JS**: `app_handle.emit("event", data)`
- **JS → Rust**: `invoke("command", args)` (async, returns Promise)

## Configuration

- Default scan interval: 600 seconds (10 minutes)
- History: 20 entries max
- Config: `%APPDATA%/lotaria/config.json`
- Cache: `%LOCALAPPDATA%/lotaria/` (screenshots, audio)
- Auto-cleanup after 24 hours

## Common Issues

1. **Build fails with "frontendDist doesn't exist"**:
   - Run `npm run build` first to create `dist/`

2. **Icons missing**:
   - Add `32x32.png`, `128x128.png`, `icon.ico` to `src-tauri/icons/`

3. **Cargo check is slow first time**:
   - First build downloads and compiles all dependencies

## Code Style

- Rust: Use `anyhow::Result` for errors, `tracing` for logs
- TypeScript: Use strict types, avoid `any`
- Frontend: Vanilla JS/TS, no frameworks
- State: Prefer immutable updates
