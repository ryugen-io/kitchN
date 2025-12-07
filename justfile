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
    ../utils/hyprcore/demo_logs.sh

# Audit dependencies
audit:
    cargo audit

# Run benchmarks (Criterion + Hyperfine)
bench:
    cargo bench
    ../utils/hyprcore/bench.sh

# Run C++ FFI Example
example-cpp: build
    @echo "Running C++ Example..."
    cd assets/examples/cpp && make main && ./main

# Run Python FFI Example
example-python: build
    @echo "Running Python Example..."
    cd assets/examples/python && python3 main.py

# Run Rust Native Example
example-rust:
    @echo "Running Rust Example..."
    cd assets/examples/rust && cargo run

# Run all examples
examples: example-cpp example-python example-rust

# Show project statistics (LOC, Sizes)
stats:
    ../utils/hyprcore/stats.sh .
