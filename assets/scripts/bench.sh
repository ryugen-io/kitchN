#!/usr/bin/env bash
set -e

# Build release binaries first
echo "Building release binaries..."
cargo build --release

HYPRCORE="./target/release/hyprcore"
CORELOG="./target/release/corelog"

# Check if hyperfine is installed
if ! command -v hyperfine &> /dev/null; then
    echo "Hyperfine not found. Please install it to run these benchmarks."
    echo "cargo install hyperfine"
    exit 1
fi

echo "Benchmarking corelog..."
hyperfine --warmup 3 \
    "$CORELOG error SYSTEM 'Test Message'" \
    "$CORELOG boot_ok"

echo "Benchmarking hyprcore..."
# Create 100 dummy fragments for unpacking
mkdir -p .bench_tmp/fragments
for i in {1..100}; do
    echo "[meta]
id = \"bench_$i\"
[[templates]]
target = \"/tmp/bench_$i\"
content = \"bench\"" > .bench_tmp/fragments/bench_$i.frag
done

$HYPRCORE pack .bench_tmp/fragments -o .bench_tmp/bench.fpkg

hyperfine --warmup 3 \
    "$HYPRCORE pack .bench_tmp/fragments -o .bench_tmp/bench_out.fpkg" \
    "$HYPRCORE install .bench_tmp/bench.fpkg" \
    "$HYPRCORE list"

# Cleanup
rm -rf .bench_tmp
