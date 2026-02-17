#!/bin/bash

# Complete deployment script for FIVE Solana Program
# Deploys the program and initializes VM state
set -euo pipefail

echo "🚀 FIVE Solana Program Deployment"
echo "=================================="

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse network parameter
NETWORK="${1:-localnet}"
PAYER_KEYPAIR="${2:-~/.config/solana/id.json}"
BUILD_MODE="${3:-${BUILD_MODE:-}}"
VM_STATE_KEYPAIR="${VM_STATE_KEYPAIR:-five-solana/target/deploy/vm-state-keypair.json}"
VM_STATE_SIZE=56
MAX_SIGN_ATTEMPTS="${MAX_SIGN_ATTEMPTS:-50}"
SURFPOOL_WRAPPER="$(pwd)/five-surfpool/surfpool"
# Expand tilde in payer path if present
if [[ "$PAYER_KEYPAIR" == ~* ]]; then
  PAYER_KEYPAIR="${PAYER_KEYPAIR/#\~/$HOME}"
fi

echo -e "${BLUE}Network: $NETWORK${NC}"
echo -e "${BLUE}Payer: $PAYER_KEYPAIR${NC}"
if [[ -n "${BUILD_MODE}" ]]; then
  echo -e "${BLUE}Build mode: $BUILD_MODE${NC}"
fi
echo ""

# Step 1: Set Solana cluster
echo -e "\n${YELLOW}Step 1: Setting Solana cluster...${NC}"
RPC_URL=""
case $NETWORK in
    "localnet")
        RPC_URL="${FIVE_LOCAL_RPC_URL:-${SOLANA_URL:-http://127.0.0.1:8899}}"
    if [[ "${SKIP_SOLANA_CONFIG_SET:-}" == "1" ]]; then
        echo -e "${YELLOW}Skipping Solana config set (SKIP_SOLANA_CONFIG_SET=1)${NC}"
    else
        solana config set --url "$RPC_URL"
    fi
        ;;
    "devnet")
        RPC_URL="https://api.devnet.solana.com"
        solana config set --url devnet
        ;;
    "testnet")
        RPC_URL="https://api.testnet.solana.com"
        solana config set --url testnet
        ;;
    *)
        echo -e "${RED}❌ Unknown network: $NETWORK${NC}"
        echo "Supported networks: localnet, devnet, testnet"
        exit 1
        ;;
esac

if [[ -z "$RPC_URL" ]]; then
    echo -e "${RED}❌ Unable to determine RPC URL for network: $NETWORK${NC}"
    exit 1
fi

echo -e "${BLUE}RPC URL: $RPC_URL${NC}"

if ! PAYER_PUBKEY=$(solana-keygen pubkey "$PAYER_KEYPAIR" 2>/dev/null); then
    echo -e "${RED}❌ Unable to read payer keypair at $PAYER_KEYPAIR${NC}"
    exit 1
fi

# Step 2: Ensure local validator (Surfpool/localnet)
if [ "$NETWORK" = "localnet" ]; then
    echo -e "\n${YELLOW}Step 2: Ensuring local validator is running...${NC}"

    if solana cluster-version --url "$RPC_URL" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Local validator is running${NC}"
    else
        if [[ -x "$SURFPOOL_WRAPPER" ]] && command -v surfpool >/dev/null 2>&1; then
            echo -e "${BLUE}Starting Surfpool-managed localnet instance...${NC}"
            FIVE_VALIDATOR=${FIVE_VALIDATOR:-surfpool} "$SURFPOOL_WRAPPER" start --no-tui -y
        else
            echo -e "${YELLOW}Local validator not detected. Start one in another terminal:${NC}"
            echo -e "${BLUE}solana-test-validator --reset${NC}"
            echo -e "${YELLOW}Press Enter when validator is ready...${NC}"
            read -r
        fi

        if solana cluster-version --url "$RPC_URL" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ Validator responding on $RPC_URL${NC}"
        else
            echo -e "${RED}❌ Validator is still unavailable at $RPC_URL${NC}"
            exit 1
        fi
    fi
fi

# Step 3: Fund payer on localnet
if [ "$NETWORK" = "localnet" ]; then
    echo -e "\n${YELLOW}Step 3: Funding payer wallet for localnet...${NC}"
    MIN_BALANCE_LAMPORTS=$((5 * 1000000000))   # 5 SOL
    AIRDROP_LAMPORTS=$((50 * 1000000000))      # 50 SOL

    BALANCE_LAMPORTS=$(solana balance "$PAYER_PUBKEY" --url "$RPC_URL" --lamports 2>/dev/null | awk '{print $1}' || echo 0)
    [[ "$BALANCE_LAMPORTS" =~ ^[0-9]+$ ]] || BALANCE_LAMPORTS=0

    if [ "$BALANCE_LAMPORTS" -lt "$MIN_BALANCE_LAMPORTS" ]; then
        echo -e "${BLUE}Requesting airdrop of 50 SOL for payer $PAYER_PUBKEY...${NC}"
        solana airdrop --url "$RPC_URL" "$((AIRDROP_LAMPORTS / 1000000000))" "$PAYER_PUBKEY"

        BALANCE_LAMPORTS=$(solana balance "$PAYER_PUBKEY" --url "$RPC_URL" --lamports 2>/dev/null | awk '{print $1}' || echo 0)
        [[ "$BALANCE_LAMPORTS" =~ ^[0-9]+$ ]] || BALANCE_LAMPORTS=0

        if [ "$BALANCE_LAMPORTS" -lt "$MIN_BALANCE_LAMPORTS" ]; then
            echo -e "${RED}❌ Airdrop failed or insufficient balance (current: $BALANCE_LAMPORTS lamports)${NC}"
            exit 1
        fi
    fi

    echo -e "${GREEN}✅ Payer funded. Balance: ${BALANCE_LAMPORTS} lamports${NC}"
fi

# Step 4: Build the program (optional)
if [[ -n "${BUILD_MODE}" ]]; then
    if [[ "${BUILD_MODE}" != "prod" && "${BUILD_MODE}" != "debug" ]]; then
        echo -e "${RED}❌ Unknown build mode: ${BUILD_MODE}${NC}"
        echo "Supported build modes: prod, debug"
        exit 1
    fi
    if [[ "${SKIP_BUILD:-}" == "1" ]]; then
        echo -e "${YELLOW}Skipping build (SKIP_BUILD=1)${NC}"
    else
        echo -e "\n${YELLOW}Step 4: Building FIVE program (${BUILD_MODE})...${NC}"
        ./five-scripts/build-five-solana.sh "${BUILD_MODE}"
    fi
fi

# Step 5: Deploy the program
echo -e "\n${YELLOW}Step 5: Deploying FIVE program...${NC}"

SO_FILE="five-solana/target/deploy/five.so"
KEYPAIR_FILE="five-solana/target/deploy/five-keypair.json"

if [ ! -f "$SO_FILE" ]; then
    echo -e "${RED}❌ Program binary not found: $SO_FILE${NC}"
    exit 1
fi

if [ ! -f "$KEYPAIR_FILE" ]; then
    echo -e "${RED}❌ Program keypair not found: $KEYPAIR_FILE${NC}"
    exit 1
fi

# Get program ID
PROGRAM_ID=$(solana-keygen pubkey "$KEYPAIR_FILE")
echo -e "${BLUE}🔑 Program ID: $PROGRAM_ID${NC}"

# Deploy the program
echo -e "${YELLOW}Deploying program...${NC}"
if [ "$NETWORK" = "localnet" ]; then
    echo -e "${BLUE}Using --use-rpc for local/surfpool validator compatibility${NC}"
    solana program deploy "$SO_FILE" --keypair "$PAYER_KEYPAIR" --program-id "$KEYPAIR_FILE" --use-rpc --max-sign-attempts "$MAX_SIGN_ATTEMPTS" --url "$RPC_URL"
else
    solana program deploy "$SO_FILE" --keypair "$PAYER_KEYPAIR" --program-id "$KEYPAIR_FILE" --url "$RPC_URL"
fi

DEPLOY_RESULT=$?
if [ $DEPLOY_RESULT -ne 0 ]; then
    echo -e "${RED}❌ Program deployment failed!${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Program deployed successfully!${NC}"

# Step 6: Verify deployment
echo -e "\n${YELLOW}Step 6: Verifying deployment...${NC}"
PROGRAM_INFO=$(solana program show "$PROGRAM_ID" --url "$RPC_URL" 2>/dev/null)
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Program verification successful${NC}"
    echo "$PROGRAM_INFO"
else
    echo -e "${YELLOW}⚠️  Program verification failed (may still be processing)${NC}"
fi

# Step 7: Initialize VM State account
echo -e "\n${YELLOW}Step 7: Initializing FIVE VM state...${NC}"
if ! command -v node >/dev/null 2>&1; then
    echo -e "${RED}❌ Node.js is required to initialize the VM state automatically${NC}"
    exit 1
fi

set +e
VM_INIT_OUTPUT=$(FIVE_PROGRAM_ID="$PROGRAM_ID" node scripts/init-localnet-vm-state.mjs --network "$NETWORK" --rpc-url "$RPC_URL" --program-id "$PROGRAM_ID" 2>&1)
VM_INIT_STATUS=$?
set -e

if [ $VM_INIT_STATUS -ne 0 ]; then
    echo "$VM_INIT_OUTPUT"
    echo -e "${RED}❌ VM state initialization failed${NC}"
    exit 1
fi

echo "$VM_INIT_OUTPUT"
VM_STATE_PDA=$(echo "$VM_INIT_OUTPUT" | awk -F'VM State PDA: ' '/VM State PDA:/ {print $2}' | tail -n 1 | tr -d '[:space:]')
if [[ -n "${VM_STATE_PDA// }" ]]; then
    echo -e "${GREEN}✅ VM state ready: $VM_STATE_PDA${NC}"
else
    echo -e "${YELLOW}⚠️  VM state initialization completed with unexpected output${NC}"
fi

# Step 8: Display summary
echo ""
echo -e "${GREEN}✨ FIVE Program Deployment Complete!${NC}"
echo "========================================="
echo -e "${BLUE}Program ID: $PROGRAM_ID${NC}"
echo -e "${BLUE}Network: $NETWORK${NC}"
echo -e "${BLUE}Binary: $SO_FILE${NC}"
if [ -n "${VM_STATE_PDA:-}" ]; then
    echo -e "${BLUE}VM State PDA: $VM_STATE_PDA${NC}"
fi
echo "PROGRAM_ID=${PROGRAM_ID}"
if [ -n "${VM_STATE_PDA:-}" ]; then
  echo "VM_STATE_PDA=${VM_STATE_PDA}"
fi
echo ""
echo "Next steps:"
echo "1. Start/inspect Surfpool: FIVE_VALIDATOR=surfpool ./five-surfpool/surfpool instance status"
echo "2. Deploy your Five scripts with: five deploy-and-execute script.v --target $NETWORK"
echo "3. Or deploy bytecode only: five deploy script.v --target $NETWORK"
echo "4. Execute on-chain: five execute <script_account> -f 0 --target $NETWORK"
echo ""
echo "Example usage:"
echo "five deploy-and-execute examples/add.v --target $NETWORK"
