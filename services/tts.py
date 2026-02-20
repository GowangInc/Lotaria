import asyncio
import os
import time
import wave
import subprocess
from io import BytesIO
from pathlib import Path
from abc import ABC, abstractmethod
from typing import Optional

from .state import config, TEMP_DIR, LOCAL_TTS_MODEL

import litellm
litellm.set_verbose = False
litellm.suppress_debug_info = True


class BaseTTSService(ABC):
    model: str = "unknown"

    @abstractmethod
    def synthesize(self, text: str) -> tuple[bytes, str]:
        """Returns (audio_bytes, mime_type)."""
        pass


class LocalTTSService(BaseTTSService):
    def __init__(self):
        self.model = LOCAL_TTS_MODEL
        self._voice_name = "en_US-lessac-medium"

        try:
            import piper
            self._piper_available = True
            print("[Lotaria] Piper TTS available via Python package")
        except ImportError:
            self._piper_available = False
            print("[Lotaria] Piper Python package not found, will use subprocess")

    def synthesize(self, text: str) -> tuple[bytes, str]:
        timestamp = int(time.time())
        output_file = TEMP_DIR / f"audio_{timestamp}.wav"

        if self._piper_available:
            return self._synthesize_python(text, output_file)
        return self._synthesize_subprocess(text, output_file)

    def _synthesize_python(self, text: str, output_file: Path) -> tuple[bytes, str]:
        from piper import PiperVoice

        voice = PiperVoice.load(self._voice_name)
        wav_buffer = BytesIO()
        with wave.open(wav_buffer, "wb") as wav_file:
            voice.synthesize(text, wav_file)

        audio_bytes = wav_buffer.getvalue()
        output_file.write_bytes(audio_bytes)
        return audio_bytes, "audio/wav"

    def _synthesize_subprocess(self, text: str, output_file: Path) -> tuple[bytes, str]:
        try:
            result = subprocess.run(
                ["piper", "--model", self._voice_name, "--output_file", str(output_file)],
                input=text.encode(), capture_output=True, timeout=30,
            )
            if result.returncode != 0:
                raise RuntimeError(f"Piper failed: {result.stderr.decode()}")
            audio_bytes = output_file.read_bytes()
            return audio_bytes, "audio/wav"
        except FileNotFoundError:
            raise RuntimeError(
                "Piper not found. Install via: pip install piper-tts\n"
                "Or download from: https://github.com/rhasspy/piper/releases"
            )


class LiteLLMTTSService(BaseTTSService):
    def __init__(self, model: str, voice: str, api_key: str = None):
        self.model = model
        self.voice = voice
        self.api_key = api_key

    def synthesize(self, text: str) -> tuple[bytes, str]:
        kwargs = {
            "model": self.model,
            "voice": self.voice,
            "input": text,
        }
        if self.api_key:
            kwargs["api_key"] = self.api_key

        response = litellm.speech(**kwargs)

        # litellm.speech() returns an HttpxBinaryResponseContent with .read()
        if hasattr(response, "read"):
            audio_bytes = response.read()
        elif hasattr(response, "content"):
            audio_bytes = response.content
        else:
            audio_bytes = bytes(response)

        timestamp = int(time.time())
        # OpenAI TTS returns mp3 by default
        ext = "mp3" if "openai" in self.model else "wav"
        audio_file = TEMP_DIR / f"audio_{timestamp}.{ext}"
        audio_file.write_bytes(audio_bytes)

        mime_type = "audio/mpeg" if ext == "mp3" else "audio/wav"
        return audio_bytes, mime_type


class GeminiTTSService(BaseTTSService):
    """Gemini TTS uses the google-genai SDK directly (not LiteLLM)
    because Gemini TTS works via generate_content with audio modality,
    not the standard OpenAI-compatible /audio/speech endpoint."""

    def __init__(self, model: str, voice: str, api_key: str = None):
        from google import genai
        key = api_key or __import__("os").environ.get("GEMINI_API_KEY") or \
              __import__("os").environ.get("API_KEY") or \
              __import__("os").environ.get("GOOGLE_API_KEY")
        if not key:
            raise ValueError("Gemini API key required for Gemini TTS")
        self.client = genai.Client(api_key=key)
        self.model = model.replace("gemini/", "")  # genai wants bare model name
        self.voice = voice

    def synthesize(self, text: str) -> tuple[bytes, str]:
        from google.genai import types

        response = self.client.models.generate_content(
            model=self.model,
            contents=text,
            config=types.GenerateContentConfig(
                response_modalities=["AUDIO"],
                speech_config=types.SpeechConfig(
                    voice_config=types.VoiceConfig(
                        prebuilt_voice_config=types.PrebuiltVoiceConfig(voice_name=self.voice)
                    )
                ),
            ),
        )
        part = response.candidates[0].content.parts[0]
        audio_data = part.inline_data.data
        mime_type = part.inline_data.mime_type or "audio/wav"

        import base64
        if isinstance(audio_data, str):
            audio_data = base64.b64decode(audio_data)

        if "L16" in mime_type or "pcm" in mime_type:
            sample_rate = 24000
            if "rate=" in mime_type:
                rate_part = mime_type.split("rate=")[1].split(";")[0]
                sample_rate = int(rate_part)

            wav_buffer = BytesIO()
            with wave.open(wav_buffer, "wb") as wav_file:
                wav_file.setnchannels(1)
                wav_file.setsampwidth(2)
                wav_file.setframerate(sample_rate)
                wav_file.writeframes(audio_data)
            audio_data = wav_buffer.getvalue()
            mime_type = "audio/wav"

        timestamp = int(time.time())
        audio_file = TEMP_DIR / f"audio_{timestamp}.wav"
        audio_file.write_bytes(audio_data)

        return audio_data, mime_type


class GeminiLiveTTSService(BaseTTSService):
    """Gemini TTS via the Live API (Native Audio Dialog).
    Uses the unlimited free-tier Live API instead of the rate-limited TTS endpoint."""

    LIVE_MODEL = "gemini-2.5-flash-native-audio-preview-12-2025"

    def __init__(self, voice: str = "Kore", api_key: str = None):
        from google import genai
        key = api_key or os.environ.get("GEMINI_API_KEY") or \
              os.environ.get("API_KEY") or os.environ.get("GOOGLE_API_KEY")
        if not key:
            raise ValueError("Gemini API key required for Gemini Live TTS")
        self.client = genai.Client(api_key=key)
        self.model = self.LIVE_MODEL
        self.voice = voice

    def synthesize(self, text: str) -> tuple[bytes, str]:
        try:
            loop = asyncio.get_running_loop()
        except RuntimeError:
            loop = None

        if loop and loop.is_running():
            # Already in an async context — run in a new thread
            import concurrent.futures
            with concurrent.futures.ThreadPoolExecutor() as pool:
                future = pool.submit(asyncio.run, self._synthesize_async(text))
                return future.result(timeout=30)
        else:
            return asyncio.run(self._synthesize_async(text))

    async def _synthesize_async(self, text: str) -> tuple[bytes, str]:
        from google.genai import types

        live_config = types.LiveConnectConfig(
            response_modalities=["AUDIO"],
            speech_config=types.SpeechConfig(
                voice_config=types.VoiceConfig(
                    prebuilt_voice_config=types.PrebuiltVoiceConfig(voice_name=self.voice)
                )
            ),
        )

        audio_chunks: list[bytes] = []
        async with self.client.aio.live.connect(
            model=self.LIVE_MODEL, config=live_config
        ) as session:
            await session.send_client_content(
                turns=[{
                    "role": "user",
                    "parts": [{"text": f"Read the following text aloud naturally: {text}"}],
                }],
                turn_complete=True,
            )
            async for response in session.receive():
                if response.server_content and response.server_content.model_turn:
                    for part in response.server_content.model_turn.parts:
                        if part.inline_data and isinstance(part.inline_data.data, bytes):
                            audio_chunks.append(part.inline_data.data)
                if response.server_content and response.server_content.turn_complete:
                    break

        raw_audio = b"".join(audio_chunks)
        if not raw_audio:
            raise RuntimeError("Gemini Live API returned no audio data")

        # Convert raw PCM (24kHz, 16-bit, mono) to WAV
        wav_buffer = BytesIO()
        with wave.open(wav_buffer, "wb") as wf:
            wf.setnchannels(1)
            wf.setsampwidth(2)
            wf.setframerate(24000)
            wf.writeframes(raw_audio)
        audio_data = wav_buffer.getvalue()

        timestamp = int(time.time())
        audio_file = TEMP_DIR / f"audio_{timestamp}.wav"
        audio_file.write_bytes(audio_data)

        return audio_data, "audio/wav"


# ---------------------------------------------------------------------------
# Factory (lazy singleton, recreated when model/provider changes)
# ---------------------------------------------------------------------------

_tts_service: Optional[BaseTTSService] = None
_tts_service_key: Optional[str] = None


def get_tts_service() -> BaseTTSService:
    global _tts_service, _tts_service_key

    provider = config["tts_provider"]
    model = config["tts_model"]
    voice = config.get("tts_voice", "")
    current_key = f"{provider}:{model}:{voice}"

    if _tts_service is not None and _tts_service_key != current_key:
        _tts_service = None

    if _tts_service is None:
        if provider == "local":
            _tts_service = LocalTTSService()
        elif provider == "gemini" and "live" in model:
            api_key = config.get("api_keys", {}).get("gemini")
            _tts_service = GeminiLiveTTSService(voice=voice or "Kore", api_key=api_key)
        elif provider == "gemini":
            api_key = config.get("api_keys", {}).get("gemini")
            _tts_service = GeminiTTSService(model=model, voice=voice or "Kore", api_key=api_key)
        else:
            api_key = config.get("api_keys", {}).get(provider)
            _tts_service = LiteLLMTTSService(model=model, voice=voice or "alloy", api_key=api_key)
        _tts_service_key = current_key
        print(f"[Lotaria] TTS service initialized: {model} (voice={voice})")

    return _tts_service
