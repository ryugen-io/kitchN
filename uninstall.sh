#!/usr/bin/env bash
# =============================================================================
# Hyprcore Uninstall Script
# Removes binaries, config, data and log directories
# =============================================================================

set -euo pipefail
IFS=$'\n\t'

# Fail fast on undefined variables and pipe failures
shopt -s inherit_errexit 2>/dev/null || true

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------
readonly CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/hyprcore"
readonly DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/hyprcore"
readonly STATE_DIR="${XDG_STATE_HOME:-$HOME/.local/state}/hyprcore"
readonly INSTALL_DIR="${HOME}/.local/bin"

# Colors (Sweet Dracula palette - 24-bit true color)
readonly GREEN=$'\033[38;2;80;250;123m'
readonly YELLOW=$'\033[38;2;241;250;140m'
readonly CYAN=$'\033[38;2;139;233;253m'
readonly RED=$'\033[38;2;255;85;85m'
readonly PURPLE=$'\033[38;2;189;147;249m'
readonly NC=$'\033[0m'

# Icons (Nerd Font)
readonly CHECK=''
readonly WARN=''
readonly ERR=''
readonly INFO_ICON=''

# -----------------------------------------------------------------------------
# Logging Functions
# -----------------------------------------------------------------------------
log()     { echo -e "${CYAN}[info]${NC} ${INFO_ICON}  $*"; }
success() { echo -e "${GREEN}[ok]${NC}   ${CHECK}  $*"; }
warn()    { echo -e "${YELLOW}[warn]${NC} ${WARN}  $*" >&2; }
error()   { echo -e "${RED}[err]${NC}  ${ERR}  $*" >&2; }
die()     { error "$*"; exit 1; }

# -----------------------------------------------------------------------------
# Cleanup & Signal Handling
# -----------------------------------------------------------------------------
cleanup() {
    local exit_code=$?
    # Add cleanup tasks here if needed
    exit "$exit_code"
}
trap cleanup EXIT
trap 'die "Interrupted"' INT TERM

# -----------------------------------------------------------------------------
# Utility Functions
# -----------------------------------------------------------------------------
remove_if_exists() {
    local path="$1"
    local desc="$2"
    
    if [[ -e "$path" ]]; then
        rm -rf "$path"
        success "Removed $desc"
    else
        warn "$desc not found, skipping"
    fi
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    log "Starting Hyprcore uninstall"
    
    # Remove binaries
    remove_if_exists "${INSTALL_DIR}/corelog" "corelog binary"
    remove_if_exists "${INSTALL_DIR}/hyprcore" "hyprcore binary"
    
    # Remove directories
    remove_if_exists "$CONFIG_DIR" "config directory"
    remove_if_exists "$DATA_DIR" "data directory"
    remove_if_exists "$STATE_DIR" "state directory"
    
    echo ""
    echo -e "${PURPLE}[hyprcore]${NC} ${CHECK}  Uninstall complete"
}

main "$@"
