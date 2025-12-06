# Hyprcore Justfile

default: build

# Build release binaries
build:
    cargo build --release

# Run the installation script (Config setup + Build)
install:
    ./install.sh

# Run tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean

# Check code quality
lint:
    cargo clippy -- -D warnings
    cargo fmt -- --check

# Run corelog example
run-log preset="test_pass":
    ./target/release/corelog {{preset}}

# Run hyprcore sync
sync:
    ./target/release/hyprcore sync

# Install example fragment
install-waybar:
    ./target/release/hyprcore install ./assets/fragments/waybar.frag


# Uninstall everything
uninstall:
    ./uninstall.sh

# Pre-commit checks (Lint + Format)
pre-commit:
    cargo clippy -- -D warnings

# Show demo logs
show:
    ./assets/scripts/demo_logs.sh

# Audit dependencies
audit:
    cargo audit

# Run benchmarks (Criterion + Hyperfine)
bench:
    cargo bench
    ./assets/scripts/bench.sh
