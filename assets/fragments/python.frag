[package]
name = "python-sdk"
version = "0.1.0"
authors = ["Hyprcore Team <team@hyprcore.io>"]
description = "Python bindings and state caching"
license = "MIT"

[[templates]]
# 1. Generate fast Cache
target = "~/.local/share/hyprcore/cache/state.json"
content = """
{
    "colors": { "primary": "{{ colors.primary }}", "error": "{{ colors.error }}" },
    "icons": { "error": "{{ icons.error }}", "success": "{{ icons.success }}" }
}
"""

[[files]]
# 2. Install Wrapper Module
target = "~/.local/lib/python3.11/site-packages/hypr.py"
content = """
import subprocess, json, os
CACHE_FILE = os.path.expanduser("~/.local/share/hyprcore/cache/state.json")

def get_color(name):
    # Load JSON logic here...
    pass

class Log:
    @staticmethod
    def error(scope, msg):
        subprocess.run(["corelog", "error", scope, msg])
"""
