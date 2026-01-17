#!/bin/bash

# Build script for FIVE Solana Program
# Builds the program for deployment to localnet/devnet
set -euo pipefail

echo "🚀 Building FIVE Solana Program"
echo "================================"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse build mode
BUILD_MODE="${1:-prod}"

# Step 1: Ensure target directory exists
echo -e "\n${YELLOW}Step 1: Setting up build environment...${NC}"
mkdir -p five-solana/target/deploy

# Copy keypair to expected location
if [ -f "five-solana/mito-program-keypair-fixed.json" ]; then
    cp five-solana/mito-program-keypair-fixed.json five-solana/target/deploy/five-keypair.json
    echo -e "${GREEN}✅ Program keypair copied${NC}"
    
    # Show the program ID
    PROGRAM_ID=$(solana-keygen pubkey five-solana/mito-program-keypair-fixed.json 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo -e "${BLUE}🔑 Program ID: $PROGRAM_ID${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  No program keypair found, will generate new one${NC}"
fi

# Step 2: Build the Solana program
echo -e "\n${YELLOW}Step 2: Building Solana Program...${NC}"

cd five-solana

if [ "$BUILD_MODE" = "debug" ]; then
    echo -e "${BLUE}Building with debug features...${NC}"
    cargo build-sbf --features debug-logs --sbf-out-dir target/deploy
else
    echo -e "${BLUE}Building for production (optimized)...${NC}"
    cargo build-sbf --no-default-features --features production --sbf-out-dir target/deploy
fi

BUILD_RESULT=$?
cd ..

if [ $BUILD_RESULT -ne 0 ]; then
    echo -e "${RED}❌ Solana program build failed!${NC}"
    echo ""
    echo "Common issues and solutions:"
    echo "1. Stack overflow: Reduce stack usage in ExecutionContext"
    echo "2. Missing dependencies: Run 'cargo update' in five-solana"
    echo "3. Rust version: Ensure using 'rustup override set 1.79.0' in five-solana"
    exit 1
fi

echo -e "${GREEN}✅ Solana program build successful!${NC}"

# Step 3: Check for built artifacts
echo -e "\n${YELLOW}Step 3: Verifying build artifacts...${NC}"

SO_FILE="five-solana/target/deploy/five.so"
if [ -f "$SO_FILE" ]; then
    echo -e "${GREEN}✅ Program binary found: $SO_FILE${NC}"
    
    # Show file size
    FILE_SIZE=$(ls -lh "$SO_FILE" | awk '{print $5}')
    echo -e "${BLUE}📦 Binary size: $FILE_SIZE${NC}"
else
    echo -e "${RED}❌ Program binary not found at $SO_FILE${NC}"
    echo "Build may have failed silently or output is in different location"
    exit 1
fi

echo ""
echo -e "${GREEN}✨ Build completed successfully!${NC}"
echo ""
echo "Next steps:"
echo "1. Start local validator: solana-test-validator"
echo "2. Deploy program: solana program deploy $SO_FILE"
echo "3. Initialize VM state (see deploy-and-init.sh)"
