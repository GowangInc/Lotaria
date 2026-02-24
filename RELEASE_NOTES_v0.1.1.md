# v0.1.1 - FoxCode Multi-Provider Support

## What's New in v0.1.1

### FoxCode Multi-Provider Support
- Added FoxCode as a multi-provider proxy supporting:
  - **Gemini models**: gemini-3-pro, gemini-3-flash, gemini-2.5-*
  - **Codex models**: gpt-5.3-codex, gpt-5.2, gpt-5.1, gpt-5
  - **Claude models**: claude-sonnet-4-6, claude-opus-4-*, thinking variants
- Automatic routing to correct endpoint based on model prefix
- Single API key works across all providers
- Cost: ~¥0.03-0.35 per million tokens

### Eye Tracking Improvements
- Fixed independent eye tracking for cross-eyed effect
- Eyes now track cursor independently when mouse is between them
- Fixed owl's eyes not moving (removed animation conflict)

### Bug Fixes
- Fixed FoxCode 401 authentication error with proper x-api-key header

## Installation

Download and run either installer:
- **Lotaria_0.1.1_x64_en-US.msi** - Windows MSI installer
- **Lotaria_0.1.1_x64-setup.exe** - NSIS installer

## Requirements
- Windows 10/11
- API key from supported provider (Gemini, FoxCode, OpenAI, etc.)
