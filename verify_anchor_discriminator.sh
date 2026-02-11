#!/bin/bash
set -e

# Path to the DSL compiler binary (using cargo run for dev)
COMPILER="cargo run --quiet --package five-dsl-compiler --bin compile_script --"

# Fixture path
FIXTURE="five-templates/cpi-examples/anchor-program-call-e2e.v"
OUTPUT_BIN="five-templates/cpi-examples/anchor-program-call-e2e.bin"

echo "Compiling $FIXTURE..."
$COMPILER $FIXTURE

if [ ! -f "$OUTPUT_BIN" ]; then
    echo "Error: Output binary not found at $OUTPUT_BIN"
    exit 1
fi

echo "Verifying discriminator bytes..."

# Expected discriminator for 'initialize': sha256("global:initialize")[..8] = afaf6d1f0d989bed
# Expected discriminator for 'reset': 0102030405060708

# Check using xxd/grep
if xxd -p -c 1000 "$OUTPUT_BIN" | grep -q "afaf6d1f0d989bed"; then
    echo "SUCCESS: Found derived discriminator 'afaf6d1f0d989bed' (initialize)"
else
    echo "FAILURE: Derived discriminator 'afaf6d1f0d989bed' NOT found"
    xxd -p "$OUTPUT_BIN"
    exit 1
fi

if xxd -p -c 1000 "$OUTPUT_BIN" | grep -q "0102030405060708"; then
    echo "SUCCESS: Found explicit discriminator '0102030405060708' (reset)"
else
    echo "FAILURE: Explicit discriminator '0102030405060708' NOT found"
    exit 1
fi

echo "All verifications passed!"
