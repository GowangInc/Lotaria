# Lotaria Local Models Guide

Complete guide to using local vision and TTS models with Lotaria.

---

## Quick Start

### 1. Check Your Hardware

```python
from lotaria_model_db import get_quick_recommendation

# Detect your GPU VRAM (or specify manually)
recommendation = get_quick_recommendation(vram_gb=4.0)
print(f"Vision: {recommendation['vision'].name}")
print(f"TTS: {recommendation['tts'].name}")
```

### 2. Use Curated Presets (Recommended)

```python
from lotaria_models_curated import recommend_set, UserProfile

# Get recommendation based on your setup
preset = recommend_set(vram_gb=4.0, profile=UserProfile.BALANCED)
print(preset.install_command)
```

### 3. Direct Integration

```python
from lotaria_local_models_integration import LocalVisionService, LocalTTSService

# Vision
vision = LocalVisionService("moondream", backend="ollama")
result = vision.analyze(image_bytes, "What's on the screen?")

# TTS
tts = LocalTTSService("piper", voice="en_US-lessac-medium")
audio = tts.synthesize("Hello from Lotaria!")
```

---

## Curated Model Sets

We've curated 6 model combinations to avoid overwhelming users:

| Set ID | Profile | VRAM | Vision | TTS | Setup |
|--------|---------|------|--------|-----|-------|
| `minimal` | Easiest | 2.5GB | Moondream (Ollama) | Piper | ⭐⭐⭐⭐⭐ |
| `balanced` | Balanced | 4GB | Qwen2.5-VL 3B | Piper | ⭐⭐⭐⭐ |
| `balanced-ollama` | Balanced | 6GB | LLaVA-Llama3 | Piper | ⭐⭐⭐⭐⭐ |
| `quality` | Quality | 8GB | Qwen2.5-VL 7B | MeloTTS | ⭐⭐⭐ |
| `quality-clone` | Premium | 10GB | Qwen2.5-VL 7B | XTTS v2 | ⭐⭐ |
| `privacy` | Privacy | 2.5GB | Moondream (Ollama) | Piper | ⭐⭐⭐⭐⭐ |

### Installation Examples

#### Minimal Setup (Easiest)
```bash
# 1. Install Ollama from https://ollama.com
# 2. Pull vision model
ollama pull moondream

# 3. Install Piper
pip install piper-tts
piper-download --voice en_US-lessac-medium
```

#### Balanced Quality
```bash
# 1. Install PyTorch with CUDA
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu126

# 2. Install transformers and utils
pip install transformers accelerate qwen-vl-utils

# 3. Install Piper
pip install piper-tts
piper-download --voice en_US-lessac-medium
```

#### Quality Setup
```bash
# 1. Install PyTorch
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu126

# 2. Install vision dependencies
pip install transformers accelerate qwen-vl-utils

# 3. Install MeloTTS
pip install melotts
```

---

## Model Database

### Vision Models

| Model | VRAM | Size | Ollama | Best For |
|-------|------|------|--------|----------|
| **Moondream 2B** | 2.5GB | 1.6GB | ✅ | Fast, edge devices |
| **Qwen2.5-VL 3B** | 4GB | 2.5GB | ❌ | Quality, OCR |
| **Qwen2.5-VL 7B** | 7GB | 5.5GB | ❌ | Best quality |
| **LLaVA 7B** | 5GB | 4.5GB | ✅ | General purpose |
| **LLaVA-Llama3 8B** | 6GB | 4.9GB | ✅ | Better reasoning |
| **BakLLaVA 7B** | 5GB | 4.1GB | ✅ | Fast, efficient |
| **MiniCPM-V 2.6** | 4GB | 5.5GB | ✅ | OCR specialist |
| **SmolVLM 2.2B** | 2GB | 1.3GB | ❌ | Ultra-light |
| **InternVL2 4B** | 4GB | 3.0GB | ❌ | Balanced |
| **Gemma 3 4B** | 3GB | 3.3GB | ✅ | Google ecosystem |

### TTS Models

| Model | VRAM | Size | CPU | Best For |
|-------|------|------|-----|----------|
| **Piper** | 0GB | 0.1GB | ✅ | Fast, edge |
| **MeloTTS** | 2GB | 0.2GB | ✅ | Natural prosody |
| **Parler TTS** | 4GB | 2.0GB | ❌ | Production quality |
| **XTTS v2** | 4GB | 1.5GB | ❌ | Voice cloning |
| **ChatTTS** | 4GB | 1.0GB | ✅ | Conversational |
| **Orpheus TTS** | 6GB | 3.0GB | ❌ | Emotional, human-like |
| **Kokoro** | 0GB | 0.3GB | ✅ | Ultra-fast |

---

## Download Sources

### Primary: HuggingFace

```python
from lotaria_model_db import ModelDatabase, ModelDownloader

db = ModelDatabase()
downloader = ModelDownloader()

# Download from HuggingFace (default)
downloader.download_model("moondream", backend="huggingface")
```

### Fallback: ModelScope (China)

```python
# Automatic fallback if HuggingFace fails
downloader.download_model("qwen2.5-vl-3b", backend="huggingface", fallback=True)

# Or use ModelScope directly
downloader.download_model("qwen2.5-vl-3b", backend="modelscope")
```

### Simplest: Ollama

```python
# For Ollama-supported models
downloader.download_model("moondream", backend="ollama")

# Or use CLI
# ollama pull moondream
```

---

## CLI Usage

### Model Database CLI

```bash
# List all models
python lotaria_model_db.py list

# Download a model
python lotaria_model_db.py download moondream --backend=ollama

# Get recommendations for your VRAM
python lotaria_model_db.py recommend 4.0

# Export catalog to JSON
python lotaria_model_db.py export models.json
```

### Curated Sets CLI

```bash
# List curated sets
python lotaria_models_curated.py list

# Get detailed info
python lotaria_models_curated.py info minimal

# Interactive recommendation
python lotaria_models_curated.py recommend
```

### Integration CLI

```bash
# Check available models
python lotaria_local_models_integration.py check

# Test vision model
python lotaria_local_models_integration.py test-vision moondream screenshot.png

# Test TTS model
python lotaria_local_models_integration.py test-tts piper "Hello World"
```

---

## Hardware Requirements

### Minimum (2-3GB VRAM)
- **Vision**: SmolVLM 2.2B or Moondream 2B
- **TTS**: Piper (CPU)
- **Use case**: Basic functionality, edge devices

### Recommended (4-6GB VRAM)
- **Vision**: Qwen2.5-VL 3B or MiniCPM-V 2.6
- **TTS**: Piper or MeloTTS
- **Use case**: Good quality, reasonable speed

### High-End (8GB+ VRAM)
- **Vision**: Qwen2.5-VL 7B
- **TTS**: XTTS v2 or Orpheus TTS
- **Use case**: Best quality, voice cloning

### CPU-Only
- **Vision**: Not recommended (too slow)
- **TTS**: Piper, MeloTTS, or Kokoro
- **Use case**: No GPU available

---

## Integration with Lotaria

### Option 1: Use Curated Presets

```python
# In your Lotaria config
from lotaria_models_curated import recommend_set, UserProfile

def get_local_config():
    vram = detect_vram()  # Your VRAM detection
    preset = recommend_set(vram, UserProfile.BALANCED)
    
    return {
        "vision_model": preset.vision_id,
        "vision_backend": preset.install_method,
        "tts_model": preset.tts_id,
        "tts_voice": "en_US-lessac-medium"  # Default
    }
```

### Option 2: Direct Service Integration

```python
# In your Lotaria services
from lotaria_local_models_integration import (
    LocalVisionService, 
    LocalTTSService,
    get_recommended_services
)

class LocalVisionProvider:
    def __init__(self, model_id="moondream"):
        self.service = LocalVisionService(model_id, backend="auto")
    
    def analyze(self, image_bytes, prompt):
        return self.service.analyze(image_bytes, prompt)

class LocalTTSProvider:
    def __init__(self, model_id="piper", voice=None):
        self.service = LocalTTSService(model_id, voice=voice)
    
    def synthesize(self, text):
        return self.service.synthesize(text)
```

### Option 3: Factory Pattern

```python
from lotaria_local_models_integration import (
    create_vision_service,
    create_tts_service,
    get_recommended_services
)

def create_local_providers(vram_gb):
    rec = get_recommended_services(vram_gb)
    
    vision = create_vision_service(
        rec["vision"]["model_id"],
        backend=rec["vision"]["backend"]
    )
    
    tts = create_tts_service(
        rec["tts"]["model_id"],
        voice=rec["tts"].get("voice")
    )
    
    return vision, tts
```

---

## Troubleshooting

### HuggingFace Download Fails

```python
# Use ModelScope fallback
downloader.download_model("qwen2.5-vl-3b", fallback=True)

# Or set mirror
export HF_ENDPOINT=https://hf-mirror.com
```

### Ollama Not Found

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Or on Windows: https://ollama.com/download/windows
```

### CUDA Out of Memory

```python
# Use smaller model
vision = LocalVisionService("smolvlm")

# Or use quantization
# Models auto-detect and use int8/int4 if needed
```

### Piper Voice Not Found

```bash
# Download voice manually
piper-download --voice en_US-lessac-medium

# Or specify voice path
service = LocalTTSService("piper", voice="/path/to/voice.onnx")
```

---

## Model Comparison

### Vision Model Benchmarks

| Model | MMMU | TextVQA | Speed | VRAM |
|-------|------|---------|-------|------|
| GPT-4o | 69.9 | - | Cloud | - |
| Qwen2.5-VL 7B | 54.1 | 85.2 | Medium | 7GB |
| Qwen2.5-VL 3B | 48.0 | 78.5 | Fast | 4GB |
| LLaVA 7B | 33.4 | 65.0 | Medium | 5GB |
| Moondream 2B | 35.0 | 70.0 | Fast | 2.5GB |
| SmolVLM 2.2B | 32.0 | 60.0 | Fast | 2GB |

### TTS Quality Comparison

| Model | Naturalness | Speed | VRAM | Languages |
|-------|-------------|-------|------|-----------|
| Orpheus TTS | ⭐⭐⭐⭐⭐ | Medium | 6GB | EN |
| XTTS v2 | ⭐⭐⭐⭐⭐ | Medium | 4GB | 14+ |
| ChatTTS | ⭐⭐⭐⭐⭐ | Medium | 4GB | ZH, EN |
| Parler TTS | ⭐⭐⭐⭐ | Medium | 4GB | 6 |
| MeloTTS | ⭐⭐⭐⭐ | Fast | 2GB | 6 |
| Piper | ⭐⭐⭐ | Very Fast | 0GB | 30+ |
| Kokoro | ⭐⭐⭐ | Ultra Fast | 0GB | 3 |

---

## Advanced Usage

### Custom Model Registration

```python
from lotaria_model_db import ModelDatabase, ModelInfo, ModelType

db = ModelDatabase()

# Add custom model
custom_model = ModelInfo(
    id="my-custom-model",
    name="My Custom VLM",
    type=ModelType.VISION,
    description="Custom vision model",
    min_vram_gb=4.0,
    recommended_vram_gb=6.0,
    can_run_cpu=False,
    cpu_performance="poor",
    parameters="3B",
    model_size_gb=2.5,
    huggingface_repo="username/model-name",
    ollama_name=None
)

# Register (if extending the DB)
db._vision_models[custom_model.id] = custom_model
```

### Streaming Responses

```python
# For models that support streaming
vision = LocalVisionService("moondream")

# Stream the response
for chunk in vision.analyze(image_bytes, "Describe this", stream=True):
    print(chunk, end="")
```

### Voice Cloning with XTTS

```python
tts = LocalTTSService("xtts-v2")

# Clone a voice from sample
audio = tts.synthesize(
    "Hello, this is my cloned voice!",
    speaker_wav="/path/to/sample.wav"
)
```

---

## License & Attribution

All models listed are open-source with various licenses:

- **Apache 2.0**: Moondream, Qwen2.5-VL, MiniCPM-V, InternVL, SmolVLM, Orpheus TTS, Parler TTS
- **MIT**: Piper, MeloTTS, Kokoro
- **LLaMA**: LLaVA variants, BakLLaVA
- **Gemma**: Gemma 3
- **CPML**: XTTS v2
- **AGPL**: ChatTTS

Please respect the license terms of each model.

---

## Support

For issues or questions:

1. Check model-specific documentation on HuggingFace
2. Review Ollama documentation at https://ollama.com
3. Open an issue in the Lotaria repository
