import asyncio
import os
import time
import wave
import subprocess
from io import BytesIO
from pathlib import Path
from abc import ABC, abstractmethod
from typing import Optional

from .state import config, TEMP_DIR, LOCAL_TTS_MODEL, MODELS_DIR, PIPER_VOICES
from .downloader import ModelDownloader

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
    def __init__(self, voice: str = "en_US-lessac-medium"):
        self.model = LOCAL_TTS_MODEL
        self._voice_name = voice
        self._voice_path = None
        
        try:
            import piper
            self._piper_available = True
            print("[Lotaria] Piper TTS available via Python package")
        except ImportError:
            self._piper_available = False
            print("[Lotaria] Piper Python package not found, will use subprocess")

    def _ensure_voice(self):
        if self._voice_path and self._voice_path.exists():
            return self._voice_path

        # Handle the default voice or custom path
        if self._voice_name in PIPER_VOICES or self._voice_name == "en_US-lessac-medium":
            from huggingface_hub import hf_hub_download
            repo = "rhasspy/piper-voices"
            
            # Map friendly names to paths if needed, though we use full identifiers
            # en_US-lessac-medium -> en/en_US/lessac/medium/en_US-lessac-medium.onnx
            parts = self._voice_name.split("-")
            lang_short = parts[0].split("_")[0] # en
            lang_full = parts[0] # en_US
            name = parts[1] # lessac
            quality = parts[2] # medium
            
            filename = f"{lang_short}/{lang_full}/{name}/{quality}/{self._voice_name}.onnx"
            
            print(f"[Lotaria] Downloading/Ensuring Piper voice: {self._voice_name}")
            
            # Download to our app's models directory
            target_dir = MODELS_DIR / "piper"
            target_dir.mkdir(parents=True, exist_ok=True)
            
            onnx_path = hf_hub_download(
                repo_id=repo,
                filename=filename,
                local_dir=target_dir,
                local_dir_use_symlinks=False
            )
            # Also need the config file or Piper will crash
            hf_hub_download(
                repo_id=repo,
                filename=filename + ".json",
                local_dir=target_dir,
                local_dir_use_symlinks=False
            )
            
            self._voice_path = Path(onnx_path)
            return self._voice_path
        
        # If it's already a path, use it
        p = Path(self._voice_name)
        if p.exists():
            self._voice_path = p
            return p
            
        raise FileNotFoundError(f"Piper voice model not found: {self._voice_name}")

    def synthesize(self, text: str) -> tuple[bytes, str]:
        timestamp = int(time.time())
        output_file = TEMP_DIR / f"audio_{timestamp}.wav"

        if self._piper_available:
            return self._synthesize_python(text, output_file)
        return self._synthesize_subprocess(text, output_file)

    def _synthesize_python(self, text: str, output_file: Path) -> tuple[bytes, str]:
        from piper import PiperVoice
        voice_path = self._ensure_voice()
        voice = PiperVoice.load(str(voice_path))
        wav_buffer = BytesIO()
        with wave.open(wav_buffer, "wb") as wav_file:
            voice.synthesize(text, wav_file)

        audio_bytes = wav_buffer.getvalue()
        output_file.write_bytes(audio_bytes)
        return audio_bytes, "audio/wav"

    def _synthesize_subprocess(self, text: str, output_file: Path) -> tuple[bytes, str]:
        try:
            voice_path = self._ensure_voice()
            result = subprocess.run(
                ["piper", "--model", str(voice_path), "--output_file", str(output_file)],
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


class KokoroTTSService(BaseTTSService):
    """Uses the Kokoro-82M model via kokoro-onnx.
    No PyTorch/Torch dependency = No Windows DLL errors.
    """
    def __init__(self, voice: str = "af_heart"):
        self.model = "kokoro"
        self.voice = voice
        self._kokoro = None

    def _ensure_kokoro(self):
        if self._kokoro is None:
            from kokoro_onnx import Kokoro
            from huggingface_hub import hf_hub_download
            
            # Ensure model and voices are downloaded
            target_dir = MODELS_DIR / "kokoro"
            target_dir.mkdir(parents=True, exist_ok=True)
            
            repo = "hexgrad/Kokoro-82M"
            onnx_path = hf_hub_download(repo_id=repo, filename="kokoro-v1.0.onnx", local_dir=target_dir)
            voices_path = hf_hub_download(repo_id=repo, filename="voices-v1.0.bin", local_dir=target_dir)
            
            self._kokoro = Kokoro(str(onnx_path), str(voices_path))

    def synthesize(self, text: str) -> tuple[bytes, str]:
        self._ensure_kokoro()
        import soundfile as sf
        
        samples, sample_rate = self._kokoro.create(text, voice=self.voice, speed=1, lang="en-us")
        
        wav_buffer = BytesIO()
        sf.write(wav_buffer, samples, sample_rate, format='WAV')
        audio_bytes = wav_buffer.getvalue()
        
        timestamp = int(time.time())
        audio_file = TEMP_DIR / f"audio_{timestamp}.wav"
        audio_file.write_bytes(audio_bytes)
        
        return audio_bytes, "audio/wav"


class KittenTTSService(BaseTTSService):
    """Uses the KittenTTS model (80M variant).
    Optimized for CPU and extremely expressive.
    """
    def __init__(self, voice: str = "female_1"):
        self.model = "kittentts-80m"
        self.voice = voice
        self._model = None

    def _ensure_model(self):
        if self._model is None:
            try:
                # KittenTTS 80M currently requires torch
                from kittentts import KittenTTS
                repo_id = config.get("custom_settings", {}).get("local_tts_id", "KittenML/KittenTTS-80M")
                model_path = ModelDownloader().ensure_model(repo_id)
                self._model = KittenTTS.from_pretrained(str(model_path))
            except ImportError:
                raise ImportError("KittenTTS not installed. pip install git+https://github.com/KittenML/KittenTTS")
            except Exception as e:
                if "c10.dll" in str(e) or "1114" in str(e):
                    raise RuntimeError("KittenTTS failed due to a PyTorch DLL error. This is a common Windows issue. "
                                     "Please use 'Piper' or 'Kokoro' instead, as they now use stable ONNX engines.")
                raise e

    def synthesize(self, text: str) -> tuple[bytes, str]:
        self._ensure_model()
        import soundfile as sf
        
        # Mapping voice names to KittenTTS internal IDs if needed
        # For now, we assume the user picks what KittenTTS supports (female_1, male_1, etc.)
        audio = self._model.predict(text, voice=self.voice)
        
        wav_buffer = BytesIO()
        sf.write(wav_buffer, audio, 24000, format='WAV')
        audio_bytes = wav_buffer.getvalue()
        
        timestamp = int(time.time())
        audio_file = TEMP_DIR / f"audio_{timestamp}.wav"
        audio_file.write_bytes(audio_bytes)
        
        return audio_bytes, "audio/wav"


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
            try:
                if "kokoro" in model:
                    _tts_service = KokoroTTSService(voice=voice or "af_heart")
                    # Test for DLL/Import issues immediately
                    from kokoro_onnx import Kokoro
                elif "kittentts" in model:
                    _tts_service = KittenTTSService(voice=voice or "female_1")
                    from kittentts import KittenTTS
                else:
                    _tts_service = LocalTTSService(voice=voice or "en_US-lessac-medium") # Piper
            except Exception as e:
                # Catching all because DLL load errors are often generic Exceptions or ImportErrors
                print(f"[Lotaria] Local model {model} failed to load: {e}")
                print(f"[Lotaria] Falling back to Piper (Standalone Engine)")
                _tts_service = LocalTTSService(voice="en_US-lessac-medium")
                model = "local/piper-fallback"
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
