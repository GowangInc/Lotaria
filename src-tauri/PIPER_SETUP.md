# Piper TTS Setup

Lotaria can bundle Piper TTS for offline, high-quality local text-to-speech.

**Note**: We use the original Piper (rhasspy/piper) which provides standalone binaries, not Piper1 GPL which is Python-only.

## Quick Setup

### 1. Download Piper Binary

Download the appropriate binary for your platform from [Piper Releases](https://github.com/rhasspy/piper/releases/tag/2023.11.14-2):

**Windows (64-bit)**:
- Download: `piper_windows_amd64.tar.gz`
- Extract the entire archive (includes piper.exe and required DLLs)
- Copy ALL files from the extracted `piper/` folder to `src-tauri/binaries/`
- You should have: `piper.exe`, `espeak-ng-data/` folder, and any DLL files

**macOS (Intel)**:
- Download: `piper_macos_x64.tar.gz`
- Extract the entire archive
- Copy ALL files from the extracted folder to `src-tauri/binaries/`
- Make executable: `chmod +x src-tauri/binaries/piper`

**macOS (Apple Silicon)**:
- Download: `piper_macos_arm64.tar.gz`
- Extract the entire archive
- Copy ALL files from the extracted folder to `src-tauri/binaries/`
- Make executable: `chmod +x src-tauri/binaries/piper`

**Linux (64-bit)**:
- Download: `piper_linux_x86_64.tar.gz`
- Extract the entire archive
- Copy ALL files from the extracted folder to `src-tauri/binaries/`
- Make executable: `chmod +x src-tauri/binaries/piper`

**Important**: Piper requires additional files beyond just the executable:
- `espeak-ng-data/` folder (phoneme data)
- Various DLL/shared library files
- Extract the ENTIRE archive contents, not just the binary

### 2. Download Default Voice Model

Download the default voice (en_US-danny-low, ~10MB) from [Piper Voices on Hugging Face](https://huggingface.co/rhasspy/piper-voices/tree/main/en/en_US/en_US-danny-low):

1. Download `en_US-danny-low.onnx` (~10MB)
2. Download `en_US-danny-low.onnx.json` (~1KB)
3. Place both files in `src-tauri/models/`

### 3. Directory Structure

After setup, you should have:

```
src-tauri/
├── binaries/
│   └── piper.exe (or piper on Unix)
└── models/
    ├── en_US-danny-low.onnx
    └── en_US-danny-low.onnx.json
```

## Additional Voices

Users can download additional voices at runtime. They will be cached in:
- Windows: `%LOCALAPPDATA%\lotaria\piper\`
- macOS/Linux: `~/.cache/lotaria/piper/`

Available voices:
- `en_US-lessac-medium` (~30MB) - Clear, professional
- `en_US-amy-medium` (~30MB) - Warm, friendly
- `en_US-danny-low` (~10MB) - Fast, lightweight (bundled)
- `en_US-joe-medium` (~30MB) - Deep, authoritative
- `en_GB-alan-medium` (~30MB) - British accent
- `en_GB-jenny_dioco-medium` (~30MB) - British female

## Testing

To test Piper locally before bundling:

```bash
# Test synthesis
echo "Hello world" | ./src-tauri/binaries/piper \
  --model ./src-tauri/models/en_US-danny-low.onnx \
  --output_file test.wav

# Play the result
# Windows: start test.wav
# macOS: afplay test.wav
# Linux: aplay test.wav
```

## Bundling

The Tauri config (`tauri.conf.json`) is already set up to bundle:
- Binary: `src-tauri/binaries/piper` → bundled as sidecar
- Models: Place in `src-tauri/models/` → bundled as resources

When building the app, these will be included automatically.
