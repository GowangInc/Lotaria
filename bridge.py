import json
import time
import base64
import threading

from services.state import (
    config, save_config, build_roast_prompt, add_to_history,
    PROVIDERS, get_masked_api_keys, set_api_key,
)
from services.capture import ScreenCaptureService
from services.vision import get_vision_service
from services.tts import get_tts_service


class LotariaBridge:
    """JS API exposed to pywebview via window.pywebview.api.*"""

    def __init__(self):
        self.window = None           # set by app.py after window creation
        self.monitor_thread = None   # set by app.py

    def roast_now(self):
        """Capture screen, analyze, TTS — returns result dict to JS."""
        # Notify JS we're thinking
        if self.window:
            self.window.evaluate_js("onThinkingStart()")

        # Hide window to avoid self-capture
        if self.window:
            try:
                self.window.hide()
                time.sleep(0.15)
            except Exception:
                pass

        try:
            screen = ScreenCaptureService()
            filename, image_bytes = screen.capture()
        finally:
            if self.window:
                try:
                    self.window.show()
                except Exception:
                    pass

        prompt = build_roast_prompt()
        analysis = get_vision_service().analyze(image_bytes, prompt)
        timestamp = int(time.time())
        add_to_history(analysis, timestamp)

        audio_b64 = None
        if config["audio_enabled"]:
            try:
                audio_bytes, _ = get_tts_service().synthesize(analysis)
                audio_b64 = base64.b64encode(audio_bytes).decode("utf-8")
            except Exception as e:
                print(f"[Lotaria] TTS error in roast_now: {e}")

        result = {
            "text": analysis,
            "audio_b64": audio_b64,
            "timestamp": timestamp,
        }

        return result

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
            )
            self.monitor_thread.start()
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
        """Return the PROVIDERS dict so JS can build the settings UI."""
        return PROVIDERS

    def get_api_keys(self):
        """Return which providers have keys set (masked)."""
        return get_masked_api_keys()

    def save_api_key(self, provider, key):
        """Save an API key for a provider."""
        set_api_key(provider, key)
        return True

    def set_vision_config(self, provider, model):
        """Update vision provider and model."""
        config["vision_provider"] = provider
        config["vision_model"] = model
        save_config()
        print(f"[Lotaria] Vision config updated: {provider}/{model}")
        return True

    def set_tts_config(self, provider, model, voice):
        """Update TTS provider, model, and voice."""
        config["tts_provider"] = provider
        config["tts_model"] = model
        config["tts_voice"] = voice
        save_config()
        print(f"[Lotaria] TTS config updated: {provider}/{model} voice={voice}")
        return True

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
