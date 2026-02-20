# Lotaria API Models Guide

Complete guide to cost-effective API providers for Lotaria's vision + TTS needs.

**Last Updated:** February 20, 2026

---

## Executive Summary

For all-day usage with Lotaria (screen capture + roast generation + TTS), here are the top recommendations:

| Rank | Provider | Daily Cost | Monthly Cost | Best For |
|------|----------|-----------|--------------|----------|
| 🥇 | **Google Gemini** | $0.02-0.15 | $0.50-4.50 | Best overall value, free tier |
| 🥈 | **Groq (Llama) + Kokoro** | $0.001 | $0.03 | Ultra-cheap TTS option |
| 🥉 | **Groq + OpenAI TTS** | $0.03-0.08 | $1.00-2.50 | Fastest inference |
| 4 | **DeepSeek + Kokoro** | $0.002 | $0.06 | Absolute cheapest |
| 5 | **OpenAI GPT-4o Mini** | $0.05-0.15 | $1.50-4.50 | Reliable, well-known |

**Key Insight:** Google Gemini 2.0 Flash offers a generous FREE tier (15 requests/min) that covers most Lotaria users entirely. For TTS, **Kokoro at $0.70/M characters** is the cheapest quality option.

---

## Top 10 Vision API Providers

### 1. Google Gemini (🏆 Best Overall Value)

| Model | Vision Input | Vision Output | Context |
|-------|-------------|---------------|---------|
| **Gemini 2.0 Flash** | **FREE** | **FREE** | 1M |
| **Gemini 2.5 Flash** | $0.15/M | $3.50/M | 2M |
| **Gemini 2.5 Pro** | $1.25-2.50/M | $10-15/M | 2M |

**Free Tier Limits:**
- Gemini 2.0 Flash: 15 requests/minute, 1,000 requests/day
- Gemini 2.5 Flash: 10 requests/minute, 250 requests/day

**Cost Calculator (30 roasts/day):**
```
Vision (2.0 Flash FREE): $0.00
Monthly: $0.00 (within free tier!)
```

**Pros:**
- ✅ Generous FREE tier covers most users
- ✅ Unified vision + TTS in one API
- ✅ No credit card required for free tier
- ✅ Fast inference
- ✅ Excellent vision quality

**Cons:**
- ❌ Rate limits on free tier
- ❌ Less "household name" recognition

**Setup:**
```python
export GEMINI_API_KEY="your-key"  # https://aistudio.google.com/app/apikey
```

---

### 2. Groq (⚡ Fastest & Cheapest for Open Models)

| Model | Input | Output | Speed |
|-------|-------|--------|-------|
| **Llama 3.1 8B** | $0.05/M | $0.08/M | 840+ TPS |
| **Llama 3.1 70B** | $0.59/M | $0.79/M | 250+ TPS |
| **Mixtral 8x7B** | $0.24/M | $0.24/M | Fast |

**Note:** Groq doesn't have native TTS. Pair with:
- **Kokoro TTS**: $0.70/M chars (cheapest!)
- **OpenAI TTS**: $15/M characters
- **Local Piper**: FREE

**Cost Calculator (30 roasts/day with Kokoro TTS):**
```
Vision (Llama 3.1 8B): ~500 tokens × 30 = 15K tokens = $0.00075/day
TTS (Kokoro): ~100 chars × 30 = 3,000 chars = $0.0021/day
Total: ~$0.003/day = ~$0.09/month
```

**Pros:**
- ✅ Fastest inference (840+ tokens/sec)
- ✅ Cheapest per token for open models
- ✅ No rate limits on paid tier
- ✅ OpenAI-compatible API

**Cons:**
- ❌ No native TTS (need separate provider)
- ❌ No free tier

**Setup:**
```python
export GROQ_API_KEY="your-key"  # https://console.groq.com
```

---

### 3. DeepSeek (💰 Lowest Cost Vision)

| Model | Input | Output | Context |
|-------|-------|--------|---------|
| **DeepSeek-V3.2** | $0.28/M | $0.42/M | 128K |
| **DeepSeek-VL (Vision)** | $0.30/M | $1.20/M | 128K |
| **Cache Hit** | $0.028/M | $0.42/M | - |

**Off-Peak Discount:** 75% off (12:30am-8:30am Beijing time)

**Cost Calculator (30 roasts/day with Kokoro TTS, off-peak):**
```
Vision (DeepSeek-VL): ~500 tokens × 30 = 15K tokens = $0.0045/day
TTS (Kokoro): ~100 chars × 30 = 3,000 chars = $0.0021/day
Total: ~$0.007/day = ~$0.21/month
```

**Pros:**
- ✅ Ultra-cheap API pricing
- ✅ Off-peak discounts
- ✅ Strong reasoning capabilities
- ✅ 128K context window

**Cons:**
- ❌ No native TTS
- ❌ China-based (potential latency)

**Setup:**
```python
export DEEPSEEK_API_KEY="your-key"  # https://platform.deepseek.com
```

---

### 4. OpenAI (🏢 Most Reliable)

| Model | Vision Input | Vision Output |
|-------|-------------|---------------|
| **GPT-4o Mini** | $0.15/M | $0.60/M |
| **GPT-4o** | $2.50/M | $10.00/M |

**Cost Calculator (30 roasts/day, GPT-4o Mini + TTS):**
```
Vision (GPT-4o Mini): ~500 tokens × 30 = 15K tokens = $0.00225/day
TTS (OpenAI TTS-1): ~100 chars × 30 = 3,000 chars = $0.045/day
Total: ~$0.047/day = ~$1.41/month
```

**Pros:**
- ✅ Most reliable uptime
- ✅ Best-known brand
- ✅ Excellent TTS quality
- ✅ Great documentation

**Cons:**
- ❌ No free API tier
- ❌ More expensive than alternatives

**Setup:**
```python
export OPENAI_API_KEY="your-key"  # https://platform.openai.com
```

---

### 5. Anthropic Claude (🧠 Best Quality, Most Expensive)

| Model | Input | Output | Context |
|-------|-------|--------|---------|
| **Claude Haiku 3.5** | $0.80/M | $4.00/M | 200K |
| **Claude Sonnet 4.5** | $3.00/M | $15.00/M | 200K |
| **Claude Opus 4.5** | $5.00/M | $25.00/M | 200K |

**Free Credits:** $5 for new accounts

**Cost Calculator (30 roasts/day, Sonnet 4.5 + OpenAI TTS):**
```
Vision (Sonnet 4.5): ~500 tokens × 30 = 15K tokens = $0.045/day
TTS (OpenAI TTS-1): ~100 chars × 30 = 3,000 chars = $0.045/day
Total: ~$0.09/day = ~$2.70/month
```

**Pros:**
- ✅ Best-in-class reasoning
- ✅ 200K context window
- ✅ Prompt caching (90% savings)

**Cons:**
- ❌ Most expensive
- ❌ No native TTS
- ❌ No free tier

---

### 6. Mistral AI (🇪🇺 European Alternative)

| Model | Input | Output | Context |
|-------|-------|--------|---------|
| **Pixtral 12B (Vision)** | $0.15/M | $0.15/M | 128K |
| **Mistral Large 2** | $2.00/M | $6.00/M | 128K |

**Free Tier:** Available via "la Plateforme"

**Cost Calculator (30 roasts/day, Pixtral + OpenAI TTS):**
```
Vision (Pixtral): ~500 tokens × 30 = 15K tokens = $0.00225/day
TTS (OpenAI TTS-1): ~100 chars × 30 = 3,000 chars = $0.045/day
Total: ~$0.047/day = ~$1.41/month
```

---

### 7. Fireworks AI

| Model Size | Input | Output |
|------------|-------|--------|
| **<4B params** | $0.10/M | $0.40/M |
| **4-16B params** | $0.20/M | $0.80/M |
| **>16B params** | $0.90/M | $3.60/M |

**Free Credits:** $1 for serverless inference

---

### 8. Together AI

| Model | Input | Output |
|-------|-------|--------|
| **Llama 3 8B** | $0.20/M | $0.20/M |
| **Llama 3 70B** | $0.90/M | $0.90/M |

---

### 9. OpenRouter (🌐 Universal Gateway)

OpenRouter aggregates multiple providers with a 5-5.5% platform fee.

**Pricing:** Provider rate + 5.5% fee

**Free Models:** Some models available for free (rate limited)

---

### 10. Cohere

| Model | Input | Output | Context |
|-------|-------|--------|---------|
| **Command R7B** | $0.0375/M | $0.15/M | 128K |
| **Command R** | $0.15/M | $0.60/M | 128K |

**Note:** No native vision or TTS.

---

## TTS API Providers (Complete List)

### 🏆 Best Value TTS Options

| Provider | Price/M chars | Quality | Speed | Best For |
|----------|--------------|---------|-------|----------|
| **Kokoro** | **$0.70** | ⭐⭐⭐ | Fast | Cheapest quality TTS |
| **Amazon Polly Standard** | **$4.00** | ⭐⭐⭐ | Fast | AWS integration |
| **OpenAI TTS-1** | **$15.00** | ⭐⭐⭐⭐ | Medium | Simple integration |
| **Deepgram Aura-2** | **$15.00** | ⭐⭐⭐⭐ | Sub-200ms | Enterprise voice AI |
| **Gemini TTS Flash** | **$10.00** | ⭐⭐⭐⭐ | Fast | Native Gemini |
| **Fish Audio** | **$15.00** | ⭐⭐⭐⭐ | Medium | Community voices |

### Premium TTS Options

| Provider | Price/M chars | Quality | Best For |
|----------|--------------|---------|----------|
| **ElevenLabs Multilingual v2** | ~$180-300 | ⭐⭐⭐⭐⭐ | Best quality, voice cloning |
| **ElevenLabs Turbo v2.5** | ~$60-75 | ⭐⭐⭐⭐⭐ | Fast, high quality |
| **OpenAI TTS HD** | $30.00 | ⭐⭐⭐⭐⭐ | High quality |
| **Azure Neural TTS** | $16.00 | ⭐⭐⭐⭐ | Enterprise |
| **Amazon Polly Neural** | $16.00 | ⭐⭐⭐⭐ | AWS ecosystem |
| **Google Cloud Neural** | $16.00 | ⭐⭐⭐⭐ | Google ecosystem |
| **Gemini TTS Pro** | $20.00 | ⭐⭐⭐⭐⭐ | Best Gemini quality |

### Detailed TTS Pricing

#### 1. Kokoro (🏆 Cheapest Quality TTS)
- **Price:** $0.70 per 1M characters
- **Quality:** Good (ELO 1,059)
- **Best For:** Budget-conscious users
- **Note:** Open-weight model, self-hosting option

#### 2. Amazon Polly
| Voice Type | Price/M chars | Free Tier |
|------------|--------------|-----------|
| Standard | $4.00 | 5M chars/mo (12mo) |
| Neural | $16.00 | 1M chars/mo (12mo) |
| Generative | $30.00 | 100K chars/mo |
| Long-Form | $100.00 | 500K chars/mo |

#### 3. OpenAI TTS
| Model | Price/M chars | Quality |
|-------|--------------|---------|
| TTS-1 | $15.00 | Good |
| TTS-1 HD | $30.00 | Excellent |
| GPT-4o mini TTS | ~$15.00 | Good |

#### 4. Deepgram Aura-2
- **Price:** $15.00 per 1M characters ($13.50 with Growth plan)
- **Latency:** Sub-200ms
- **Languages:** 10+ (English, Dutch, German, French, Italian, Japanese)
- **Best For:** Enterprise voice AI, real-time applications
- **Free Credits:** $200 to start

#### 5. Gemini TTS
| Model | Price/M chars | Voices |
|-------|--------------|--------|
| 2.5 Flash Preview | $10.00 | Kore, Fenrir, Leda, Puck, Zeus |
| 2.5 Pro Preview | $20.00 | Same voices, higher quality |

**Free Tier:** 3 requests/min, 15 requests/day

#### 6. ElevenLabs
| Plan | Price | Characters |
|------|-------|------------|
| Free | $0 | 10K/mo |
| Starter | $5/mo | 30K/mo |
| Creator | $22/mo | 100K/mo |
| Pro | $99/mo | 500K/mo |

**Pay-as-you-go:** ~$0.18-0.30 per 1K characters

#### 7. Azure Speech TTS
| Voice Type | Price/M chars | Free Tier |
|------------|--------------|-----------|
| Standard | $4.00 | 500K/mo |
| Neural | $16.00 | - |
| Custom Neural | $24.00 | - |

#### 8. Google Cloud TTS
| Voice Type | Price/M chars | Free Tier |
|------------|--------------|-----------|
| Standard | $4.00 | 4M chars/mo |
| WaveNet | $16.00 | 1M chars/mo |
| Neural | $16.00 | 1M chars/mo |
| Studio | $160.00 | - |

#### 9. Cartesia Sonic
| Plan | Price | Credits |
|------|-------|---------|
| Free | $0 | 20K/mo |
| Pro | $5/mo | 100K/mo |
| Startup | $49/mo | 1.25M/mo |
| Scale | $299/mo | 8M/mo |

**Effective rate:** ~$37-47 per 1M characters
**Speed:** 40-90ms time-to-first-audio (fastest!)

#### 10. Fish Audio
| Plan | Price | Features |
|------|-------|----------|
| Free | $0 | 500 chars/gen |
| Plus | $5.50/mo | 250K credits |
| Pro | $37.50/mo | 2M credits |

**API:** $15 per 1M characters

---

## Cost Comparison Summary

### Daily Usage Costs (30 roasts/day)

| Provider | Vision Cost | TTS Cost | Total/Day | Total/Month |
|----------|-------------|----------|-----------|-------------|
| **Gemini 2.0 Flash + Gemini TTS** | FREE | $0.03 | **$0.03** | **$0.90** |
| **Gemini 2.0 Flash + Kokoro** | FREE | $0.002 | **$0.002** | **$0.06** |
| **Groq + Kokoro** | $0.001 | $0.002 | **$0.003** | **$0.09** |
| **DeepSeek + Kokoro** | $0.004 | $0.002 | **$0.006** | **$0.18** |
| **Groq + OpenAI TTS** | $0.001 | $0.045 | **$0.046** | **$1.38** |
| **GPT-4o Mini + OpenAI TTS** | $0.002 | $0.045 | **$0.047** | **$1.41** |
| **Gemini 2.5 Flash + Gemini TTS** | $0.005 | $0.03 | **$0.035** | **$1.05** |
| **Claude Sonnet + OpenAI TTS** | $0.045 | $0.045 | **$0.09** | **$2.70** |
| **GPT-4o + TTS HD** | $0.038 | $0.09 | **$0.128** | **$3.84** |

### Heavy Usage (100 roasts/day)

| Provider | Daily Cost | Monthly Cost |
|----------|-----------|--------------|
| **Gemini 2.0 Flash + Gemini TTS** | $0.10 | $3.00 |
| **Gemini 2.0 Flash + Kokoro** | $0.007 | $0.21 |
| **Groq + Kokoro** | $0.01 | $0.30 |
| **DeepSeek + Kokoro** | $0.02 | $0.60 |
| **Groq + OpenAI TTS** | $0.15 | $4.50 |
| **GPT-4o Mini + OpenAI TTS** | $0.16 | $4.80 |

---

## Curated Recommendations by Use Case

### 💰 Budget-Conscious (Under $1/month)

**Winner: Gemini 2.0 Flash + Kokoro TTS**
```python
VISION_MODEL = "gemini-2.0-flash"  # FREE
TTS_MODEL = "kokoro"  # $0.70/M chars

# Expected cost: $0.06/month (30 roasts/day)
# Expected cost: $0.21/month (100 roasts/day)
```

**Alternative: Groq + Kokoro**
```python
VISION_PROVIDER = "groq"
VISION_MODEL = "llama-3.1-8b"  # $0.05/M
TTS_MODEL = "kokoro"  # $0.70/M

# Expected cost: $0.09/month (30 roasts/day)
```

---

### ⚖️ Balanced Quality & Cost ($1-3/month)

**Winner: Gemini 2.5 Flash + Gemini TTS**
```python
VISION_MODEL = "gemini-2.5-flash"  # $0.15/M
TTS_MODEL = "gemini-2.5-flash-preview-tts"  # $10/M
TTS_VOICE = "Kore"

# Expected cost: $1.05/month (30 roasts/day)
```

**Alternative: GPT-4o Mini + OpenAI TTS**
```python
VISION_MODEL = "gpt-4o-mini"  # $0.15/M
TTS_MODEL = "tts-1"  # $15/M
TTS_VOICE = "alloy"

# Expected cost: $1.41/month (30 roasts/day)
```

---

### 🎯 Best Quality (Under $5/month)

**Winner: Gemini 2.5 Pro + Gemini TTS Pro**
```python
VISION_MODEL = "gemini-2.5-pro"  # $1.25-2.50/M
TTS_MODEL = "gemini-2.5-pro-preview-tts"  # $20/M

# Expected cost: $2-4/month (30 roasts/day)
```

**Alternative: GPT-4o + OpenAI TTS HD**
```python
VISION_MODEL = "gpt-4o"  # $2.50/M
TTS_MODEL = "tts-1-hd"  # $30/M
TTS_VOICE = "nova"

# Expected cost: $3.84/month (30 roasts/day)
```

---

### 🚀 All-Day Heavy Usage (50-100 roasts/day)

**Winner: Gemini 2.0 Flash (Free) + Kokoro**
```python
VISION_MODEL = "gemini-2.0-flash"  # FREE
TTS_MODEL = "kokoro"  # $0.70/M

# 100 roasts/day = ~$0.21/month
```

---

## Quality vs Price Analysis

### Vision Model Quality Ranking

| Rank | Model | Quality | Price/M Input | Value Score |
|------|-------|---------|---------------|-------------|
| 1 | GPT-4o | ⭐⭐⭐⭐⭐ | $2.50 | ⭐⭐⭐ |
| 2 | Gemini 2.5 Pro | ⭐⭐⭐⭐⭐ | $1.25-2.50 | ⭐⭐⭐⭐ |
| 3 | Claude Sonnet 4.5 | ⭐⭐⭐⭐⭐ | $3.00 | ⭐⭐⭐ |
| 4 | Gemini 2.5 Flash | ⭐⭐⭐⭐ | $0.15 | ⭐⭐⭐⭐⭐ |
| 5 | GPT-4o Mini | ⭐⭐⭐⭐ | $0.15 | ⭐⭐⭐⭐⭐ |
| 6 | Gemini 2.0 Flash | ⭐⭐⭐⭐ | FREE | ⭐⭐⭐⭐⭐ |

### TTS Quality Ranking

| Rank | Provider | Quality | Price/M | Best For |
|------|----------|---------|---------|----------|
| 1 | ElevenLabs Multilingual | ⭐⭐⭐⭐⭐ | ~$180-300 | Premium quality |
| 2 | OpenAI TTS HD | ⭐⭐⭐⭐⭐ | $30 | High quality |
| 3 | Gemini TTS Pro | ⭐⭐⭐⭐⭐ | $20 | Best Gemini quality |
| 4 | Deepgram Aura-2 | ⭐⭐⭐⭐ | $15 | Enterprise |
| 5 | OpenAI TTS-1 | ⭐⭐⭐⭐ | $15 | Good balance |
| 6 | Gemini TTS Flash | ⭐⭐⭐⭐ | $10 | Native integration |
| 7 | Kokoro | ⭐⭐⭐ | $0.70 | Cheapest quality |

---

## Recommended Combinations

### The "Free Forever" Setup
```python
# Vision: Gemini 2.0 Flash (FREE)
# TTS: Local Piper (FREE)
# Total: $0/month

config = {
    "vision_provider": "gemini",
    "vision_model": "gemini-2.0-flash",
    "tts_provider": "local",
    "tts_model": "piper",
    "tts_voice": "en_US-lessac-medium"
}
```

### The "Under $1" Setup (Recommended!)
```python
# Vision: Gemini 2.0 Flash (FREE)
# TTS: Kokoro ($0.70/M chars)
# Total: ~$0.06/month (30 roasts/day)

config = {
    "vision_provider": "gemini",
    "vision_model": "gemini-2.0-flash",
    "tts_provider": "kokoro",
    "tts_model": "kokoro-82m"
}
```

### The "Best Value" Setup
```python
# Vision: Gemini 2.5 Flash ($0.15/M)
# TTS: Gemini TTS ($10/M chars)
# Total: ~$1.05/month (30 roasts/day)

config = {
    "vision_provider": "gemini",
    "vision_model": "gemini-2.5-flash",
    "tts_provider": "gemini",
    "tts_model": "gemini-2.5-flash-preview-tts",
    "tts_voice": "Kore"
}
```

### The "Premium" Setup
```python
# Vision: GPT-4o ($2.50/M)
# TTS: OpenAI TTS HD ($30/M chars)
# Total: ~$3.84/month (30 roasts/day)

config = {
    "vision_provider": "openai",
    "vision_model": "gpt-4o",
    "tts_provider": "openai",
    "tts_model": "tts-1-hd",
    "tts_voice": "nova"
}
```

---

## Rate Limits Comparison

| Provider | Free Tier RPM | Paid Tier RPM | Notes |
|----------|---------------|---------------|-------|
| **Gemini 2.0 Flash** | 15 | 1,000 | Very generous |
| **Gemini 2.5 Flash** | 10 | 1,000 | Good for most |
| **OpenAI** | N/A | 500-10,000 | No free tier |
| **Anthropic** | N/A | 50-5,000 | No free tier |
| **Groq** | N/A | Unlimited | Very fast |
| **DeepSeek** | 10 | 100+ | Off-peak discounts |

---

## Quick Decision Tree

```
Do you want to pay $0?
├── YES → Gemini 2.0 Flash + Local Piper TTS
│         └── Or Gemini 2.0 Flash + Kokoro (~$0.06/month)
│
└── NO → What's your priority?
          ├── Cheapest overall → DeepSeek + Kokoro
          ├── Fastest → Groq + Kokoro
          ├── Best quality → GPT-4o + OpenAI TTS HD
          ├── Easiest setup → Gemini (unified API)
          └── Best TTS quality → Gemini + ElevenLabs
```

---

## API Key Setup Guide

### Google Gemini
```bash
# https://aistudio.google.com/app/apikey
export GEMINI_API_KEY="your-key-here"
```

### OpenAI
```bash
# https://platform.openai.com/api-keys
export OPENAI_API_KEY="your-key-here"
```

### Groq
```bash
# https://console.groq.com/keys
export GROQ_API_KEY="your-key-here"
```

### Anthropic
```bash
# https://console.anthropic.com/settings/keys
export ANTHROPIC_API_KEY="your-key-here"
```

### DeepSeek
```bash
# https://platform.deepseek.com/api_keys
export DEEPSEEK_API_KEY="your-key-here"
```

### OpenRouter
```bash
# https://openrouter.ai/keys
export OPENROUTER_API_KEY="your-key-here"
```

---

## Final Recommendations

| Use Case | Recommended Provider | Expected Monthly Cost |
|----------|---------------------|----------------------|
| **Free forever** | Gemini 2.0 Flash + Piper | $0 |
| **Under $1** | Gemini 2.0 Flash + Kokoro | $0.06-0.21 |
| **Best value** | Gemini 2.5 Flash + Gemini TTS | $1-4 |
| **Fastest** | Groq + Kokoro | $0.09-0.30 |
| **Best quality** | GPT-4o + OpenAI TTS HD | $2.50-8 |
| **Heavy usage** | Gemini 2.0 Flash + Kokoro | $0.21 |

**Bottom Line:** For Lotaria, start with **Google Gemini 2.0 Flash + Kokoro TTS**. The free tier of Gemini covers vision entirely, and Kokoro at $0.70/M is the cheapest quality TTS option. Total cost: under $0.10/month for typical usage!
