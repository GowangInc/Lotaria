# CLAUDE.md

This file provides guidance when working with the Lotaria Tauri codebase.

## Project Overview

Lotaria is a desktop pet that sits on your screen, periodically captures the screen, uses vision AI to analyze user activity, and roasts the user with a speech bubble + TTS audio.

**Architecture: Tauri 2.0 + Rust + TypeScript**

## Current State

- **UI**: Tauri frameless transparent window with Vite-built frontend
- **Vision**: Gemini (default), OpenAI, Anthropic, Groq, Ollama (local)
- **TTS**: Piper TTS (bundled, offline), Gemini TTS (free), OpenAI, Murf AI, ElevenLabs, Inworld AI
- **Moods**: 10 built-in (roast, helpful, encouraging, sarcastic, zen, anime, gordon, therapist, detective, hype) + custom with AI improvement
- **Intensity**: Configurable roast intensity slider (1-10, default 5) injected into prompts
- **Sound Effects**: Blip on roast start, chime on roast complete (generated sine waves via rodio)
- **Global Hotkey**: `Ctrl+Shift+R` triggers instant roast from anywhere (Tauri global shortcut plugin)
- **Frequency**: Configurable monitoring intervals (often/frequent/infrequent)
- **Pet Styles**: 10 highly detailed animated designs with unique personalities:
  - Each pet has multiple layers (body, pseudo-elements, shadows)
  - Custom animations (morphing, glowing, floating, waving, etc.)
  - Unique color schemes and visual effects
  - Environmental effects (scanlines, energy fields, rain, ripples)
- **Animations**: Avatar collapses before screenshot, expands after capture
- **History**: Last 20 roasts saved to config dir for context/callbacks
- **Storage**: Images + audio saved to cache dir, auto-cleanup after 24h; log files auto-cleanup after 48h
- **Build**: Vite builds frontend to `dist/`, Tauri bundles into native app
- **Settings**: Save button at bottom of settings panel; config persisted on save
- **Config Migration**: On startup, validates/fixes deprecated models and mismatched voices
- **Website**: Single-page landing site in `website/` (gitignored, for Cloudflare Pages)

## Tech Stack

- **Desktop**: Tauri 2.0 — transparent, frameless, always-on-top, click-through
- **Frontend Build**: Vite + TypeScript
- **Screen Capture**: `xcap` crate (cross-platform)
- **Vision**: Direct HTTP APIs (Gemini, OpenAI-compatible, Ollama local)
- **TTS**: Piper TTS (subprocess, bundled), Gemini TTS via `reqwest`, OpenAI, Murf AI, ElevenLabs, Inworld AI
- **Audio**: Playback via `rodio`
- **UI**: Single HTML file with TypeScript (no framework)
- **State**: JSON file persistence via `dirs` crate
- **Click-through**: `pointer-events: none` on background, `pointer-events: auto` on interactive elements

## Pet Design Philosophy

**CURRENT STATE**: Pets have improved visual details but are still fundamentally blob variations. Need major redesign.

**TODO - Future Pet Improvements**:
- **More distinct base shapes**: Not all rounded/circular (consider: tall/thin, wide/flat, angular, asymmetric)
- **Random animation variations**: Procedural timing offsets, random animation choices on load
- **Non-blob shapes**: Cat should have ears that break the silhouette, robot should be boxy, octopus tentacles should be actual limbs
- **Interactive states**: React to clicks, mouse proximity, time of day
- **Personality through movement**: Each pet moves fundamentally differently (hop vs glide vs mechanical step)
- **Break the 100x100 box**: Some pets could extend beyond the container (long tentacles, antenna, tail)

**Current Design Approach** (placeholder until redesign):
- Multi-layered effects using pseudo-elements (::before, ::after)
- Unique color gradients and shadow effects for depth
- Character-specific animations (not just generic movement)
- Environmental details (scanlines, glows, particles, weather)
- Pure CSS (no images) for lightweight performance

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

| Provider | Vision | TTS | Cost | Notes |
|----------|--------|-----|------|-------|
| **Google Gemini** | ✅ | ✅ | FREE | **Recommended** — vision + TTS included |
| **FoxCode** | ✅ | ⚠️ | $ ~¥0.12-0.35/M | Gemini proxy, TTS untested |
| **Inworld AI** | ❌ | ✅ | $ ~$5-10/M chars | Cheapest TTS, tts-1.5-mini/max, 7 voices |
| **Groq** | ✅ | ❌ | $ ~$1-2.50/mo | Fastest inference, needs separate TTS provider |
| **OpenAI** | ✅ | ✅ | $$ ~$1.50-5/mo | gpt-4.1-mini/4o, gpt-4o-mini-tts/tts-1 |
| **Anthropic** | ✅ | ❌ | $$$ ~$2.70/mo | Claude Sonnet/Opus, needs separate TTS provider |
| **ElevenLabs** | ❌ | ✅ | $$$ ~$5/mo+ | Free 10k chars, 3 models, premium voices |
| **Murf AI** | ❌ | ✅ | $$$$ ~$26/mo | Premium TTS only, Falcon (fast) / Gen2 (studio) |

API keys entered in-app are persisted to config and override environment variables.

### Gemini Models

**Vision** (accept image/video input, ordered newest first):
- `gemini-3.1-pro-preview` — Advanced reasoning, agentic coding (Preview)
- `gemini-3-flash-preview` — Frontier-class at fraction of cost (Preview)
- `gemini-3-pro-preview` — SOTA reasoning + multimodal (Preview)
- `gemini-2.5-flash` — Best price-performance (Stable, free tier)
- `gemini-2.5-pro` — Most advanced for complex tasks (Stable, free tier)
- `gemini-2.5-flash-lite` — Fastest/cheapest in 2.5 family (Stable)
- `gemini-2.0-flash` — Deprecated, shutdown June 2026
- `gemini-2.0-flash-lite` — Deprecated, shutdown June 2026

**TTS**:
- `gemini-2.5-flash-preview-tts` — Fast, low-latency, controllable
- `gemini-2.5-pro-preview-tts` — High-fidelity (podcasts, audiobooks)
- `gemini-2.5-flash-lite-preview-tts` — Cheapest TTS option

**Voices** (30 total): Kore, Charon, Puck, Fenrir, Aoede, Leda, Orus, Zephyr, Achernar, Achird, Algenib, Algieba, Alnilam, Autonoe, Callirrhoe, Despina, Enceladus, Erinome, Gacrux, Iapetus, Laomedeia, Pulcherrima, Rasalgethi, Sadachbia, Sadaltager, Schedar, Sulafat, Umbriel, Vindemiatrix, Zubenelgenubi

### FoxCode (Multi-Provider Proxy)

- **Base URLs**:
  - Gemini: `https://code.newcli.com/gemini`
  - Codex: `https://code.newcli.com/codex/v1`
  - Claude: `https://code.newcli.com/claude/aws`
- **Auth**: `x-api-key` header (same API key for all endpoints)
- **Vision Models**:
  - Gemini: gemini-3-pro, gemini-3-pro-high, gemini-3-pro-preview, gemini-3-flash, gemini-3-flash-preview, gemini-2.5-pro, gemini-2.5-flash, gemini-2.5-flash-lite
  - Codex: gpt-5.3-codex, gpt-5.2, gpt-5.2-codex, gpt-5.1, gpt-5.1-codex, gpt-5.1-codex-mini, gpt-5.1-codex-max, gpt-5, gpt-5-codex
  - Claude: claude-sonnet-4-6, claude-opus-4-6, claude-opus-4-5, claude-sonnet-4-5, claude-haiku-4-5-20251001, claude-opus-4, claude-opus-4-1, and thinking variants
- **TTS**: Not supported (requires separate TTS provider)
- **Cost**: ~¥0.03-0.35 per million tokens (significantly cheaper than official APIs)
- **Note**: Third-party proxy service - automatically routes to correct endpoint based on model prefix

### Murf AI

- **API**: `POST https://api.murf.ai/v1/speech/stream` (streaming, returns audio bytes directly)
- **Auth**: `api-key` header
- **Models**: `FALCON` (fast, streaming) / `GEN2` (studio-quality)
- **Voices**: en-US-natalie, en-US-amara, en-US-marcus, en-US-nate, en-US-carter, en-US-phoebe, en-US-terrell, en-UK-ruby, en-UK-hazel, en-UK-gabriel, en-UK-theo, en-UK-mason
- **Formats**: WAV, MP3, FLAC, PCM, OGG

### ElevenLabs

- **API**: `POST https://api.elevenlabs.io/v1/text-to-speech/{voice_id}` (returns MP3 audio directly)
- **Auth**: `xi-api-key` header
- **Models**: `eleven_multilingual_v2` (29 languages), `eleven_flash_v2_5` (low latency), `eleven_turbo_v2_5` (fastest)
- **Voices**: Rachel, Domi, Bella, Antoni, Elli, Josh, Arnold, Adam, Sam (voice name = voice_id for presets)
- **Free tier**: 10,000 characters/month

### Inworld AI

- **API**: `POST https://api.inworld.ai/tts/v1/tts` (returns WAV audio directly)
- **Auth**: `Authorization: Bearer` header
- **Models**: `tts-1.5-mini` ($5/M chars, lowest latency) / `tts-1.5-max` ($10/M chars, best quality)
- **Voices**: Sarah, Mark, Hana, Blake, Clive, Luna, Hades
- **Supports**: 15 languages, instant voice cloning

## Project Structure

```
src-tauri/              # Rust backend
├── src/
│   ├── main.rs         # Entry point, window creation
│   ├── lib.rs          # Module exports
│   ├── state.rs        # Config, history, PROVIDERS, prompts, migrations
│   ├── capture.rs      # ScreenCapture (xcap)
│   ├── vision.rs       # Vision services (Gemini, OpenAI)
│   ├── tts.rs          # TTS services + AudioPlayer
│   └── commands.rs     # Tauri commands (roast_now, improve_mood)
├── capabilities/       # Tauri 2.0 capabilities
├── icons/              # App icons
├── Cargo.toml
└── tauri.conf.json

src/                    # Frontend
├── index.html          # UI markup + styles (10 pet styles, custom mood)
└── main.ts             # TypeScript logic

website/                # Landing page (gitignored)
├── index.html          # Single-page marketing site
├── _redirects          # Cloudflare Pages redirects
└── README.md           # Deployment guide

dist/                   # Built frontend (gitignored)
```

## Architecture

### Rust Backend

**State Management** (`state.rs`):
- `Config` - Serializable app configuration with fields:
  - `mood: String` - Selected mood (roast/helpful/encouraging/sarcastic/custom)
  - `custom_mood: String` - User's custom mood prompt (when mood == "custom")
  - `pet_style: String` - Selected pet design (default/cat/ghost/robot/etc.)
  - Vision/TTS provider configs, API keys, etc.
- `History` - Vec of recent roasts (max 20 entries)
- `StateManager` - Handles persistence to config/cache dirs
- `ProviderDef` - Static provider definitions with vision_models, tts_models, tts_voices, live_voices, cost_note
- `MOOD_PROMPTS` - Built-in mood prompt templates (roast, helpful, encouraging, sarcastic)
- `build_prompt()` - Builds final prompt using mood (or custom_mood) + history + timestamp
- **Config migration on load**: Fixes deprecated models, validates voice names against provider

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
- `GeminiTTSService` - Standard Gemini TTS (generateContent with responseModalities: AUDIO)
- `GeminiLiveTTSService` - Live API (currently delegates to standard)
- `OpenAITTSService` - OpenAI speech endpoint
- `MurfTTSService` - Murf AI streaming endpoint (returns WAV bytes directly)
- `ElevenLabsTTSService` - ElevenLabs `/v1/text-to-speech/{voice_id}` (returns MP3)
- `InworldTTSService` - Inworld AI `/tts/v1/tts` (returns WAV)
- `AudioPlayer` - rodio-based playback (play_async spawns thread)
- **IMPORTANT**: Gemini API response uses camelCase JSON (`inlineData`, `mimeType`) — all response structs use `#[serde(rename_all = "camelCase")]`

**Commands** (`commands.rs`):
- All Tauri command handlers
- `roast_now` - Main capture → analyze → TTS flow
- `toggle_monitoring` - Background interval task
- `get_moods` - Returns list of available moods (built-in)
- `improve_mood` - Uses vision API to enhance custom mood prompts
- `AppState` - Shared state with tokio::sync::RwLock

### Frontend

**Build Flow**:
1. Vite builds `src/` → `dist/` (HTML + JS)
2. Tauri embeds `dist/` into binary
3. `tauri.conf.json` points to `../dist`

**Key Functions** (`main.ts`):
- `triggerRoast` - Collapses avatar (300ms), calls backend, expands avatar, displays result
- `deliverRoast` - Shows speech bubble with text
- `toggleMonitoring` - Starts/stops background roasting
- `showSettings` - Opens settings modal
- `buildVisionUI` / `buildTtsUI` / `buildMoodUI` - Populate dropdowns from provider definitions
- `setupDrag` - Manual drag with DPI scaling (`devicePixelRatio`)

**Avatar Animations**:
- `.hiding` class - Collapses to 0.1 scale before screenshot (300ms)
- `.showing` class - Expands back to 1.0 scale after screenshot (300ms)
- Avatar expands ~400ms after roast starts (after screenshot is captured)

**Settings Save Flow**:
1. User clicks 💾 Save Settings button (`#settings-save`)
2. JS reads all select values (vision provider/model, TTS provider/model/voice, mood, custom_mood)
3. Each value is saved via `invoke('set_config', { key, value })`
4. Config reloaded from backend, settings panel closes

**Custom Mood Flow**:
1. User selects "Custom" from mood dropdown
2. Custom mood section (`#custom-mood-section`) becomes visible
3. User enters text in `#custom-mood-input` textarea
4. User clicks "✨ Improve with AI" (`#improve-mood-btn`)
5. Frontend calls `invoke('improve_mood', { moodText })`
6. Backend sends mood text to vision API with improvement meta-prompt
7. Improved prompt (max 800 chars) replaces original in textarea
8. User saves settings, custom mood stored in `config.custom_mood`
9. When `config.mood == "custom"`, `build_prompt()` uses `config.custom_mood` instead of built-in prompts

### Communication

- **Rust → JS**: `app_handle.emit("event", data)`
- **JS → Rust**: `invoke("command", args)` (async, returns Promise)

## Configuration

### Frequency Intervals

Set via **Settings → Frequency** or `config.json`:

| Setting | Min | Max | Description |
|---------|-----|-----|-------------|
| `often` | 5 min | 10 min | Very frequent roasts |
| `frequent` | 10 min | 20 min | Default balanced rate |
| `infrequent` | 25 min | 45 min | Occasional roasts |

**Gemini Free Tier Override**: When `gemini_free_tier: true` and using Gemini TTS, intervals are automatically set to 60-90 minutes to avoid rate limits.

### Other Settings

- History: 20 entries max
- Config: `%APPDATA%/lotaria/config.json`
- Cache: `%LOCALAPPDATA%/lotaria/` (screenshots, audio)
- Log: `%LOCALAPPDATA%/lotaria/app.log`
- Auto-cleanup after 24 hours

## Website

A single-page landing site is located in `website/` (gitignored):

- `website/index.html` - Complete marketing page with:
  - Hero section with animated title
  - Feature showcase (8 features)
  - AI provider comparison grid
  - Download section
  - Floating demo pet
- `website/_redirects` - Cloudflare Pages redirects
- `website/README.md` - Deployment instructions

**Deploy to Cloudflare Pages**:
1. Go to pages.cloudflare.com
2. Create new project
3. Upload `website/` folder
4. Update GitHub/Discord links before deploying

## Common Issues

1. **Build fails with "frontendDist doesn't exist"**:
   - Run `npm run build` first to create `dist/`

2. **Icons missing**:
   - Add `32x32.png`, `128x128.png`, `icon.ico` to `src-tauri/icons/`

3. **Cargo check is slow first time**:
   - First build downloads and compiles all dependencies

4. **Build fails with "Access is denied" (os error 5)**:
   - The app is still running. Quit Lotaria before rebuilding.

5. **No audio output**:
   - Check logs at `%LOCALAPPDATA%/lotaria/app.log`
   - Verify TTS voice matches the TTS provider (e.g., "Kore" for Gemini, "alloy" for OpenAI)
   - Gemini TTS response structs MUST use `#[serde(rename_all = "camelCase")]`

6. **Drag doesn't work / cursor drifts**:
   - Drag uses manual positioning with `devicePixelRatio` scaling
   - `screenX/Y` are CSS pixels, `outerPosition`/`setPosition` use physical pixels

7. **Custom mood improvement fails**:
   - Check that a vision provider is configured with a valid API key
   - The improve_mood command uses the currently selected vision model
   - No image is sent - it's a text-to-text improvement
   - Check logs at `%LOCALAPPDATA%/lotaria/app.log`

8. **Avatar stays collapsed during roast**:
   - Avatar should collapse for 300ms before screenshot
   - Expands back ~400ms after roast starts (while vision analysis runs)
   - If it stays collapsed, check browser console for animation errors

9. **Window blocks clicks on desktop**:
   - Fixed via `pointer-events: none` on html/body/#app
   - Interactive elements use `pointer-events: auto` (character, speech bubble, menus)
   - Transparent areas now click-through to applications underneath
   - If an element should be clickable but isn't, check it has `pointer-events: auto`

## Code Style

- Rust: Use `anyhow::Result` for errors, `tracing` for logs
- TypeScript: Use strict types, avoid `any`
- Frontend: Vanilla JS/TS, no frameworks
- State: Prefer immutable updates




## New features to include as we progress

### 🟢 Trivial (< 1 hour)
(All trivial features implemented)

### 🟡 Easy (a few hours)
- **System tray controls** — Tauri 2.0 has built-in system tray support; wire menu items to existing commands
- **Pet moods that change** — Timer/random selector rotates through existing 10 moods automatically
- **Blacklist apps/windows** — Check foreground window title against config list before capture, skip if matched
- **Break reminders** — Track activity timestamps, emit "take a break" prompt if idle > threshold
- **Right-click context menu on pet** — Quick actions: roast now, change mood, mute, settings, quit (replaces needing to open full settings)
- **Pet click reactions** — Click/poke the pet and it reacts with a mini animation + short quip (no screenshot needed, just a canned or AI one-liner)
- **Scheduled personalities** — Different mood at different times of day (e.g., encouraging in the morning, sarcastic after midnight)

### 🟠 Medium (half-day to a day)
- **Idle animations** — Multiple CSS animation states (yawn, stretch, play) triggered randomly when not roasting
- **Pet reactions to specific apps** — Detect foreground app via window title, map app categories to different prompt variations
- **Custom triggers** — Rule system for roasting on specific app launches or after X minutes idle
- **Activity tracking** — Poll foreground window at intervals, log app usage over time with a persistent data store
- **Multi-monitor awareness** — Let user pick which monitor to capture; optionally let pet wander between monitors
- **Pet visual overhaul** — Replace current CSS-only blob pets with dramatically higher-quality visuals:
  - Use SVG or sprite-sheet based pets instead of pure CSS shapes
  - Distinct silhouettes per pet (cat with ears, robot with antenna, octopus with tentacles — not blob variations)
  - Expressive states: happy, annoyed, sleeping, shocked, thinking (tied to mood/activity)
  - Smooth multi-frame animations (walk cycles, idle fidgets, reaction sequences)
  - Particle effects and environmental flair (sparkles, rain, steam, glow trails)
  - Optional user-submitted or AI-generated pet skins
- **Chat window with file drag-and-drop** — Toggleable mini chat panel next to the pet:
  - Click pet or a chat icon to open a small resizable chat window
  - Text input to talk to the pet directly (uses current vision model for responses)
  - Drag-and-drop files/images into the chat — pet analyzes them with vision AI and responds
  - Conversation history within the session (scrollable, auto-cleared on close or configurable)
  - Pet reacts visually while "thinking" (loading animation, squinting expression)
  - Supports screenshots pasted from clipboard (Ctrl+V)

### 🔴 Hard (multi-day)
- **Productivity reports** — Depends on activity tracking; aggregation logic + report UI + scheduled generation
- **Pet evolution** — Progression state machine, multiple visual stages per pet, persistence tied to user behavior
- **Pet conversations** — Multi-instance architecture for multiple pet windows, inter-pet conversation engine
- **Plugin system** — Defined plugin API, loading/sandboxing mechanism (WASM/scripting), lifecycle hooks, docs
- **Voice conversations** — Speak to the pet via microphone (STT → AI response → TTS); requires audio input handling + streaming
- **Desktop widget mode** — Expandable dashboard showing weather, calendar, todos, or quick notes alongside the pet


