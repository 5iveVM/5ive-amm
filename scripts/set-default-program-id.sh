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
#   --target      Optional target cluster (localnet, devnet, mainnet)
#                 If not specified, updates all clusters in constants.vm.toml
#
# Examples:
#   ./scripts/set-default-program-id.sh 8h8gqgMhfq5qmPbs9nNHkXNoy2jb1JywxaRC6W68wGVm
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
VM_CONSTANTS_FILE="${PROJECT_ROOT}/five-solana/constants.vm.toml"

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
    echo "  --target      Optional network: devnet, mainnet, localnet"
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
    if [[ ! "$TARGET" =~ ^(devnet|mainnet|local|localnet)$ ]]; then
      print_error "Invalid target: '$TARGET'"
      print_info "Valid targets: devnet, mainnet, localnet"
      return 1
    fi
  fi

  # Validate program ID format
  if ! validate_pubkey "$PROGRAM_ID"; then
    return 2
  fi

  # Normalize legacy alias
  if [[ "$TARGET" == "local" ]]; then
    TARGET="localnet"
  fi

  # Check that files exist
  if [[ ! -f "$VM_CONSTANTS_FILE" ]]; then
    print_error "File not found: $VM_CONSTANTS_FILE"
    return 3
  fi

  # Check write permissions
  if [[ ! -w "$VM_CONSTANTS_FILE" ]]; then
    print_error "Permission denied: cannot write to $VM_CONSTANTS_FILE"
    return 4
  fi

  print_info "Setting cluster program ID in five-solana/constants.vm.toml..."
  print_info "  Program ID: ${BLUE}${PROGRAM_ID}${NC}"
  if [[ -n "$TARGET" ]]; then
    print_info "  Target:    ${BLUE}${TARGET}${NC}"
  fi

  update_cluster() {
    local cluster="$1"
    if [[ "$OSTYPE" == "darwin"* ]]; then
      sed -i '' -E "/^\\[clusters\\.${cluster}\\]$/,/^\\[/{s/^program_id = \".*\"$/program_id = \"${PROGRAM_ID}\"/;}" "$VM_CONSTANTS_FILE"
    else
      sed -i -E "/^\\[clusters\\.${cluster}\\]$/,/^\\[/{s/^program_id = \".*\"$/program_id = \"${PROGRAM_ID}\"/;}" "$VM_CONSTANTS_FILE"
    fi
    if ! awk -v cluster="$cluster" -v program="$PROGRAM_ID" '
      $0 ~ "^\\[clusters\\." cluster "\\]$" { in_cluster=1; next }
      /^\[/ && in_cluster { exit }
      in_cluster && $0 == "program_id = \"" program "\"" { found=1; exit }
      END { exit(found ? 0 : 1) }
    ' "$VM_CONSTANTS_FILE"; then
      print_error "Failed to set ${cluster} program_id in ${VM_CONSTANTS_FILE}"
      return 1
    fi
    print_success "Updated ${BLUE}${cluster}${NC} program_id in ${BLUE}five-solana/constants.vm.toml${NC}"
  }

  if [[ -n "$TARGET" ]]; then
    update_cluster "$TARGET" || return 4
  else
    update_cluster "localnet" || return 4
    update_cluster "devnet" || return 4
    update_cluster "mainnet" || return 4
  fi

  print_success "Cluster program ID update complete"
  echo ""
  print_info "Next steps:"
  echo "  1. Regenerate constants: ./scripts/build-five-solana-cluster.sh --cluster <cluster>"
  echo "  2. Rebuild SDK if needed: cd five-sdk && npm run build"

  return 0
}

main "$@"
exit $?
