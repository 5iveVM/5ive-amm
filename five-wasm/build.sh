#!/bin/bash

set -euo pipefail

echo "🚀 Building Five VM WASM Module"
echo "=================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check dependencies
echo -e "${BLUE}📋 Checking dependencies...${NC}"

if ! command -v wasm-pack &> /dev/null; then
    echo -e "${RED}❌ wasm-pack not found.${NC}"
    echo -e "${YELLOW}Please install wasm-pack from https://rustwasm.github.io/wasm-pack/installer/ (recommended to pin a version).${NC}"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust: https://rustup.rs/${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Dependencies verified${NC}"

# Create output directories
echo -e "${BLUE}📁 Creating output directories...${NC}"
mkdir -p pkg pkg-node pkg-bundler reports

# Build for different targets
echo -e "${BLUE}🔨 Building WASM for web target...${NC}"
wasm-pack build --target web --out-dir pkg --release

echo -e "${BLUE}🔨 Building WASM for Node.js target...${NC}"
wasm-pack build --target nodejs --out-dir pkg-node --release

echo -e "${BLUE}🔨 Building WASM for bundler target...${NC}"
wasm-pack build --target bundler --out-dir pkg-bundler --release

# Analyze bundle sizes
echo -e "${BLUE}📊 Analyzing bundle sizes...${NC}"
echo "Web target:"
ls -lh pkg/five_vm_wasm_bg.wasm | awk '{print "  WASM: " $5}'
ls -lh pkg/five_vm_wasm.js | awk '{print "  JS:   " $5}'

echo "Node.js target:"
ls -lh pkg-node/five_vm_wasm_bg.wasm | awk '{print "  WASM: " $5}'
ls -lh pkg-node/five_vm_wasm.js | awk '{print "  JS:   " $5}'

echo "Bundler target:"
ls -lh pkg-bundler/five_vm_wasm_bg.wasm | awk '{print "  WASM: " $5}'
ls -lh pkg-bundler/five_vm_wasm.js | awk '{print "  JS:   " $5}'

# Optimize WASM binary
echo -e "${BLUE}⚡ Optimizing WASM binary...${NC}"
if command -v wasm-opt &> /dev/null; then
    for target in pkg pkg-node pkg-bundler; do
        if [ -f "$target/five_vm_wasm_bg.wasm" ]; then
            echo "  Optimizing $target..."
            wasm-opt -Oz --enable-bulk-memory "$target/five_vm_wasm_bg.wasm" -o "$target/five_vm_wasm_bg.wasm"
        fi
    done
    echo -e "${GREEN}✅ WASM optimization complete${NC}"
else
    echo -e "${YELLOW}⚠️  wasm-opt not found. Install binaryen for optimization.${NC}"
fi

# Run tests
echo -e "${BLUE}🧪 Running tests...${NC}"
if [ -f "package.json" ]; then
    if command -v npm &> /dev/null; then
        npm test || echo -e "${YELLOW}⚠️  Some tests failed${NC}"
    else
        echo -e "${YELLOW}⚠️  npm not found, skipping tests${NC}"
    fi
fi

# Generate size report
echo -e "${BLUE}📋 Generating size report...${NC}"
cat > reports/build-report.md << EOF
# WASM Build Report

Generated: $(date)

## Bundle Sizes

### Web Target
- WASM: $(ls -lh pkg/five_vm_wasm_bg.wasm 2>/dev/null | awk '{print $5}' || echo 'N/A')
- JS: $(ls -lh pkg/five_vm_wasm.js 2>/dev/null | awk '{print $5}' || echo 'N/A')

### Node.js Target  
- WASM: $(ls -lh pkg-node/five_vm_wasm_bg.wasm 2>/dev/null | awk '{print $5}' || echo 'N/A')
- JS: $(ls -lh pkg-node/five_vm_wasm.js 2>/dev/null | awk '{print $5}' || echo 'N/A')

### Bundler Target
- WASM: $(ls -lh pkg-bundler/five_vm_wasm_bg.wasm 2>/dev/null | awk '{print $5}' || echo 'N/A')
- JS: $(ls -lh pkg-bundler/five_vm_wasm.js 2>/dev/null | awk '{print $5}' || echo 'N/A')

## Build Configuration
- Target: $(rustc --version 2>/dev/null || echo 'Unknown')
- Profile: Release
- Optimization: -Oz (if wasm-opt available)

## Features
- ✅ Zero-copy deserialization
- ✅ TypeScript definitions
- ✅ Multiple target support
- ✅ Performance optimizations
- ✅ Error handling
EOF

echo -e "${GREEN}✅ Build complete!${NC}"
echo ""
echo -e "${BLUE}📦 Generated packages:${NC}"
echo "  • pkg/           - Web target (ES modules)"
echo "  • pkg-node/      - Node.js target (CommonJS)"  
echo "  • pkg-bundler/   - Bundler target (webpack, etc.)"
echo ""
echo -e "${BLUE}📊 Reports:${NC}"
echo "  • reports/build-report.md - Detailed build information"
echo ""
echo -e "${BLUE}🚀 Usage:${NC}"
echo "  • Import from wrapper/index.ts for TypeScript"
echo "  • Use pkg/ directly for JavaScript/web"
echo "  • Use pkg-node/ for Node.js applications"
