import os
import json
import webview
from dotenv import load_dotenv

load_dotenv()

from services.state import load_config, load_history, cleanup_old_files, config
from bridge import LotariaBridge
from monitor import MonitoringThread


def main():
    # Startup
    load_config()
    load_history()
    cleanup_old_files()

    bridge = LotariaBridge()

    # Resolve the HTML file path relative to this script
    html_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "ui", "index.html")

    # Get screen size for bottom-right positioning
    # We'll position in on_loaded since we need the screen info
    window = webview.create_window(
        title="Lotaria",
        url=html_path,
        js_api=bridge,
        width=400,
        height=350,
        frameless=True,
        transparent=True,
        on_top=True,
        easy_drag=False,
        resizable=False,
    )

    bridge.window = window

    def on_loaded():
        """Called when the webview DOM is ready."""
        # Position bottom-right
        try:
            for screen in webview.screens:
                sw, sh = screen.width, screen.height
                break
            else:
                sw, sh = 1920, 1080
            x = sw - 420
            y = sh - 400
            window.move(x, y)
        except Exception as e:
            print(f"[Lotaria] Could not position window: {e}")

        # Send initial config to JS
        payload = json.dumps(dict(config))
        window.evaluate_js(f"onConfigLoaded({payload})")

        # Auto-start monitoring
        bridge.monitor_thread = MonitoringThread(
            on_roast=bridge._push_roast,
            window_ref=window,
        )
        bridge.monitor_thread.start()
        print("[Lotaria] Auto-started monitoring")

    window.events.loaded += on_loaded

    print("[Lotaria] Starting desktop pet...")
    webview.start(gui="qt", debug=False)

    # Cleanup on exit
    if bridge.monitor_thread and bridge.monitor_thread.is_alive():
        bridge.monitor_thread.stop()
        bridge.monitor_thread.join(timeout=2)
    print("[Lotaria] Goodbye!")


if __name__ == "__main__":
    main()
