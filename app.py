import os
import sys
import json
import signal
import atexit
import webview
from dotenv import load_dotenv

load_dotenv()

from services.state import load_config, load_history, cleanup_old_files, config, save_config
from bridge import LotariaBridge
from monitor import MonitoringThread


def main():
    # Startup
    load_config()
    
    # Check for --reset flag to force first-run mode
    if '--reset' in sys.argv:
        print("[Lotaria] Reset flag detected - forcing first-run mode")
        config['first_run'] = True
        save_config()
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
        width=420,
        height=400,
        frameless=True,
        transparent=True,
        on_top=True,
        easy_drag=False,
        resizable=True,
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

        # Auto-start monitoring (skip on first run - let user complete onboarding)
        if not config.get('first_run', True):
            bridge.monitor_thread = MonitoringThread(
                on_roast=bridge._push_roast,
                window_ref=window,
                processing_lock=bridge._processing_lock,
            )
            bridge.monitor_thread.start()
            config["is_active"] = True
            print("[Lotaria] Auto-started monitoring")

    window.events.loaded += on_loaded

    def cleanup():
        if bridge.monitor_thread and bridge.monitor_thread.is_alive():
            bridge.monitor_thread.stop()
            bridge.monitor_thread.join(timeout=2)

    def sigint_handler(sig, frame):
        print("\n[Lotaria] Ctrl+C received, shutting down...")
        cleanup()
        try:
            window.destroy()
        except Exception:
            pass
        sys.exit(0)

    signal.signal(signal.SIGINT, sigint_handler)
    atexit.register(cleanup)

    print("[Lotaria] Starting desktop pet...")
    try:
        webview.start(gui="qt", debug=False)
    except KeyboardInterrupt:
        pass

    # Cleanup on exit
    cleanup()
    print("[Lotaria] Goodbye!")


if __name__ == "__main__":
    main()
