# Research Prompt for Kimi Agent: Lotaria Desktop Pet

## Introduction & Context

I'm building **Lotaria** - a desktop pet application that sits on your screen as a transparent, always-on-top pixel art character. It periodically captures screenshots, uses vision AI to analyze what you're doing, and delivers savage comedy roasts via speech bubbles and text-to-speech audio.

Think of it as a digital companion that roasts you for procrastinating, celebrates your productivity, or calls out your questionable browsing habits. The tone is inspired by comedy roasts (think Nikki Glaser or Anthony Jeselnik) - brutal but funny.

### The Core Problem I'm Trying to Solve

I want Lotaria to be something users can:
1. **Install and run immediately** without hours of configuration
2. **Actually enjoy using** without it being annoying or creepy
3. **Afford to run** without surprise API bills
4. **Trust with their screen data** (privacy concerns)
5. **Customize** to their taste without drowning in options

But I'm struggling with several fundamental architectural decisions that will shape the entire user experience.

### Current Implementation

**Tech Stack:**
- Desktop: pywebview with Qt backend (transparent, frameless, always-on-top window)
- Screen Capture: mss library
- Vision AI: LiteLLM wrapper supporting multiple providers (Gemini, OpenAI, Anthropic, OpenRouter) + optional local Qwen3-VL
- TTS: Gemini TTS via google-genai SDK, OpenAI TTS via LiteLLM, optional local Piper
- UI: Single HTML file with inline CSS/JS (pixel art character, eye tracking, speech bubbles, right-click context menu)
- Storage: JSON files for config and history (last 20 roasts)
- Python: 3.10+ (Windows-focused, though cross-platform would be nice)

**Currently Configured Providers:**
| Provider | Vision Models | TTS Models | Notes |
|----------|--------------|------------|-------|
| Google Gemini | gemini-2.0-flash, 2.5-flash, 2.5-pro | gemini-2.5-flash-preview-tts | Native TTS, cheap |
| OpenAI | gpt-4o, gpt-4o-mini | tts-1, tts-1-hd | Industry standard |
| Anthropic | claude-sonnet-4 | - | Excellent vision, no TTS |
| OpenRouter | Various aggregated | - | Access to many models |
| Local | Qwen3-VL-2B | Piper | Free, requires GPU |

**Current Defaults:**
- Vision: Gemini 2.0 Flash
- TTS: Gemini 2.5 Flash Preview TTS (voice: "Kore")
- Interval: 300 seconds (5 minutes)
- Starts monitoring automatically
- Speech bubbles: enabled
- Audio: enabled

### What I'm Uncertain About

1. **The "it just works" dilemma**: I can default to API providers (easy setup, costs money) or local models (harder setup, free). Which leads to better user retention?

2. **The paradox of choice**: I support 4 API providers + local. Is this confusing? Should I pick one "blessed" path?

3. **Cost anxiety**: Users might love the app but stop using it if they're worried about API costs. How do I make costs predictable and acceptable?

4. **Privacy trust**: The app literally watches your screen. How do I design this so users feel in control and not surveilled?

5. **Interaction model**: Desktop pet vs system tray vs web dashboard - which interaction pattern fits this concept best?

6. **Python packaging hell**: Desktop Python apps are notoriously hard to distribute. What's the current state of the art?

### Target User Personas

**Primary**: Tech-savvy developer/creative professional who spends long hours at the computer, enjoys internet culture, appreciates dark humor, and doesn't mind spending a few dollars a month on tooling.

**Secondary**: Privacy-conscious user who wants the experience without sending screenshots to the cloud, willing to invest in a GPU or tolerate lower quality.

**Tertiary**: Casual user who saw a viral video/stream and wants to try it out. Low tolerance for complex setup.

### Success Criteria

- New user can go from "discovered" to "roasted" in under 5 minutes
- Monthly operating cost under $5 for typical usage (10-20 roasts/day)
- User feels in control of when/what is captured
- Works reliably on Windows without admin privileges
- Can run entirely offline (local models) as an option

---

## Research Questions

Based on the above context, please research and provide comprehensive recommendations on:

---

### 1. Best Default Options for "Works Out of the Box"

What would be the optimal default configuration for users who just want to install and run without tinkering? Consider:
- Which provider should be default? (Gemini has both vision + TTS in one ecosystem, OpenAI is more universally known/trusted)
- Which specific models balance quality and cost for roasts that are actually funny?
- What scan interval feels right? (Too frequent = annoying, too rare = forgettable)
- Should it start monitoring automatically or wait for explicit user opt-in? (Privacy vs convenience)
- Should speech bubbles and audio default to on or off?
- Any other defaults that improve first-time UX?

**What I'm looking for**: A specific "golden path" config that I can ship with confidence, with reasoning for each choice.

---

### 2. Python Environment Management Best Practices

What are the common Python issues users face with desktop apps like this and what are current best practices?
- Virtual environment setup and activation pitfalls on Windows
- PyQt6/QtWebEngine installation issues (MSVC runtime requirements, common on fresh Windows installs)
- Python 3.14 compatibility (pythonnet has no wheels, forcing Qt backend)
- CUDA/PyTorch installation pain for local models (version mismatches, GBs of downloads)
- Packaging/distribution options in 2025: PyInstaller, Nuitka, Briefcase, others?
- Should I provide a setup script? What should it actually do?
- One-file executable vs installer vs "run from source"?

**What I'm looking for**: A recommended distribution strategy and any setup automation I should provide.

---

### 3. Best Local Models to Run (2025)

Research the current state-of-the-art for local vision and TTS models:
- **Vision**: Is Qwen3-VL-2B still the best option for ~4GB VRAM? What about:
  - LLaVA variants (LLaVA-NeXT, LLaVA-OneVision)
  - Moondream (ultra-lightweight)
  - InternVL2
  - MiniCPM-V
  - SmolVLM
- **TTS**: Is Piper still the best offline TTS? What about:
  - Coqui TTS (XTTS v2)
  - MeloTTS
  - ONNX-based alternatives
  - sherpa-onnx
- Hardware requirements: What's the minimum VRAM for acceptable performance? Can any run on CPU reasonably?
- Speed vs quality trade-offs for real-time desktop use

**What I'm looking for**: Updated model recommendations for both vision and TTS, with hardware requirements and a decision matrix.

---

### 4. Cheapest API Options with Acceptable Quality

Research current pricing (as of 2025) for vision + TTS combinations:
- **Google Gemini**: gemini-2.0-flash, 2.5-flash, 2.5-pro (vision) + gemini-2.5-flash-preview-tts
- **OpenAI**: gpt-4o-mini, gpt-4o (vision) + tts-1, tts-1-hd
- **Anthropic**: claude-sonnet-4 (vision only - need separate TTS)
- **OpenRouter**: Aggregated pricing, any deals?
- **Other providers to consider**:
  - Groq (speed/cost claims)
  - Together AI
  - Fireworks AI
  - Any new budget players?

Calculate approximate monthly costs for:
- Light usage: 10 roasts/day, ~500 tokens vision + ~100 chars TTS per roast
- Medium usage: 30 roasts/day
- Heavy usage: 50 roasts/day

**What I'm looking for**: A cost comparison table and specific recommendations for best value combinations. Should I recommend mixing providers (cheap vision + premium TTS)?

---

### 5. Local vs API: Which Should Be Default?

This is a fundamental product decision I need to make.

**Arguments for API default:**
- Zero setup, works immediately after entering API key
- Significantly better vision model quality (roasts are funnier)
- No GPU required - works on any laptop
- Lower battery/resource usage
- Faster model switching

**Arguments for Local default:**
- Complete privacy - no screenshots leave the machine
- Zero ongoing costs after initial setup
- Works offline/air-gapped
- No API key management friction
- "Set it and forget it" - no usage anxiety

**Questions to research:**
- What do similar projects do? (ComfyUI, Ollama, etc.)
- What's the current user expectation for AI desktop apps?
- Can/should I auto-detect hardware and suggest?
- How much does vision model quality actually matter for this use case?
- Is "free" worth the setup friction?

**What I'm looking for**: A clear recommendation on which should be default, with a decision framework for when to suggest the alternative.

---

### 6. Right-Click Context Menu UX

The current menu structure:
```
├─ 🔥 Roast Now
├─ ⏯️ Monitoring: [Start/Stop]
├─ ⏱️ Scan Interval ▶
│  ├─ 1 minute
│  ├─ 5 minutes ✓
│  ├─ 10 minutes
│  └─ 30 minutes
├─ ───────────────
├─ 💬 Speech Bubble: [On/Off]
├─ 🔊 Audio: [On/Off]
├─ ───────────────
├─ ⚙️ Settings...
└─ ❌ Quit
```

**Questions:**
- Is this the right structure? What should be added or removed?
- Should there be a "Last Roast" display or "Repeat Last" option?
- Personality modes: Should users be able to toggle between Gentle/Balanced/Savage?
- Quick provider switching: Worth adding without opening full settings?
- Keyboard shortcuts: Should they be shown in menu items?
- Status info: Show next roast countdown, current provider, API status?
- What are common patterns from successful desktop pets (Desktop Goose, BonziBuddy, etc.) or modern system tray apps?
- Should there be a "Mute for X hours" option?

**What I'm looking for**: An improved menu structure design and UX best practices for desktop pet interactions.

---

### 7. Which API Services to Include

Currently supporting: Gemini, OpenAI, Anthropic, OpenRouter, Local

**Should I add:**
- Groq (claims extremely fast inference, competitive pricing)
- Together AI (good for open models)
- Azure OpenAI (enterprise appeal)
- Cloudflare Workers AI (free tier?)
- Ollama (local API wrapper - would make local models easier)
- LM Studio (similar to Ollama)
- Custom OpenAI-compatible endpoints (for power users)

**Questions:**
- What are the pros/cons of supporting too many vs too few providers?
- Is there value in being "provider agnostic" vs "curated experience"?
- Which providers are most reliable in terms of uptime/API stability?
- Any providers I should drop?

**What I'm looking for**: A recommended provider list with inclusion/exclusion rationale.

---

### 8. Alternative Front Ends and Interaction Models

The current UI is a transparent pywebview window with a draggable pixel art character. But I'm wondering if this is the right approach.

**Alternatives to research:**
- **System tray only** (no visible window, just icon + notification toasts)
  - Pros: Zero screen real estate, native feel
  - Cons: Less personality, no idle animations
- **Web dashboard** (Flask/FastAPI server, accessible from browser/phone)
  - Pros: Rich UI potential, accessible remotely
  - Cons: Requires browser open, not "always visible"
- **Terminal/TUI** (using rich or textual)
  - Pros: Extremely lightweight, works over SSH
  - Cons: No images/animations
- **Discord/Slack bot** (same core logic, different delivery)
  - Pros: Social sharing, multi-user
  - Cons: Requires Discord/Slack, no local TTS
- **Windows notification toasts** (win10toast, Windows-Toast-Notifications)
  - Pros: Native feel, integrated into OS
  - Cons: Limited interaction, transient
- **Electron app** (heavier but more customizable)
  - Pros: Full web stack, easier UI development
  - Cons: Massive bundle size

**Questions:**
- What interaction pattern best fits a "desktop pet" concept?
- Should I support multiple modes?
- What are users expecting from this type of app?

**What I'm looking for**: A comparison of approaches with recommendation for primary interaction model.

---

### 9. Security, Privacy, and Trust Considerations

**Current implementation:**
- API keys stored in `.temp/config.json` (masked in UI)
- Screenshots stored in `.temp/` (auto-deleted after 24h)
- History of last 20 roasts persisted

**Questions to research:**
- **API Key Security**: Should I use OS keyring instead of JSON file? (keyring library, Windows Credential Manager, macOS Keychain)
- **Screenshot Privacy**: Should I implement blur detection for sensitive content (password fields, credit cards)?
- **Data Minimization**: Should there be a "privacy mode" that doesn't store anything?
- **User Control**: Should there be an "incognito mode" for certain applications (don't capture when specific windows are active)?
- **Transparency**: Should I show a preview of what was captured before sending to API?
- **Consent**: Is auto-start monitoring creepy? Should there be a prominent "Start Monitoring" button on first launch?

**What I'm looking for**: Security/privacy best practices and features that build user trust.

---

### 10. Additional Considerations

What else should I be thinking about?

**Potential areas:**
- **Accessibility**: Screen reader support, high contrast mode, reduced motion option
- **Analytics**: Should I track anonymous usage? (opt-in only?) What metrics matter?
- **Updates**: Auto-update mechanism? Or manual updates?
- **Error Handling**: How to gracefully handle API failures, rate limits, network issues?
- **Community Features**: Share roasts, leaderboards, custom prompts?
- **Customization**: User-uploaded characters, custom voices, personalized roasting styles?
- **Internationalization**: Worth supporting other languages?
- **Platform Expansion**: macOS/Linux support complexity?

**What I'm looking for**: Anything important I'm overlooking that could make or break this project.

---

## Expected Deliverables

For each question above, please provide:

1. **Specific recommendations** with clear reasoning
2. **Current data** (pricing, benchmark scores, hardware requirements) as of 2025
3. **Code examples or configuration snippets** where helpful
4. **Pros/cons lists** for major decisions
5. **Decision frameworks** ("Choose X if Y, otherwise Z")
6. **Links to relevant resources** (docs, comparisons, similar projects)

I'm looking for actionable guidance I can implement, not just general information. Assume I'm technically capable but appreciate specific recommendations backed by research.
