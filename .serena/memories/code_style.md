# Kitchn - Code Style & Conventions

## Rust Edition
- **k-lib, kitchn, k-log**: Edition 2024
- **k-ffi**: Edition 2021 (for stable C-ABI)

## Naming Conventions
- **Structs/Enums**: `PascalCase` (e.g., `ThemeConfig`, `Commands`, `ConfigError`)
- **Functions/Methods**: `snake_case` (e.g., `load_from_dir`, `log_to_terminal`)
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case` (e.g., `config`, `logger`, `ingredient`)
- **Crate Names**: `k-lib`, `k-ffi`, `k-log` (kebab-case in Cargo.toml)

## Error Handling
- Use `thiserror` for defining error enums:
  ```rust
  #[derive(thiserror::Error, Debug)]
  pub enum ConfigError {
      #[error("Failed to load config: {0}")]
      LoadError(String),
  }
  ```
- Use `anyhow::Result` for CLI/binary error propagation
- Typed errors in library code, anyhow in binaries

## Serialization
- `serde` with `derive` feature for all config structs
- `toml` for human-readable configs
- `bincode` for binary caching (PastryDB)

## CLI Patterns
- Use `clap` with derive macros:
  ```rust
  #[derive(Parser)]
  struct Cli {
      #[command(subcommand)]
      command: Commands,
  }
  
  #[derive(Subcommand)]
  enum Commands {
      /// Doc comment becomes help text
      Stock { path: PathBuf },
  }
  ```

## Logging
- `tracing` for structured logging in CLI
- `colored` for terminal output formatting
- Custom logger in k-lib for themed output

## Testing
- Inline module tests with `#[cfg(test)]`
- Integration tests in `tests/` directory
- Use `tempfile` for filesystem tests
- Use `assert_cmd` + `predicates` for CLI tests

## FFI Patterns
- `#[no_mangle]` and `extern "C"` for C-ABI
- Opaque pointer patterns (`*mut Context`)
- Manual memory management with `_new`/`_free` pairs
- `cbindgen` for header generation

## Documentation
- Doc comments (`///`) on public items
- Module-level docs (`//!`) for crate overview
- Examples in doc comments where helpful
