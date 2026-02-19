import time
from io import BytesIO
from pathlib import Path

import mss
from PIL import Image

from .state import TEMP_DIR


class ScreenCaptureService:
    @staticmethod
    def capture() -> tuple[str, bytes]:
        """Capture the full primary screen. Returns (filename, png_bytes)."""
        with mss.mss() as sct:
            monitor = sct.monitors[1]  # Primary monitor
            screenshot = sct.grab(monitor)

            img = Image.frombytes("RGB", screenshot.size, screenshot.bgra, "raw", "BGRX")

            timestamp = int(time.time())
            filename = f"capture_{timestamp}.png"
            filepath = TEMP_DIR / filename

            img.save(filepath, "PNG")

            buffer = BytesIO()
            img.save(buffer, format="PNG")
            image_bytes = buffer.getvalue()

            return filename, image_bytes
