#!/bin/bash

# Build latest corelog if needed (ensure we use the release binary)
LOG=./target/release/corelog

echo "----------------------------------------"
echo "  Hyprcore Visual Palette"
echo "----------------------------------------"
echo ""

# Helper to print color block
# Usage: show_color <key>
show_color() {
    key=$1
    # We use the 'info' preset but override the message
    #  is a Nerd Font block, or we can use ANSI block like █
    # Let's use a big block for visibility: ████
    $LOG info "  <$key>██████</$key>  $key"
}

echo "--- Semantic ---"
show_color "primary"
show_color "secondary"
show_color "success"
show_color "error"
show_color "warn"
show_color "info"
show_color "orange"

echo ""
echo "--- UI / Special ---"
show_color "bg"
show_color "fg"
show_color "cursor"
show_color "selection_bg"
show_color "selection_fg"
show_color "tabs"
show_color "tabs_active"

echo ""
echo "--- ANSI Normal ---"
show_color "black"
show_color "red"
show_color "green"
show_color "yellow"
show_color "blue"
show_color "magenta"
show_color "cyan"
show_color "white"

echo ""
echo "--- ANSI Bright ---"
show_color "bright_black"
show_color "bright_red"
show_color "bright_green"
show_color "bright_yellow"
show_color "bright_blue"
show_color "bright_magenta"
show_color "bright_cyan"
show_color "bright_white"

echo ""
echo "----------------------------------------"
