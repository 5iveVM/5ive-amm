#!/bin/bash
##
# Set default program ID for Five SDK at npm publish time.
#
# This script injects a baked program ID into the compiled SDK, allowing
# released packages to have a default program ID without environment variables.
#
# Usage:
#   ./scripts/set-default-program-id.sh <program-id> [--target <target>]
#
# Arguments:
#   program-id    Solana public key (base58 encoded, 32-44 characters)
#   --target      Optional target network (devnet, testnet, mainnet, local)
#                 If not specified, applies to all targets
#
# Examples:
#   ./scripts/set-default-program-id.sh HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg
#   ./scripts/set-default-program-id.sh HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg --target devnet
#
# Error Codes:
#   0   Success
#   1   Missing program ID argument
#   2   Invalid Solana pubkey format
#   3   File not found
#   4   Write permission denied
#

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Files to update
SDK_RESOLVER_FILE="${PROJECT_ROOT}/five-sdk/src/config/ProgramIdResolver.ts"

#
# Print colored output
#
print_info() {
  echo -e "${BLUE}ℹ${NC} $*"
}

print_success() {
  echo -e "${GREEN}✓${NC} $*"
}

print_error() {
  echo -e "${RED}✗${NC} $*" >&2
}

print_warning() {
  echo -e "${YELLOW}⚠${NC} $*"
}

#
# Validate Solana pubkey format
# Accepts base58 encoded strings, 32-44 characters
#
validate_pubkey() {
  local pubkey="$1"

  # Check length (base58 encoded Solana addresses are 32-44 chars)
  # Solana uses standard base58 alphabet: 123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz
  if [[ ! "$pubkey" =~ ^[123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz]{32,44}$ ]]; then
    print_error "Invalid Solana pubkey format: '$pubkey'"
    print_info "Expected base58 encoded address (32-44 characters, standard alphabet)"
    return 2
  fi

  return 0
}

#
# Main script
#
main() {
  # Parse arguments
  if [[ $# -lt 1 ]]; then
    print_error "Missing program ID argument"
    echo ""
    echo "Usage: $0 <program-id> [--target <target>]"
    echo ""
    echo "Arguments:"
    echo "  program-id    Solana public key (base58 encoded)"
    echo "  --target      Optional network: devnet, testnet, mainnet, local"
    echo ""
    echo "Examples:"
    echo "  $0 HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg"
    echo "  $0 HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg --target devnet"
    return 1
  fi

  local PROGRAM_ID="$1"
  local TARGET=""

  # Parse optional --target flag
  if [[ $# -ge 3 ]] && [[ "$2" == "--target" ]]; then
    TARGET="$3"
    # Validate target
    if [[ ! "$TARGET" =~ ^(devnet|testnet|mainnet|local|wasm)$ ]]; then
      print_error "Invalid target: '$TARGET'"
      print_info "Valid targets: devnet, testnet, mainnet, local, wasm"
      return 1
    fi
  fi

  # Validate program ID format
  if ! validate_pubkey "$PROGRAM_ID"; then
    return 2
  fi

  # Check that files exist
  if [[ ! -f "$SDK_RESOLVER_FILE" ]]; then
    print_error "File not found: $SDK_RESOLVER_FILE"
    return 3
  fi

  # Check write permissions
  if [[ ! -w "$SDK_RESOLVER_FILE" ]]; then
    print_error "Permission denied: cannot write to $SDK_RESOLVER_FILE"
    return 4
  fi

  print_info "Setting default program ID in Five SDK..."
  print_info "  Program ID: ${BLUE}${PROGRAM_ID}${NC}"
  if [[ -n "$TARGET" ]]; then
    print_info "  Target:    ${BLUE}${TARGET}${NC}"
  fi

  # Update SDK resolver constant
  # Match: export const FIVE_BAKED_PROGRAM_ID = '';
  # Replace with: export const FIVE_BAKED_PROGRAM_ID = 'program-id';
  if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS requires -i ''
    sed -i '' "s|export const FIVE_BAKED_PROGRAM_ID = '';|export const FIVE_BAKED_PROGRAM_ID = '${PROGRAM_ID}';|" "$SDK_RESOLVER_FILE"
  else
    # Linux sed
    sed -i "s|export const FIVE_BAKED_PROGRAM_ID = '';|export const FIVE_BAKED_PROGRAM_ID = '${PROGRAM_ID}';|" "$SDK_RESOLVER_FILE"
  fi

  # Verify the change was made
  if grep -q "export const FIVE_BAKED_PROGRAM_ID = '${PROGRAM_ID}';" "$SDK_RESOLVER_FILE"; then
    print_success "Updated ${BLUE}five-sdk/src/config/ProgramIdResolver.ts${NC}"
  else
    print_error "Failed to update ProgramIdResolver.ts"
    return 4
  fi

  print_success "Default program ID set successfully"
  echo ""
  print_info "Resolution precedence (in order):"
  echo "  1. Explicit call parameter"
  echo "  2. SDK default: ${BLUE}FiveSDK.setDefaultProgramId()${NC}"
  echo "  3. Environment variable: ${BLUE}FIVE_PROGRAM_ID${NC}"
  echo "  4. Baked default (just set): ${BLUE}${PROGRAM_ID}${NC}"
  echo ""
  print_info "Next steps:"
  echo "  1. Rebuild SDK: cd five-sdk && npm run build"
  echo "  2. Publish package: npm publish"

  return 0
}

main "$@"
exit $?
