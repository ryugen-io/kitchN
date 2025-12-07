[package]
name = "fish-theme"
version = "0.1.0"
authors = ["Hyprcore Team <team@hyprcore.io>"]
description = "Fish shell integration for Hyprcore constants"
license = "MIT"

[[templates]]
target = "~/.config/fish/conf.d/hypr_theme.fish"
content = """
set -gx HYPR_PRIMARY "{{ colors.primary }}"
set -gx HYPR_FONT_MONO "{{ fonts.mono }}"
set -gx HYPR_ICON_ERR "{{ icons.error }}"

function hlog
    corelog $argv
end
"""
