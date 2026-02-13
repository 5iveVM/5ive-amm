#!/bin/bash
# Build script for Five LSP WASM bindings
#
# Usage:
#   ./build-wasm.sh              # Build for release (with copy to frontend)
#   ./build-wasm.sh --dev        # Build for development
#   ./build-wasm.sh --no-copy    # Build without copying to frontend

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
FRONTEND_DIR="$REPO_ROOT/five-frontend"
BUILD_MODE="--release"
COPY_TO_FRONTEND=true

echo "Building Five LSP WASM bindings..."

# Parse arguments
for arg in "$@"; do
    case "$arg" in
        --dev)
            BUILD_MODE="--dev"
            echo "Mode: Development"
            ;;
        --no-copy)
            COPY_TO_FRONTEND=false
            echo "Will not copy to frontend"
            ;;
        *)
            echo "Unknown argument: $arg"
            exit 1
            ;;
    esac
done

[ "$BUILD_MODE" = "--dev" ] || echo "Mode: Release"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "ERROR: wasm-pack is not installed. Install it with:"
    echo "  cargo install wasm-pack"
    exit 1
fi

# Build WASM bindings (suppress Rust warnings for clean output)
cd "$SCRIPT_DIR"
export RUSTFLAGS="-A warnings"

if [ "$BUILD_MODE" = "--dev" ]; then
    wasm-pack build --dev --target web --out-dir pkg 2>&1 | grep -v "newer version of wasm-pack"
else
    wasm-pack build --release --target web --out-dir pkg 2>&1 | grep -v "newer version of wasm-pack"
fi

echo "✓ WASM build complete!"

# Copy to frontend if requested and it exists
if [ "$COPY_TO_FRONTEND" = true ] && [ -d "$FRONTEND_DIR" ]; then
    echo ""
    echo "Copying to frontend public directory..."
    mkdir -p "$FRONTEND_DIR/public/wasm"

    for file in five_lsp.js five_lsp.d.ts five_lsp_bg.wasm five_lsp_bg.wasm.d.ts .gitignore; do
        if [ -f "pkg/$file" ]; then
            cp "pkg/$file" "$FRONTEND_DIR/public/wasm/$file"
            echo "  ✓ Copied: $file"
        fi
    done

    echo ""
    echo "✓ WASM files copied to five-frontend/public/wasm/"
fi

echo ""
echo "Next steps:"
echo "1. Navigate to five-frontend: cd ../five-frontend"
echo "2. Install dependencies: npm install"
echo "3. Start development: npm run dev"
