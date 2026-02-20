# Lotaria Implementation Plan

Based on the research report priority matrix, here's the implementation roadmap.

## Priority Legend
- **P0** - Critical (Do First)
- **P1** - High Priority  
- **P2** - Medium Priority
- **P3** - Low Priority
- **P4** - Future Consideration

---

## P0 - Critical Features

### 1. Gemini Default + Free Tier Optimization
**Impact:** High | **Effort:** Low | **Files:** `services/state.py`, `ui/index.html`

Tasks:
- [ ] Update default config to use Gemini 2.0 Flash (free tier eligible)
- [ ] Add free tier awareness (15 requests/minute limit)
- [ ] Implement rate limit handling with user-friendly messages
- [ ] Add cost estimator in settings UI

```python
# In services/state.py
config = {
    "vision_provider": "gemini",
    "vision_model": "gemini/gemini-2.0-flash",  # FREE tier
    "tts_provider": "gemini",
    "tts_model": "gemini/gemini-2.5-flash-preview-tts",
    "tts_voice": "Kore",
    # ... rest unchanged
}
```

### 2. OS Keyring for API Key Storage
**Impact:** High | **Effort:** Low | **Files:** `services/state.py`, `requirements.txt`

Tasks:
- [ ] Add `keyring` to requirements.txt
- [ ] Create migration: JSON → keyring
- [ ] Update `set_api_key()` to use keyring
- [ ] Update `get_api_key()` to check keyring first, fallback to env
- [ ] Add keyring availability check for older Windows

### 3. Auto-Detect GPU for Local Model Suggestion
**Impact:** Medium | **Effort:** Low | **Files:** `services/state.py`, `ui/index.html`

Tasks:
- [ ] Create `detect_gpu_capabilities()` function
- [ ] On first launch, detect GPU and suggest local if capable
- [ ] Show VRAM info in settings
- [ ] Disable local option if no GPU/insufficient VRAM

### 4. Privacy Mode + Application Blacklist
**Impact:** High | **Effort:** Medium | **Files:** `services/capture.py`, `services/state.py`, `monitor.py`, `ui/index.html`

Tasks:
- [ ] Add privacy settings to config
- [ ] Implement active window detection
- [ ] Create blacklist of sensitive apps (password managers, banking)
- [ ] Add "Pause when private apps active" toggle
- [ ] Add visual indicator when monitoring is paused

---

## P1 - High Priority Features

### 5. Improved Right-Click Context Menu
**Impact:** Medium | **Effort:** Low | **Files:** `ui/index.html`

Tasks:
- [ ] Add "Last Roast" option (view + replay)
- [ ] Add "Mute For" submenu (1 hour, until tomorrow, custom)
- [ ] Add "Personality" submenu (Gentle, Balanced, Savage)
- [ ] Add status display (next roast countdown, provider)
- [ ] Show keyboard shortcuts in menu items

### 6. Personality Modes
**Impact:** Medium | **Effort:** Low | **Files:** `services/state.py`, `ui/index.html`

Tasks:
- [ ] Add personality config option
- [ ] Create personality prompts dictionary
- [ ] Update `build_roast_prompt()` to include personality
- [ ] Add personality selector in settings
- [ ] Persist personality choice

```python
PERSONALITIES = {
    "gentle": "Keep it encouraging and light...",
    "balanced": "Playful teasing, observational humor...",
    "savage": "Brutal honesty, no holding back..."
}
```

### 7. Moondream Local Model Support
**Impact:** Medium | **Effort:** Medium | **Files:** `services/vision.py`, `requirements.txt`

Tasks:
- [ ] Add Moondream model option alongside Qwen3-VL
- [ ] Update model loading code
- [ ] Add model selection in local settings
- [ ] Test VRAM requirements (3GB vs 4GB)
- [ ] Benchmark speed vs quality

### 8. First-Launch Onboarding
**Impact:** High | **Effort:** Medium | **Files:** `ui/index.html`, `bridge.py`

Tasks:
- [ ] Create welcome modal
- [ ] Show privacy explanation
- [ ] API key entry or "Use Local Models" choice
- [ ] Explicit "Start Monitoring" button (not auto-start)
- [ ] Remember first-run completion

---

## P2 - Medium Priority Features

### 9. Groq Provider Support
**Impact:** Low | **Effort:** Low | **Files:** `services/state.py`

Tasks:
- [ ] Add Groq to PROVIDERS dict
- [ ] Add vision models (llama-3.1-8b, mixtral)
- [ ] Test LiteLLM integration
- [ ] Document pricing advantage

### 10. Ollama Integration
**Impact:** Medium | **Effort:** Medium | **Files:** `services/state.py`, `services/vision.py`

Tasks:
- [ ] Detect running Ollama instance
- [ ] Add Ollama provider option
- [ ] List available vision models from Ollama
- [ ] Fallback if Ollama not running

### 11. Windows Notification Integration
**Impact:** Low | **Effort:** Low | **Files:** `bridge.py`, `services/tts.py`

Tasks:
- [ ] Add win10toast to requirements
- [ ] Create notification for non-intrusive roasts
- [ ] Configurable: popup vs notification
- [ ] Test with different urgency levels

### 12. Auto-Update Mechanism
**Impact:** Medium | **Effort:** Medium | **Files:** New file `updater.py`

Tasks:
- [ ] Check GitHub releases for updates
- [ ] Show update notification in UI
- [ ] Download and apply updates
- [ ] Respect user preference (auto-check vs manual)

### 13. Error Handling with Fallback Roasts
**Impact:** Medium | **Effort:** Low | **Files:** `bridge.py`, `monitor.py`

Tasks:
- [ ] Create fallback roast messages
- [ ] Handle rate limits gracefully
- [ ] Handle API key errors with helpful messages
- [ ] Add retry logic with exponential backoff

---

## P3 - Low Priority Features

### 14. Analytics (Opt-In Only)
**Impact:** Low | **Effort:** Low | **Files:** New file `analytics.py`

Tasks:
- [ ] Create privacy-first analytics
- [ ] Track only: app starts, roast counts, errors
- [ ] No content, no screenshots, no PII
- [ ] Explicit opt-in during onboarding

### 15. Accessibility Improvements
**Impact:** Medium | **Effort:** Medium | **Files:** `ui/index.html`

Tasks:
- [ ] Add ARIA labels to speech bubble
- [ ] Respect system reduced-motion setting
- [ ] High contrast mode option
- [ ] Screen reader notification support

---

## P4 - Future Considerations

### 16. Internationalization (i18n)
**Impact:** Low | **Effort:** High

- Multiple language support
- Locale-specific TTS voices
- Translation of UI

### 17. macOS Support
**Impact:** Medium | **Effort:** High

- Platform detection and adaptations
- Menu bar vs system tray
- Notarization for distribution

### 18. Community Features
**Impact:** Low | **Effort:** Medium

- Roast sharing (opt-in)
- Custom character skins
- Community prompts

---

## Implementation Order

### Phase 1: Foundation (P0)
1. OS Keyring implementation
2. Gemini default optimization
3. GPU detection
4. Privacy mode + blacklist

### Phase 2: UX Polish (P1)
5. First-launch onboarding
6. Improved context menu
7. Personality modes
8. Moondream support

### Phase 3: Expansion (P2)
9. Groq provider
10. Ollama integration
11. Error handling improvements
12. Windows notifications

### Phase 4: Nice-to-Have (P3+)
13. Auto-updater
14. Analytics
15. Accessibility
16. Community features

---

## Quick Wins (Do Today)

These can be implemented in under 30 minutes:

1. **Change default model to gemini-2.0-flash** (free tier)
2. **Add `keyring` to requirements.txt**
3. **Create GPU detection function skeleton**
4. **Add fallback roast messages for API errors**
5. **Update context menu with keyboard shortcuts**

---

## Estimated Timeline

| Phase | Duration | Features |
|-------|----------|----------|
| Phase 1 | 1 week | P0 - Critical |
| Phase 2 | 1 week | P1 - High Priority |
| Phase 3 | 1-2 weeks | P2 - Medium Priority |
| Phase 4 | Ongoing | P3+ - Nice to have |

**Total MVP with all P0-P1:** ~2 weeks
