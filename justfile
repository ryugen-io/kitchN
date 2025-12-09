# Kitchn Justfile

default: build

# Build release binaries
build:
    cargo build --release

# Run the installation script (Config setup + Build)
install:
    ./install.sh

# Run tests
test:
    cargo test --workspace

# Clean build artifacts
clean:
    cargo clean

# Check code quality
lint:
    cargo clippy -- -D warnings
    cargo fmt -- --check

# Run kitchn-log example
run-log preset="test_pass":
    ./target/release/kitchn-log {{preset}}

# Run kitchn cook (sync)
cook:
    ./target/release/kitchn cook

# Stock example ingredient
stock-waybar:
    ./target/release/kitchn stock ./assets/ingredients/waybar.ing



# Uninstall everything
uninstall:
    ./uninstall.sh

# Pre-commit checks (Lint + Format)
pre-commit:
    cargo clippy --all-targets --all-features -- -D warnings

# Show demo logs
show:
    ../utils/kitchn/demo_logs.sh

# Audit dependencies
audit:
    cargo audit

# Run benchmarks (Criterion + Hyperfine)
bench:
    cargo bench
    ../utils/kitchn/bench.sh

# Run benchmarks for kitchn_lib only
bench-lib:
    cargo bench -p kitchn_lib

# Run tests for kitchn_lib only
test-lib:
    cargo test -p kitchn_lib

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

# Memory leak check (ASan + LSan)
memcheck: build
    ../utils/kitchn/memcheck.sh .

# Show project statistics (LOC, Sizes)
stats:
    ../utils/kitchn/stats.sh .

# Debug Kitchn (Spawns listener)
debug:
    cargo run --bin kitchn -- --debug

# Optimize binaries with UPX
compact: build
    @echo "Compacting binaries..."
    @upx --best --lzma target/release/kitchn
    @upx --best --lzma target/release/kitchn-log
