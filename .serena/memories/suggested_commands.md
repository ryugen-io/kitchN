# Kitchn - Development Commands

## Just Commands (Primary)

```bash
# Build
just build              # Build release binaries
just install            # Full installation (config + build)

# Testing
just test               # Run all workspace tests
just test-lib           # Test k-lib only

# Code Quality
just lint               # Clippy + format check
just pre-commit         # Full pre-commit checks

# Benchmarks
just bench              # All benchmarks (Criterion + Hyperfine)
just bench-lib          # k-lib benchmarks only

# Run Examples
just examples           # All FFI examples
just example-cpp        # C++ example
just example-python     # Python example
just example-rust       # Rust example

# Development
just cook               # Apply all ingredients
just stock-waybar       # Stock example ingredient
just debug              # Run with debug viewer

# Maintenance
just clean              # Clean build artifacts
just audit              # Audit dependencies
just uninstall          # Remove installation

# Release
just compact            # Optimize binaries with UPX
just package <target> <name>  # Create release tarball
```

## Cargo Commands

```bash
# Build
cargo build --release
cargo build -p k-lib    # Single crate

# Test
cargo test --workspace
cargo test -p k-lib

# Lint
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt -- --check
cargo fmt               # Auto-format

# Bench
cargo bench -p k-lib
```

## CLI Usage

```bash
# Ingredient Management
kitchn stock ./path/to/ingredient.ing
kitchn stock ./my-theme.bag
kitchn pantry
kitchn pantry clean
kitchn cook

# Packaging
kitchn wrap ./ingredients-dir/
kitchn wrap ./ingredients/ --output theme.bag

# Performance
kitchn bake             # Pre-compile configs

# Debugging
kitchn --debug
kitchn cook --debug

# Logging
kitchn-log error SYSTEM "Message"
kitchn-log boot_ok
kitchn-log boot_ok --app MyApp
```

## Memory Checking

```bash
just memcheck           # ASan + LSan
```

## Stats

```bash
just stats              # LOC, binary sizes
```
