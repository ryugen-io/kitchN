# Kitchn - Project Overview

## Mission
**"Single Source of Truth"** - Kitchn unifies theming and configuration across your entire ecosystem. One config change propagates to Shells, Scripts, Logs, GUIs, and TUI apps instantly.

## Core Concepts

### Ingredients (.ing)
TOML files that teach Kitchn how to theme a specific application. Contains:
- `[package]` - Metadata (name, version, authors)
- `[[templates]]` - Tera templates with target paths
- `[hooks]` - Reload commands (e.g., `pkill -USR1 waybar`)

### Bags (.bag)
ZIP archives containing multiple .ing files for distributing complete theme collections.

### Cookbook
Aggregated configuration from:
- `theme.toml` - Colors, fonts
- `icons.toml` - Icon sets (nerdfont, ascii)
- `layout.toml` - Log formatting, structure
- `dictionary.toml` - Log presets

### PastryDB
Binary database (Sled/bincode) in `~/.local/share/kitchn/` for instant ingredient access.

### Sweet Dracula
The standard Dracula-based color palette enforced across the system.

## Workspace Structure

```
kitchn/
├── crates/
│   ├── kitchn_lib/   (k-lib)   - Core Logic, Rust 2024
│   ├── kitchn_ffi/   (k-ffi)   - C-ABI Interface, Rust 2021
│   ├── kitchn_cli/   (kitchn)  - CLI Binary, Rust 2024
│   └── kitchn_log/   (k-log)   - Logging CLI, Rust 2024
├── include/          - Generated C headers (kitchn.h)
├── assets/
│   ├── ingredients/  - Example .ing files
│   └── examples/     - C++, Python, Rust integration
├── Cargo.toml        - Workspace config
└── justfile          - Command runner
```

## Architecture Flow

```
CLI (kitchn) ──► k-lib ──► PastryDB
                  │
                  ▼
FFI (k-ffi) ◄── Cookbook ──► Tera Templates
     │
     ▼
  kitchn.h ──► C/C++/Python
```

## Config Locations
- User config: `~/.config/kitchn/`
- Binary cache: `~/.cache/kitchn/pastry.bin`
- Data/DB: `~/.local/share/kitchn/`
- Logs: `~/.local/state/kitchn/logs/`
