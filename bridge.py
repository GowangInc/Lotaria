import json
import time
import base64
import threading

from services.state import (
    config, save_config, build_roast_prompt, add_to_history,
    PROVIDERS, get_masked_api_keys, set_api_key,
    detect_gpu_capabilities, get_default_mode_recommendation,
    mark_first_run_complete, check_rate_limit, record_api_request,
    clear_history, MOOD_PROMPTS, truncate_response, MODEL_COSTS,
    GEMINI_LIVE_VOICES, fetch_ollama_models,
)
from services.capture import ScreenCaptureService
from services.vision import get_vision_service
from services.tts import get_tts_service


class LotariaBridge:
    """JS API exposed to pywebview via window.pywebview.api.*"""

    def __init__(self):
        self.window = None           # set by app.py after window creation
        self.monitor_thread = None   # set by app.py
        self._processing_lock = threading.Lock()
        self.is_processing = False

    def roast_now(self):
        """Capture screen, analyze, TTS — returns result dict to JS."""
        if not self._processing_lock.acquire(blocking=False):
            return None # Already processing
            
        try:
            self.is_processing = True
            # Check rate limits before proceeding
            can_proceed, wait_time = check_rate_limit()
            if not can_proceed:
                return {
                    "text": f"Whoa, slow down! Please wait {wait_time:.1f}s before roasting again.",
                    "audio_b64": None,
                    "audio_duration": 0,
                    "timestamp": int(time.time()),
                    "error": "rate_limit"
                }
            
            # Record that we're making an API request
            record_api_request()
        
            # Notify JS we're thinking
            if self.window:
                self.window.evaluate_js("onThinkingStart()")

            # Move window off-screen to avoid self-capture (smoother than hide/show)
            original_pos = None
            if self.window:
                try:
                    original_pos = self.window.x, self.window.y
                    # Move to top-left corner (0,0) which is usually safe
                    self.window.move(-1000, -1000)
                    time.sleep(0.05)  # Short delay for move
                except Exception:
                    pass

            try:
                screen = ScreenCaptureService()
                filename, image_bytes = screen.capture()
            finally:
                if self.window and original_pos:
                    try:
                        self.window.move(original_pos[0], original_pos[1])
                    except Exception:
                        pass

            prompt = build_roast_prompt()
            try:
                analysis = get_vision_service().analyze(image_bytes, prompt)
            except RuntimeError as e:
                # Local model failed to load (e.g., PyTorch DLL error)
                error_msg = str(e)
                if "PyTorch failed to load" in error_msg:
                    return {
                        "text": "Local model failed to load. Please switch to API mode in Settings, or fix your PyTorch installation.",
                        "audio_b64": None,
                        "audio_duration": 0,
                        "timestamp": int(time.time()),
                        "error": error_msg
                    }
                raise
            except Exception as e:
                error_str = str(e).lower()
                if "429" in error_str or "rate limit" in error_str or "resource exhausted" in error_str:
                    model = config.get("vision_model", "unknown")
                    return {
                        "text": f"Rate limit hit on {model.split('/')[-1]}. Give me a minute to cool down...",
                        "audio_b64": None,
                        "audio_duration": 0,
                        "timestamp": int(time.time()),
                        "error": "rate_limit"
                    }
                raise
            analysis = truncate_response(analysis)
            timestamp = int(time.time())
            add_to_history(analysis, timestamp)

            audio_b64 = None
            audio_duration = 0
            if config["audio_enabled"]:
                try:
                    audio_bytes, _ = get_tts_service().synthesize(analysis)
                    audio_b64 = base64.b64encode(audio_bytes).decode("utf-8")
                    # Get actual duration from WAV
                    import wave
                    from io import BytesIO
                    try:
                        with wave.open(BytesIO(audio_bytes), "rb") as wf:
                            frames = wf.getnframes()
                            rate = wf.getframerate()
                            audio_duration = frames / rate if rate else 0
                    except Exception:
                        word_count = len(analysis.split())
                        audio_duration = (word_count / 150) * 60
                except Exception as e:
                    tts_model = config.get("tts_model", "unknown")
                    print(f"[Lotaria] TTS error ({tts_model}): {e}")

            result = {
                "text": analysis,
                "audio_b64": audio_b64,
                "audio_duration": audio_duration,
                "timestamp": timestamp,
            }

            return result
        finally:
            self.is_processing = False
            try:
                self._processing_lock.release()
            except RuntimeError:
                pass # Lock was already released or not acquired

    def toggle_monitoring(self):
        """Start or stop the background monitoring thread. Returns new is_active state."""
        from monitor import MonitoringThread

        if self.monitor_thread and self.monitor_thread.is_alive():
            self.monitor_thread.stop()
            self.monitor_thread.join(timeout=2)
            self.monitor_thread = None
            config["is_active"] = False
            print("[Lotaria] Monitoring stopped via bridge")
            return False
        else:
            self.monitor_thread = MonitoringThread(
                on_roast=self._push_roast,
                window_ref=self.window,
                processing_lock=self._processing_lock,
            )
            self.monitor_thread.start()
            config["is_active"] = True
            print("[Lotaria] Monitoring started via bridge")
            return True

    def get_config(self):
        """Return full config dict (with api_keys masked)."""
        c = dict(config)
        c["api_keys"] = get_masked_api_keys()
        return c

    def set_config(self, key, value):
        """Update one config key, persist."""
        if key in config:
            config[key] = value
            save_config()
            print(f"[Lotaria] Config updated: {key}={value}")
            return True
        return False

    def get_providers(self):
        """Return the PROVIDERS dict with dynamically fetched local models."""
        p = {k: dict(v) for k, v in PROVIDERS.items()}
        
        # Inject Ollama models
        ollama_models = fetch_ollama_models()
        if ollama_models:
            # Add dynamic models, keep 'custom' at the end
            current_vision = [m for m in p["ollama"]["vision_models"] if m != "ollama/custom"]
            p["ollama"]["vision_models"] = ollama_models + current_vision + ["ollama/custom"]
            # Register costs for these dynamic models as well
            for m in ollama_models:
                MODEL_COSTS[m] = "$"

        return p

    def get_api_keys(self):
        """Return which providers have keys set (masked)."""
        return get_masked_api_keys()

    def get_gpu_info(self):
        """Return GPU capabilities for local model recommendations."""
        return detect_gpu_capabilities()

    def get_mode_recommendation(self):
        """Return recommended mode (local vs API) based on system."""
        return get_default_mode_recommendation()

    def save_api_key(self, provider, key):
        """Save an API key for a provider."""
        set_api_key(provider, key)
        return True

    def set_vision_config(self, provider, model):
        """Update vision provider and model."""
        config["vision_provider"] = provider
        config["vision_model"] = model
        save_config()
        print(f"[Lotaria] Vision config updated: {model}")
        return True

    def set_tts_config(self, provider, model, voice):
        """Update TTS provider, model, and voice."""
        config["tts_provider"] = provider
        config["tts_model"] = model
        config["tts_voice"] = voice
        save_config()
        print(f"[Lotaria] TTS config updated: {model} voice={voice}")
        return True

    def mark_first_run_complete(self):
        """Mark first run as complete after user completes onboarding."""
        mark_first_run_complete()
        return True

    def clear_history(self):
        """Clear all roast history."""
        clear_history()
        return True

    def get_moods(self):
        """Return available mood names and labels for the UI."""
        return {key: key.capitalize() for key in MOOD_PROMPTS}

    def get_model_costs(self):
        """Return cost tier mapping for all models."""
        return MODEL_COSTS

    def get_live_voices(self):
        """Return voices supported by Gemini Live API."""
        return GEMINI_LIVE_VOICES

    def expand_window(self):
        """Expand and center the window for settings modal."""
        if not self.window:
            return None
        try:
            self._saved_pos = (self.window.x, self.window.y)
            self._saved_size = (self.window.width, self.window.height)
            import webview
            for screen in webview.screens:
                sw, sh = screen.width, screen.height
                break
            else:
                sw, sh = 1920, 1080
            new_w, new_h = 500, 700
            x = (sw - new_w) // 2
            y = (sh - new_h) // 2
            self.window.resize(new_w, new_h)
            self.window.move(x, y)
            return True
        except Exception as e:
            print(f"[Lotaria] Could not expand window: {e}")
            return False

    def restore_window(self):
        """Restore window to original size and position."""
        if not self.window:
            return
        try:
            pos = getattr(self, '_saved_pos', None)
            size = getattr(self, '_saved_size', None)
            if size:
                self.window.resize(size[0], size[1])
            if pos:
                self.window.move(pos[0], pos[1])
        except Exception as e:
            print(f"[Lotaria] Could not restore window: {e}")

    def quit(self):
        """Stop monitoring and close the window."""
        if self.monitor_thread and self.monitor_thread.is_alive():
            self.monitor_thread.stop()
            self.monitor_thread.join(timeout=2)
        if self.window:
            self.window.destroy()

    def _push_roast(self, result: dict):
        """Push a roast from the monitor thread to JS."""
        if self.window:
            payload = json.dumps(result)
            self.window.evaluate_js(f"deliverRoast({payload})")
