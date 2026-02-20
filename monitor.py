import base64
import time
import wave
from io import BytesIO
import threading
from typing import Callable

from services.state import config, add_to_history, build_roast_prompt, cleanup_old_files, check_rate_limit, record_api_request, truncate_response, get_interval_seconds
from services.capture import ScreenCaptureService
from services.vision import get_vision_service
from services.tts import get_tts_service


class MonitoringThread(threading.Thread):
    """Background thread that periodically captures the screen, roasts, and pushes results."""

    def __init__(self, on_roast: Callable[[dict], None], window_ref=None, processing_lock=None):
        super().__init__(daemon=True)
        self._stop_event = threading.Event()
        self._wake_event = threading.Event()  # wakes the interval sleep
        self.on_roast = on_roast
        self.window_ref = window_ref  # pywebview window, set after creation
        self._processing_lock = processing_lock or threading.Lock()
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

            # Wait for interval (randomized from preset), waking early if triggered or stopped
            self._wake_event.wait(timeout=get_interval_seconds())
            self._wake_event.clear()

        config["is_active"] = False
        print("[Lotaria] Monitor thread stopped")

    def _do_roast(self, hide_window: bool = True):
        """Capture screen, analyze, TTS, push result."""
        if not self._processing_lock.acquire(blocking=False):
            return # Already processing (e.g. manual roast in progress)
            
        try:
            # Check rate limits before proceeding
            can_proceed, wait_time = check_rate_limit()
            if not can_proceed:
                print(f"[Lotaria] Rate limit hit, waiting {wait_time:.1f}s")
                time.sleep(wait_time)
            
            # Record that we're making an API request
            record_api_request()
            
            # Notify JS we're thinking (simulates the transition)
            if self.window_ref:
                try:
                    self.window_ref.evaluate_js("onThinkingStart()")
                except Exception:
                    pass

            # Move window off-screen to avoid self-capture (smoother than hide/show)
            original_pos = None
            if hide_window and self.window_ref:
                try:
                    original_pos = self.window_ref.x, self.window_ref.y
                    self.window_ref.move(-1000, -1000)
                    time.sleep(0.05)
                except Exception:
                    pass

            filename, image_bytes = self._screen.capture()

            # Restore window position
            if hide_window and self.window_ref and original_pos:
                try:
                    self.window_ref.move(original_pos[0], original_pos[1])
                except Exception:
                    pass

            prompt = build_roast_prompt()
            analysis = get_vision_service().analyze(image_bytes, prompt)
            analysis = truncate_response(analysis)
            timestamp = int(time.time())
            add_to_history(analysis, timestamp)

            # TTS
            audio_b64 = None
            audio_duration = 0
            if config["audio_enabled"]:
                try:
                    audio_bytes, mime = get_tts_service().synthesize(analysis)
                    audio_b64 = base64.b64encode(audio_bytes).decode("utf-8")
                    # Estimate duration from WAV data
                    try:
                        with wave.open(BytesIO(audio_bytes), "rb") as wf:
                            frames = wf.getnframes()
                            rate = wf.getframerate()
                            audio_duration = frames / rate if rate else 0
                    except Exception:
                        # Rough estimate: ~150 words/min for TTS
                        word_count = len(analysis.split())
                        audio_duration = (word_count / 150) * 60
                except Exception as e:
                    print(f"[Lotaria] TTS error: {e}")

            result = {
                "text": analysis,
                "audio_b64": audio_b64,
                "audio_duration": audio_duration,
                "timestamp": timestamp,
            }

            print(f"[Lotaria] Roast delivered ({audio_duration:.1f}s audio): {analysis[:80]}...")
            self.on_roast(result)
        finally:
            try:
                self._processing_lock.release()
            except RuntimeError:
                pass

    def stop(self):
        self._stop_event.set()
        self._wake_event.set()  # break out of sleep

    def trigger(self):
        """Wake up the thread to roast immediately (without stopping)."""
        self._wake_event.set()
