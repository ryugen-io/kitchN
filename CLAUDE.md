# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Kitchn is a system-wide theming tool that unifies configuration across your Linux desktop ecosystem. The core concept is "Single Source of Truth" - edit one central configuration and propagate changes to all applications via Tera templates.

## Build Commands

```bash
just build              # Build release binaries
just install            # Full installation (config + build)
just test               # Run all workspace tests
just test-lib           # Test k-lib only
just lint               # Clippy + format check
just pre-commit         # Full pre-commit checks
just bench-lib          # Benchmark k-lib
just examples           # Run all FFI examples (C++, Python, Rust)
just stats              # Show project statistics (LOC, sizes)
```

Single crate/test operations:
```bash
cargo build -p k-lib
cargo test -p k-lib
cargo test -p k-lib -- test_name           # Run single test
cargo test -p k-lib -- test_name --nocapture  # With stdout
```

## Architecture

### Crate Dependency Graph
```
kitchn (CLI binary)     k-log (logging binary)     k-ffi (C-ABI library)
        └───────────────────┼───────────────────────────┘
                            │
                         k-lib (core logic)
```

### Crate Purposes
- **k-lib** (`kitchn_lib`): All business logic - config loading, template processing, ingredient parsing, PastryDB (Sled-based storage), logging
- **kitchn** (`kitchn_cli`): CLI wrapper with Clap-based argument parsing and commands in `src/commands/`
- **k-log** (`kitchn_log`): Standalone logging CLI binary
- **k-ffi** (`kitchn_ffi`): C-ABI compatible interface for embedding in C++/Python; generates `include/kitchn.h` via cbindgen

### Rust Editions
- k-lib, kitchn, k-log: **Edition 2024**
- k-ffi: **Edition 2021** (for stable C-ABI)

### Key Types in k-lib
- `Cookbook`: Aggregated config (theme + icons + layout + dictionary)
- `Ingredient`: Parsed .ing file representation
- `PastryDB`: Sled/bincode-based ingredient storage
- `ConfigError`: Typed error enum using thiserror

### Data Flow
1. **Config**: TOML files → `Cookbook::load()` → binary cache (bincode)
2. **Ingredients**: `.ing` file → `Ingredient::parse()` → `PastryDB::store()`
3. **Cook**: `PastryDB::get_all()` → Tera render → target files → hook execution

## Error Handling Pattern

- Library code (k-lib): Use `thiserror` for typed error enums
- Binary code (kitchn, k-log): Use `anyhow::Result` for propagation

## Single Instance Policy

Uses `flock()` on `~/.cache/kitchn/kitchn.lock` to prevent concurrent modifications. Debug viewer is exempt.

## Config Locations

- User config: `~/.config/kitchn/`
- Binary cache: `~/.cache/kitchn/pastry.bin`
- Data/DB: `~/.local/share/kitchn/`
- Logs: `~/.local/state/kitchn/logs/`

## FFI Development

After changing FFI functions, regenerate the C header:
```bash
cbindgen --config cbindgen.toml --crate k-ffi --output include/kitchn.h
```

Test FFI examples:
```bash
just example-cpp
just example-python
just example-rust
```

## CLI Commands Reference

```bash
kitchn stock <path>     # Install ingredient (.ing) or package (.bag)
kitchn pantry           # List all stocked ingredients
kitchn pantry clean     # Remove all ingredients from pantry
kitchn cook             # Apply all ingredients (render templates + run hooks)
kitchn wrap <dir>       # Package .ing files into a .bag archive
kitchn bake             # Pre-compile configs into binary cache
kitchn --debug          # Spawn debug viewer in separate terminal
```

Logging CLI:
```bash
k-log <level> <scope> <msg>   # Ad-hoc log (e.g., k-log error SYSTEM "failed")
k-log <preset>                # Use dictionary preset (e.g., k-log boot_ok)
k-log <preset> --app MyApp    # Override app name for log path
```
