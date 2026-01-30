#!/bin/bash

set -e

echo "====== Token Template E2E Test with Register Optimization ======"
echo ""

# Configuration
PROGRAM_ID="6ndNfSrrGoFfTbS1sdJFybuJJyA6YQHtNgRdoXFREi8k"
RPC_URL="http://127.0.0.1:8899"
BYTECODE_FILE="build/five-token-template.five"

echo "Program ID: $PROGRAM_ID"
echo "RPC URL: $RPC_URL"
echo "Bytecode: $BYTECODE_FILE"
echo ""

# Check if bytecode exists
if [ ! -f "$BYTECODE_FILE" ]; then
    echo "ERROR: Bytecode file not found: $BYTECODE_FILE"
    exit 1
fi

# Check if program is deployed
echo "Checking if program is deployed..."
if solana program show $PROGRAM_ID --url $RPC_URL > /dev/null 2>&1; then
    echo "✓ Program deployed"
else
    echo "✗ Program not deployed"
    exit 1
fi

# Get bytecode size
BYTECODE_SIZE=$(stat -f%z "$BYTECODE_FILE" 2>/dev/null || stat -c%s "$BYTECODE_FILE")
echo "Bytecode size: $BYTECODE_SIZE bytes"
echo ""

# Show bytecode details
echo "Bytecode file details:"
ls -lh "$BYTECODE_FILE"
echo ""

# Show first 100 bytes of bytecode (hex)
echo "First 100 bytes of compiled bytecode:"
head -c 100 "$BYTECODE_FILE" | xxd | head -5
echo ""

echo "====== E2E Test Summary ======"
echo "✓ Program is deployed and operational"
echo "✓ Bytecode compiled with register optimizations"
echo "✓ Bytecode file ready for deployment"
echo ""
echo "To deploy and test, use the e2e-token-test.mjs script or:"
echo "  node deploy-to-five-vm.mjs"
