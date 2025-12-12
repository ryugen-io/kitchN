# Kitchn - Dependencies Overview

## k-lib (Core)

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1.0 | Serialization framework |
| `toml` | 0.8 | TOML config parsing |
| `bincode` | 2.0 | Binary serialization for cache |
| `tera` | 1.20 | Template engine |
| `colored` | 2.2 | Terminal colors |
| `thiserror` | 2.0 | Error derive macros |
| `anyhow` | 1.0 | Error propagation |
| `chrono` | 0.4 | Date/time handling |
| `directories` | 5.0 | XDG directory paths |
| `zip` | 2.2 | .bag packaging |
| `lazy_static` | 1.5 | Static initialization |
| `libc` | 0.2 | System calls |
| `log` | 0.4 | Logging facade |

## kitchn_cli

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.5 | CLI argument parsing (derive) |
| `tracing` | 0.1 | Structured logging |
| `tracing-subscriber` | 0.3 | Log formatting |
| `tracing-appender` | 0.2 | Log file output |
| `which` | 6.0 | Find terminal executables |

## k-ffi

| Crate | Version | Purpose |
|-------|---------|---------|
| `k-lib` | internal | Core functionality |

## Dev Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tempfile` | 3.10+ | Temporary files for tests |
| `criterion` | 0.5 | Benchmarking |
| `assert_cmd` | 2.0 | CLI testing |
| `predicates` | 3.1 | Test assertions |
| `serde_json` | 1.0 | JSON in tests |

## Build Dependencies

| Tool | Purpose |
|------|---------|
| `cbindgen` | Generate C headers from Rust |
| `upx` | Binary compression (optional) |

## System Dependencies

| Tool | Purpose |
|------|---------|
| `just` | Command runner |
| Terminal | Debug viewer (rio/alacritty/kitty preferred) |
