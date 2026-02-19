import base64
import time
import threading
from typing import Callable

from services.state import config, add_to_history, build_roast_prompt, cleanup_old_files
from services.capture import ScreenCaptureService
from services.vision import get_vision_service
from services.tts import get_tts_service


class MonitoringThread(threading.Thread):
    """Background thread that periodically captures the screen, roasts, and pushes results."""

    def __init__(self, on_roast: Callable[[dict], None], window_ref=None):
        super().__init__(daemon=True)
        self._stop_event = threading.Event()
        self._wake_event = threading.Event()  # wakes the interval sleep
        self.on_roast = on_roast
        self.window_ref = window_ref  # pywebview window, set after creation
        self._screen = ScreenCaptureService()
        self._loop_count = 0

    def run(self):
        print("[Lotaria] Monitor thread started")
        config["is_active"] = True

        while not self._stop_event.is_set():
            self._loop_count += 1
            if self._loop_count % 10 == 0:
                cleanup_old_files()

            try:
                self._do_roast()
            except Exception as e:
                print(f"[Lotaria] Monitor error: {e}")

            # Wait for interval, waking early if triggered or stopped
            self._wake_event.wait(timeout=config["interval"])
            self._wake_event.clear()

        config["is_active"] = False
        print("[Lotaria] Monitor thread stopped")

    def _do_roast(self, hide_window: bool = True):
        """Capture screen, analyze, TTS, push result."""
        # Hide window before capture to avoid capturing ourselves
        if hide_window and self.window_ref:
            try:
                self.window_ref.hide()
                time.sleep(0.15)
            except Exception:
                pass

        filename, image_bytes = self._screen.capture()

        # Show window back
        if hide_window and self.window_ref:
            try:
                self.window_ref.show()
            except Exception:
                pass

        prompt = build_roast_prompt()
        analysis = get_vision_service().analyze(image_bytes, prompt)
        timestamp = int(time.time())
        add_to_history(analysis, timestamp)

        # TTS
        audio_b64 = None
        if config["audio_enabled"]:
            try:
                audio_bytes, _ = get_tts_service().synthesize(analysis)
                audio_b64 = base64.b64encode(audio_bytes).decode("utf-8")
            except Exception as e:
                print(f"[Lotaria] TTS error: {e}")

        result = {
            "text": analysis,
            "audio_b64": audio_b64,
            "timestamp": timestamp,
        }

        print(f"[Lotaria] Roast delivered: {analysis[:80]}...")
        self.on_roast(result)

    def stop(self):
        self._stop_event.set()
        self._wake_event.set()  # break out of sleep

    def trigger(self):
        """Wake up the thread to roast immediately (without stopping)."""
        self._wake_event.set()
