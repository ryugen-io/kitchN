# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2025-12-09

### Fixed
- Fixed build failures on Musl/Alpine systems (Raspberry Pi compatibility) by making FFI library build optional/dynamic.
- Fixed installation URLs in `README.md` to point to the correct `ryugen-io/kitchN` repository.
- Updated `install.sh` to robustly handle `cdylib` crate types on static-by-default targets.

## [0.2.0] - 2025-12-08

### Added
- GitHub Releases with pre-built binaries for Linux x64 and ARM64
- Automated CI/CD pipeline with GitHub Actions
- Remote installation support via `curl | bash`
- FFI library (`libkitchn_ffi.so`) included in release packages
- C header (`kitchn.h`) for FFI bindings

### Changed
- Install script now supports three modes: source, package, and remote
- Release packages include default configuration files

### Fixed
- TBD

## [0.1.0] - Initial Release

### Added
- Core `kitchn` CLI for theme and ingredient management
- `kitchn-log` utility for structured logging
- Sweet Dracula color theme
- TOML-based configuration system
- FFI bindings for C/C++ and Python
- Ingredient templating with Tera

## [0.3.0] - 2025-12-14

### Changed
- **Breaking**: Renamed crates to short-form:
  - `kitchn_lib` -> `k-lib`
  - `kitchn_ffi` -> `k-ffi`
  - `kitchn_cli` -> `k-bin`
  - `kitchn_log` -> `k-log`
- **FFI**: Major update to C-ABI.
  - Replaced context-based API with `KitchnPantry`.
  - Added `kitchn_pantry_*` functions for direct database manipulation.

### Removed
- Deprecated `id` field from `IngredientManifest`.
