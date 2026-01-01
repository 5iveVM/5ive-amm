#!/bin/bash

# Five CLI Test Script Runner (Pinned Program ID)
# Copy of test-runner.sh that exports a fixed Five VM Program ID

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Pin to the requested on-chain Five VM program ID
FIVE_VM_PROGRAM_ID="6zr1vTDCG22iuykF6QRqeU64bvPLHLXtZ3JUCjxbPg5J"
export FIVE_VM_PROGRAM_ID

echo -e "${BLUE}Using FIVE_VM_PROGRAM_ID=${YELLOW}${FIVE_VM_PROGRAM_ID}${NC}"

# Defer to the original runner while preserving env
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ORIGINAL_RUNNER="$SCRIPT_DIR/test-runner.sh"

if [ ! -x "$ORIGINAL_RUNNER" ]; then
  echo -e "${RED}Original runner not found or not executable: $ORIGINAL_RUNNER${NC}"
  exit 1
fi

# Forward all args. Example forcing on-chain devnet:
#   five-cli/test-runner-6zr.sh --onchain --network devnet
"$ORIGINAL_RUNNER" "$@"

