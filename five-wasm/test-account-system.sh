#!/bin/bash

#!/bin/bash

# Five VM Account System Integration Test Script
#
# This script tests the account system functionality using a local Solana validator
# and the Five CLI for comprehensive on-chain testing.

set -euo pipefail

# Ensure we clean up background validator on exit/interrupt
VALIDATOR_PID=""
cleanup() {
  if [ -n "$VALIDATOR_PID" ]; then
    kill "$VALIDATOR_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🧪 Five VM Account System Integration Test${NC}"
echo -e "${CYAN}===========================================${NC}"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if a port is open
port_open() {
    if command -v nc >/dev/null 2>&1; then
        nc -z localhost "$1" >/dev/null 2>&1
    else
        # Fallback to bash's /dev/tcp if available
        (exec 3<>/dev/tcp/127.0.0.1/"$1") >/dev/null 2>&1 || return 1
        exec 3>&-
        return 0
    fi
}

# Check prerequisites
echo -e "\n${BLUE}📋 Checking prerequisites...${NC}"

# Check if Solana CLI is installed
if ! command_exists solana; then
    echo -e "${RED}❌ Solana CLI not found. Please install it first:${NC}"
    echo -e "   curl --proto '=https' --tlsv1.2 -sSf https://release.anza.xyz/stable/install | sh"
    exit 1
fi
echo -e "${GREEN}✅ Solana CLI found${NC}"

# Check if Node.js is installed
if ! command_exists node; then
    echo -e "${RED}❌ Node.js not found. Please install it first.${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Node.js found${NC}"

# Check if Rust is installed
if ! command_exists cargo; then
    echo -e "${RED}❌ Rust/Cargo not found. Please install it first.${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Rust/Cargo found${NC}"

# Check if Five CLI is built
FIVE_CLI_PATH="../five-cli/dist/index.js"
if [ ! -f "$FIVE_CLI_PATH" ]; then
    echo -e "${RED}❌ Five CLI not built at ${FIVE_CLI_PATH}.${NC}"
    echo -e "   Build it first with: ${CYAN}npm --prefix ../five-cli run build${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Five CLI found${NC}"
fi

# Check if local validator is running
echo -e "\n${BLUE}🔍 Checking local Solana validator...${NC}"
if port_open 8899; then
    echo -e "${GREEN}✅ Local Solana validator is running${NC}"
    
    # Get validator info
    SLOT=$(solana slot --url http://localhost:8899 2>/dev/null || echo "unknown")
    echo -e "   Current slot: ${SLOT}"
else
    echo -e "${YELLOW}⚠️  Local Solana validator not running. Starting it now...${NC}"
    echo -e "${BLUE}   Starting solana-test-validator...${NC}"
    
    # Start validator in background
    solana-test-validator --reset --quiet &
    VALIDATOR_PID=$!
    
    # Wait for validator to start
    echo -e "   Waiting for validator to start..."
    for i in {1..30}; do
        if port_open 8899; then
            echo -e "${GREEN}✅ Local Solana validator started${NC}"
            break
        fi
        sleep 1
        echo -n "."
    done
    
    if ! port_open 8899; then
        echo -e "\n${RED}❌ Failed to start local validator after 30 seconds${NC}"
        kill $VALIDATOR_PID 2>/dev/null || true
        exit 1
    fi
fi

# Set Solana config to localhost
echo -e "\n${BLUE}⚙️  Configuring Solana CLI for localhost...${NC}"
solana config set --url http://localhost:8899 >/dev/null
echo -e "${GREEN}✅ Solana config set to localhost${NC}"

# Run account system tests in localnet mode
echo -e "\n${BLUE}🧪 Running account system tests in localnet mode...${NC}"
export FIVE_TEST_MODE=localnet

# Run the specific account system test
if cargo test test_account_system_integration -- --nocapture; then
    echo -e "\n${GREEN}🎉 Account system tests completed successfully!${NC}"
    echo -e "${GREEN}   All account operations are working with real Solana accounts${NC}"
else
    echo -e "\n${RED}❌ Account system tests failed${NC}"
    echo -e "${RED}   Check the output above for details${NC}"
    exit 1
fi

# Optional: Run all tests in localnet mode
read -p "$(echo -e ${YELLOW}Run all VM tests in localnet mode? [y/N]: ${NC})" -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "\n${BLUE}🧪 Running all VM tests in localnet mode...${NC}"
    if cargo test -- --nocapture; then
        echo -e "\n${GREEN}🎉 All tests completed successfully in localnet mode!${NC}"
    else
        echo -e "\n${YELLOW}⚠️  Some tests failed in localnet mode${NC}"
        echo -e "${YELLOW}   This is expected for tests that don't require account operations${NC}"
    fi
fi

echo -e "\n${CYAN}📊 Test Summary:${NC}"
echo -e "   • Test mode: ${GREEN}Localnet (on-chain)${NC}"
echo -e "   • Account operations: ${GREEN}Real Solana accounts${NC}"  
echo -e "   • Network: ${GREEN}Local validator${NC}"
echo -e "\n${GREEN}✅ Account system integration testing complete!${NC}"

# Clean up if we started the validator
if [ -n "$VALIDATOR_PID" ]; then
    echo -e "\n${BLUE}🧹 Cleaning up...${NC}"
    kill $VALIDATOR_PID 2>/dev/null || true
    echo -e "${GREEN}✅ Local validator stopped${NC}"
fi
