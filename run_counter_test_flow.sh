#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Starting Five VM Counter Test Flow...${NC}"

# Cleanup function
cleanup() {
    if [ -n "$SURFPOOL_PID" ]; then
        echo "Stopping Surfpool (PID $SURFPOOL_PID)..."
        kill $SURFPOOL_PID || true
    fi
}
trap cleanup EXIT

# Clear port 8899
echo "Clearing port 8899..."
lsof -ti :8899 | xargs kill -9 || true

# Build Surfpool
if [ ! -f ./five-surfpool/target/release/surfpool ]; then
    echo -e "${BLUE}Building Surfpool...${NC}"
    cd five-surfpool
    cargo build --release
    cd ..
else
    echo -e "${GREEN}Surfpool already built.${NC}"
fi

# Start Surfpool
echo -e "${BLUE}Starting Surfpool...${NC}"
FIVE_VALIDATOR=surfpool ./five-surfpool/target/release/surfpool start --no-tui -y > surfpool.log 2>&1 &
SURFPOOL_PID=$!
echo "Surfpool PID: $SURFPOOL_PID"

# Wait for Surfpool
echo "Waiting for Surfpool to be ready..."
count=0
while ! curl -s http://127.0.0.1:8899/health > /dev/null; do
    sleep 1
    count=$((count+1))
    if [ $count -ge 30 ]; then 
        echo -e "${RED}Surfpool failed to start.${NC}"
        cat surfpool.log
        exit 1
    fi
    echo -n "."
done
echo -e "\n${GREEN}Surfpool is ready!${NC}"

# Setup Payer
if [ ! -f ~/.config/solana/id.json ]; then
    echo "Setting up payer keypair..."
    mkdir -p ~/.config/solana
    if [ -f payer.json ]; then
        cp payer.json ~/.config/solana/id.json
    else
        solana-keygen new --no-bip39-passphrase -o ~/.config/solana/id.json
    fi
fi

# Deploy Five VM and Init State
echo -e "${BLUE}Deploying Five VM...${NC}"
./five-scripts/deploy-and-init.sh localnet ~/.config/solana/id.json prod > deploy.log 2>&1
cat deploy.log

# Extract IDs
PROGRAM_ID=$(grep "Program ID:" deploy.log | tail -n 1 | sed 's/\x1b\[[0-9;]*m//g' | awk '{print $3}' | tr -d '\r')
VM_STATE_PDA=$(grep "VM State PDA:" deploy.log | tail -n 1 | sed 's/\x1b\[[0-9;]*m//g' | awk '{print $4}' | tr -d '\r')

if [ -z "$PROGRAM_ID" ] || [ -z "$VM_STATE_PDA" ]; then
    echo -e "${RED}Failed to extract deployment info.${NC}"
    exit 1
fi

echo -e "${GREEN}Program ID: $PROGRAM_ID${NC}"
echo -e "${GREEN}VM State PDA: $VM_STATE_PDA${NC}"

# Run Counter Tests
echo -e "${BLUE}Running Counter E2E Tests...${NC}"
cd five-templates/counter

# Export env vars for deploy-to-five-vm.mjs
export FIVE_PROGRAM_ID="$PROGRAM_ID"
export VM_STATE_PDA="$VM_STATE_PDA"
export RPC_URL="http://127.0.0.1:8899"

# Install dependencies if needed (assuming they might be missing in fresh clone)
if [ ! -d "node_modules" ]; then
    echo "Installing counter template dependencies..."
    npm install
fi

# Run the test script
./e2e-counter-test.sh --deploy --verbose

echo -e "${GREEN}All steps completed successfully!${NC}"
