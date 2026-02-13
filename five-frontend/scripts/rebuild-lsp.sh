#!/bin/bash
# Rebuild Five LSP WASM and copy to frontend
#
# Usage:
#   npm run rebuild:lsp        # Build and copy LSP WASM
#   npm run rebuild:lsp:dev    # Build in dev mode

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
FRONTEND_DIR="$(dirname "$SCRIPT_DIR")"
LSP_DIR="$FRONTEND_DIR/../five-lsp"

echo "🔨 Rebuilding Five LSP WASM..."
echo ""

# Check if LSP directory exists
if [ ! -d "$LSP_DIR" ]; then
    echo "ERROR: LSP directory not found at $LSP_DIR"
    exit 1
fi

# Build the WASM (it will automatically copy to frontend)
cd "$LSP_DIR"

if [ "$1" = "--dev" ]; then
    ./build-wasm.sh --dev
else
    ./build-wasm.sh
fi

echo ""
echo "✅ LSP WASM rebuilt and copied to five-frontend/public/wasm/"
echo ""
echo "The updated LSP includes:"
echo "  ✓ Constraint annotation autocomplete (@signer, @mut, @init, @writable)"
echo "  ✓ Semantic analysis infrastructure"
echo "  ✓ Workspace document management"
echo "  ✓ Multi-error diagnostics"
echo ""
echo "Start the dev server with: npm run dev"
