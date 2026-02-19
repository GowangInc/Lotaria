import time
import wave
import subprocess
from io import BytesIO
from pathlib import Path
from abc import ABC, abstractmethod
from typing import Optional

from .state import config, TEMP_DIR, LOCAL_TTS_MODEL


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
        import litellm
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
        elif provider == "gemini":
            api_key = config.get("api_keys", {}).get("gemini")
            _tts_service = GeminiTTSService(model=model, voice=voice or "Kore", api_key=api_key)
        else:
            api_key = config.get("api_keys", {}).get(provider)
            _tts_service = LiteLLMTTSService(model=model, voice=voice or "alloy", api_key=api_key)
        _tts_service_key = current_key
        print(f"[Lotaria] TTS service initialized: {model} (voice={voice})")

    return _tts_service
