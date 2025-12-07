[package]
name = "brave-theme"
version = "0.1.0"
authors = ["Hyprcore Team <team@hyprcore.io>"]
description = "Browser theme generator for Brave/Chrome"
license = "MIT"

[[templates]]
target = "~/.config/hyprcore/generated/brave_theme/manifest.json"
content = """
{
  "manifest_version": 3,
  "version": "1.0",
  "name": "Hyprcore Generated Theme",
  "theme": {
    "colors": {
      "frame": {{ colors.tabs | hex_to_rgb }},
      "toolbar": {{ colors.bg | hex_to_rgb }},
      "tab_text": {{ colors.fg | hex_to_rgb }},
      "tab_background_text": {{ colors.bright_black | hex_to_rgb }},
      "bookmark_text": {{ colors.fg | hex_to_rgb }},
      "ntp_background": {{ colors.bg | hex_to_rgb }},
      "ntp_text": {{ colors.fg | hex_to_rgb }},
      "button_background": {{ colors.primary | hex_to_rgb }}
    }
  }
}
"""

[hooks]
# Optional: could open the folder, but that might be annoying.
# reload = "xdg-open ~/.config/hyprcore/generated/brave_theme/"
