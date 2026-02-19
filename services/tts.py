import os
import base64
import time
import wave
import subprocess
from io import BytesIO
from pathlib import Path
from abc import ABC, abstractmethod
from typing import Optional

from .state import config, TEMP_DIR, LOCAL_TTS_MODEL, API_TTS_MODEL


class BaseTTSService(ABC):
    model: str = "unknown"

    @abstractmethod
    def synthesize(self, text: str) -> tuple[bytes, str]:
        """Returns (wav_bytes, mime_type)."""
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


class APITTSService(BaseTTSService):
    def __init__(self):
        from google import genai
        api_key = os.environ.get("API_KEY") or os.environ.get("GOOGLE_API_KEY")
        if not api_key:
            raise ValueError("API_KEY or GOOGLE_API_KEY must be set for API TTS model")
        self.client = genai.Client(api_key=api_key)
        self.model = API_TTS_MODEL

    def synthesize(self, text: str) -> tuple[bytes, str]:
        from google.genai import types

        response = self.client.models.generate_content(
            model=self.model,
            contents=text,
            config=types.GenerateContentConfig(
                response_modalities=["AUDIO"],
                speech_config=types.SpeechConfig(
                    voice_config=types.VoiceConfig(
                        prebuilt_voice_config=types.PrebuiltVoiceConfig(voice_name="Kore")
                    )
                ),
            ),
        )
        part = response.candidates[0].content.parts[0]
        audio_data = part.inline_data.data
        mime_type = part.inline_data.mime_type or "audio/wav"

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
# Factory
# ---------------------------------------------------------------------------

_tts_service: Optional[BaseTTSService] = None


def get_tts_service() -> BaseTTSService:
    global _tts_service

    model_type = config["tts_model_type"]

    if _tts_service is not None:
        current_is_local = isinstance(_tts_service, LocalTTSService)
        if current_is_local != (model_type == "local"):
            _tts_service = None

    if _tts_service is None:
        if model_type == "local":
            _tts_service = LocalTTSService()
        else:
            _tts_service = APITTSService()
        print(f"[Lotaria] TTS service initialized: {_tts_service.model}")

    return _tts_service
