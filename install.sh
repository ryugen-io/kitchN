#!/usr/bin/env bash
# shellcheck disable=SC2155
# =============================================================================
# Hyprcore Install Script
# Sets up config directory, creates default configs, builds and installs binaries
# =============================================================================

set -euo pipefail
IFS=$'\n\t'

# Fail fast on undefined variables and pipe failures
shopt -s inherit_errexit 2>/dev/null || true

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/hyprcore"
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
command_exists() {
    command -v "$1" &>/dev/null
}

create_dir() {
    local dir="$1"
    if [[ ! -d "$dir" ]]; then
        mkdir -p "$dir" || die "Failed to create directory: $dir"
        success "Created $dir"
    else
        log "Directory exists: $dir"
    fi
}

write_config() {
    local file="$1"
    local content="$2"
    
    if [[ -f "$file" ]]; then
        warn "Config exists, skipping: $(basename "$file")"
        return 0
    fi
    
    log "Creating $(basename "$file")"
    printf '%s\n' "$content" > "$file" || die "Failed to write: $file"
    success "Created $(basename "$file")"
}

# -----------------------------------------------------------------------------
# Config Templates
# -----------------------------------------------------------------------------
THEME_CONFIG='[meta]
name = "Sweet Dracula"

[settings]
active_icons = "nerdfont"

[colors]
bg = "#161925"
fg = "#F8F8F2"
cursor = "#8BE9FD"
selection_bg = "#44475A"
selection_fg = "#F8F8F2"
tabs = "#11131C"
tabs_active = "#BD93F9"
primary = "#FF79C6"
secondary = "#BD93F9"
success = "#50FA7B"
error = "#FF5555"
warn = "#F1FA8C"
info = "#8BE9FD"
black = "#44475A"
red = "#DE312B"
green = "#2FD651"
yellow = "#D0D662"
blue = "#9C6FCF"
magenta = "#DE559C"
cyan = "#6AC5D3"
white = "#D7D4C8"
bright_black = "#656B84"
bright_red = "#FF5555"
bright_green = "#50FA7B"
bright_yellow = "#F1FA8C"
bright_blue = "#BD93F9"
bright_magenta = "#FF79C6"
bright_cyan = "#8BE9FD"
bright_white = "#F8F8F2"

[fonts]
mono = "JohtoMono Nerd Font Mono"
ui = "Roboto"
size_mono = "10"
size_ui = "11"'

ICONS_CONFIG='[nerdfont]
success = ""
error = ""
warn = ""
info = ""
net = "󰖩"

[ascii]
success = "*"
error = "!"
warn = "!!"
info = "i"
net = "#"'

LAYOUT_CONFIG='[tag]
prefix = "["
suffix = "]"
transform = "lowercase"
min_width = 0
alignment = "left"

[labels]
error = "error"
success = "success"
info = "info"
warn = "warn"

[structure]
terminal = "{tag} {scope} {icon} {msg}"
file = "{timestamp} {tag} {msg}"

[logging]
base_dir = "~/.local/state/hyprcore/logs"
path_structure = "{year}/{month}/{scope}"
filename_structure = "{level}.{year}-{month}-{day}.log"
timestamp_format = "%H:%M:%S"
write_by_default = true'

DICTIONARY_CONFIG='# System
[presets.boot_ok]
level = "success"
scope = "SYSTEM"
msg = "startup complete"

[presets.shutdown]
level = "info"
scope = "SYSTEM"
msg = "shutting down"

[presets.service_start]
level = "success"
msg = "service started"

[presets.service_stop]
level = "info"
msg = "service stopped"

[presets.service_fail]
level = "error"
msg = "service failed"

# Development
[presets.build_ok]
level = "success"
scope = "BUILD"
msg = "build complete"

[presets.pack_ok]
level = "success"
scope = "PACK"
msg = "package created"

[presets.install_ok]
level = "success"
scope = "INSTALL"
msg = "installed successfully"

[presets.build_fail]
level = "error"
scope = "BUILD"
msg = "build failed"

[presets.test_pass]
level = "success"
scope = "TEST"
msg = "all tests passed"

[presets.test_fail]
level = "error"
scope = "TEST"
msg = "tests failed"

[presets.deploy_ok]
level = "success"
scope = "DEPLOY"
msg = "deployed successfully"

# Docker
[presets.container_start]
level = "success"
scope = "DOCKER"
msg = "container started"

[presets.container_stop]
level = "info"
scope = "DOCKER"
msg = "container stopped"

# Network
[presets.net_up]
level = "success"
scope = "NET"
msg = "network connected"

[presets.net_down]
level = "warn"
scope = "NET"
msg = "network disconnected"

[presets.ssh_ok]
level = "success"
scope = "SSH"
msg = "connection established"

[presets.sync_ok]
level = "success"
scope = "SYNC"
msg = "sync complete"

[presets.sync_start]
level = "info"
scope = "SYNC"
msg = "syncing fragments"

[presets.sync_empty]
level = "info"
scope = "SYNC"
msg = "no fragments to sync"

[presets.list_empty]
level = "info"
scope = "LIST"
msg = "no fragments installed"

# Backup
[presets.backup_start]
level = "info"
scope = "BACKUP"
msg = "backup started"

[presets.backup_ok]
level = "success"
scope = "BACKUP"
msg = "backup complete"

[presets.backup_fail]
level = "error"
scope = "BACKUP"
msg = "backup failed"

# Memory Check
[presets.memcheck_start]
level = "info"
scope = "MEMCHECK"
msg = "running memory check"

[presets.memcheck_ok]
level = "success"
scope = "MEMCHECK"
msg = "no memory leaks detected"

[presets.memcheck_fail]
level = "error"
scope = "MEMCHECK"
msg = "memory issues detected"

[presets.memcheck_compile]
level = "info"
scope = "MEMCHECK"
msg = "compiling with sanitizers"

# Database
[presets.db_connect]
level = "success"
scope = "DB"
msg = "connected"

[presets.db_fail]
level = "error"
scope = "DB"
msg = "connection refused"'

# -----------------------------------------------------------------------------
# Main Installation
# -----------------------------------------------------------------------------
main() {
    log "Starting Hyprcore installation"
    
    # Verify we're in the right directory
    if [[ ! -f "${SCRIPT_DIR}/Cargo.toml" ]]; then
        die "Must run from hyprcore repository root"
    fi
    
    cd "$SCRIPT_DIR" || die "Failed to cd to script directory"
    
    # Create directories
    create_dir "$CONFIG_DIR"
    create_dir "$INSTALL_DIR"
    
    # Write config files
    write_config "${CONFIG_DIR}/theme.toml" "$THEME_CONFIG"
    write_config "${CONFIG_DIR}/icons.toml" "$ICONS_CONFIG"
    write_config "${CONFIG_DIR}/layout.toml" "$LAYOUT_CONFIG"
    write_config "${CONFIG_DIR}/dictionary.toml" "$DICTIONARY_CONFIG"
    
    # Build
    if ! command_exists cargo; then
        die "Cargo not found. Install Rust: https://rustup.rs"
    fi
    
    log "Building release binaries"
    if ! cargo build --release 2>&1; then
        die "Build failed"
    fi
    success "Build complete"
    
    # Install binaries
    local binaries=("corelog" "hyprcore")
    for bin in "${binaries[@]}"; do
        local src="target/release/${bin}"
        local dst="${INSTALL_DIR}/${bin}"
        
        if [[ ! -f "$src" ]]; then
            die "Binary not found: $src"
        fi
        
        cp "$src" "$dst" || die "Failed to install: $bin"
    done
    success "Installed binaries to $INSTALL_DIR"
    
    # PATH check
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "$INSTALL_DIR not in PATH"
        echo "  Add to shell config: export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
    
    echo ""
    echo -e "${PURPLE}[hyprcore]${NC} ${CHECK}  Installation complete"
    echo ""
    echo "Try:"
    echo "  corelog test_pass"
    echo "  corelog install_ok"
    echo "  hyprcore install ./assets/fragments/waybar.frag"
}

main "$@"
