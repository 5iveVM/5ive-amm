#!/bin/bash

# Five Workspace Build Script
# Builds all components in the Five Protocol ecosystem
set -euo pipefail

echo "🔨 Building Five Workspace"
echo "=========================="

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${BLUE}Project root: $PROJECT_ROOT${NC}"

cd "$PROJECT_ROOT"

# Build all Rust components using Cargo workspace
echo -e "\n${YELLOW}Building Rust components...${NC}"
cargo build --workspace

echo -e "\n${YELLOW}Building CLI with WebAssembly...${NC}"
cd five-cli
if [ -f "package.json" ]; then
    npm install
    npm run build
else
    echo -e "${YELLOW}⚠️  CLI package.json not found, skipping npm build${NC}"
fi

cd "$PROJECT_ROOT"

echo -e "\n${YELLOW}Building WebAssembly bindings...${NC}"
cd five-wasm
if [ -f "build.sh" ]; then
    ./build.sh
else
    echo -e "${YELLOW}⚠️  WASM build script not found, skipping WASM build${NC}"
fi

cd "$PROJECT_ROOT"

echo -e "\n${GREEN}✅ Five Workspace build completed!${NC}"

echo -e "\n${BLUE}Built components:${NC}"
echo "  ✓ five-protocol"
echo "  ✓ five-vm-mito"  
echo "  ✓ five-dsl-compiler"
echo "  ✓ five-solana"
echo "  ✓ five-cli"
echo "  ✓ five-wasm"
echo "  ✓ five-mcp"

echo -e "\n${YELLOW}Next steps:${NC}"
echo "  🧪 Run tests: ./scripts/test-workspace.sh"
echo "  🚀 Production build: ./scripts/build-production-vm.sh"
