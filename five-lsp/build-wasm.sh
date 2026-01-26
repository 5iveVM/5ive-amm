#!/bin/bash
# Build script for Five LSP WASM bindings
#
# Usage:
#   ./build-wasm.sh              # Build for release
#   ./build-wasm.sh --dev        # Build for development

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BUILD_MODE=${1:-"--release"}

echo "Building Five LSP WASM bindings..."
echo "Mode: $BUILD_MODE"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "ERROR: wasm-pack is not installed. Install it with:"
    echo "  cargo install wasm-pack"
    exit 1
fi

# Build WASM bindings
cd "$SCRIPT_DIR"

if [ "$BUILD_MODE" = "--dev" ]; then
    wasm-pack build --dev --target web --out-dir pkg
else
    wasm-pack build --release --target web --out-dir pkg
fi

echo "✓ WASM build complete!"
echo ""
echo "Next steps:"
echo "1. Copy pkg/ to your frontend project:"
echo "   cp -r pkg ../five-frontend/public/wasm/"
echo ""
echo "2. In your TypeScript, import the module:"
echo "   import * as wasmModule from '/wasm/five_lsp.js';"
echo "   const lsp = wasmModule.FiveLspWasm.new();"
echo "   const diagnostics = lsp.get_diagnostics('file:///test.v', source);"
