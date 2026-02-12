#!/bin/bash

# Setup script for Five LSP WASM integration
# Builds the LSP WASM bindings and copies them to the frontend public directory
#
# Usage:
#   ./scripts/setup-lsp-wasm.sh              # Full setup
#   ./scripts/setup-lsp-wasm.sh --check      # Just check if WASM files exist
#   ./scripts/setup-lsp-wasm.sh --clean      # Remove generated WASM files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
LSP_DIR="$REPO_ROOT/five-lsp"
FRONTEND_DIR="$REPO_ROOT/five-frontend"
WASM_DEST="$FRONTEND_DIR/public/wasm"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

function log_info() {
  echo -e "${BLUE}[LSP WASM Setup]${NC} $1"
}

function log_success() {
  echo -e "${GREEN}✓${NC} $1"
}

function log_warn() {
  echo -e "${YELLOW}⚠${NC} $1"
}

function log_error() {
  echo -e "${RED}✗${NC} $1"
}

# Check if required tools are installed
function check_dependencies() {
  log_info "Checking dependencies..."

  if ! command -v wasm-pack &> /dev/null; then
    log_error "wasm-pack is not installed"
    echo "Install it with: cargo install wasm-pack"
    return 1
  fi
  log_success "wasm-pack found"

  if ! command -v cargo &> /dev/null; then
    log_error "cargo is not installed"
    return 1
  fi
  log_success "cargo found"
}

# Check if WASM files exist
function check_wasm_files() {
  local missing=0

  if [ ! -f "$WASM_DEST/five_lsp.js" ]; then
    log_warn "Missing: $WASM_DEST/five_lsp.js"
    missing=$((missing + 1))
  else
    log_success "Found: five_lsp.js"
  fi

  if [ ! -f "$WASM_DEST/five_lsp_bg.wasm" ]; then
    log_warn "Missing: $WASM_DEST/five_lsp_bg.wasm"
    missing=$((missing + 1))
  else
    log_success "Found: five_lsp_bg.wasm"
  fi

  if [ ! -f "$WASM_DEST/five_lsp.d.ts" ]; then
    log_warn "Missing: $WASM_DEST/five_lsp.d.ts"
    missing=$((missing + 1))
  else
    log_success "Found: five_lsp.d.ts"
  fi

  if [ ! -f "$WASM_DEST/five_lsp_bg.wasm.d.ts" ]; then
    log_warn "Missing: $WASM_DEST/five_lsp_bg.wasm.d.ts"
    missing=$((missing + 1))
  else
    log_success "Found: five_lsp_bg.wasm.d.ts"
  fi

  return $missing
}

# Build WASM bindings
function build_wasm() {
  log_info "Building LSP WASM bindings..."

  cd "$LSP_DIR"

  # Run wasm-pack
  if ! wasm-pack build --release --target web --out-dir pkg 2>&1 | grep -v "wasm-opt"; then
    log_error "WASM build failed"
    return 1
  fi

  log_success "WASM build completed"
}

# Copy WASM files to frontend
function copy_wasm_files() {
  log_info "Copying WASM files to frontend..."

  # Ensure destination exists
  mkdir -p "$WASM_DEST"

  # Copy all files from pkg to frontend
  local files=("five_lsp.js" "five_lsp.d.ts" "five_lsp_bg.wasm" "five_lsp_bg.wasm.d.ts" ".gitignore")

  for file in "${files[@]}"; do
    if [ -f "$LSP_DIR/pkg/$file" ]; then
      cp "$LSP_DIR/pkg/$file" "$WASM_DEST/$file"
      log_success "Copied: $file"
    else
      log_warn "Source file not found: $LSP_DIR/pkg/$file"
    fi
  done
}

# Generate missing assets
function generate_missing_assets() {
  log_info "Checking for missing assets..."

  if [ ! -f "$FRONTEND_DIR/public/noise.png" ]; then
    log_warn "noise.png not found, generating..."
    if command -v python3 &> /dev/null && [ -f "$FRONTEND_DIR/scripts/generate_noise.py" ]; then
      python3 "$FRONTEND_DIR/scripts/generate_noise.py"
      log_success "Generated noise.png"
    else
      log_warn "Could not generate noise.png - Python 3 not found or script missing"
    fi
  else
    log_success "noise.png exists"
  fi
}

# Main setup flow
function run_setup() {
  log_info "Starting LSP WASM setup..."
  echo ""

  # Check dependencies
  if ! check_dependencies; then
    log_error "Dependency check failed"
    return 1
  fi
  echo ""

  # Build WASM
  if ! build_wasm; then
    log_error "WASM build failed"
    return 1
  fi
  echo ""

  # Copy files
  if ! copy_wasm_files; then
    log_error "Failed to copy WASM files"
    return 1
  fi
  echo ""

  # Generate assets
  generate_missing_assets
  echo ""

  # Verify
  log_info "Verifying WASM installation..."
  if check_wasm_files; then
    log_success "All WASM files verified"
  else
    log_warn "Some WASM files are missing"
  fi
  echo ""

  log_success "LSP WASM setup complete!"
  echo ""
  echo "Next steps:"
  echo "  1. Navigate to five-frontend: cd five-frontend"
  echo "  2. Install dependencies: npm install"
  echo "  3. Start development: npm run dev"
  echo ""
}

# Clean up generated files
function run_clean() {
  log_info "Cleaning up generated WASM files..."

  rm -rf "$LSP_DIR/pkg"
  log_success "Removed: $LSP_DIR/pkg"

  rm -f "$WASM_DEST/five_lsp.js"
  rm -f "$WASM_DEST/five_lsp.d.ts"
  rm -f "$WASM_DEST/five_lsp_bg.wasm"
  rm -f "$WASM_DEST/five_lsp_bg.wasm.d.ts"
  log_success "Removed WASM files from frontend"

  log_success "Cleanup complete"
}

# Handle CLI arguments
MODE="${1:-setup}"

case "$MODE" in
  setup)
    run_setup
    ;;
  check)
    log_info "Checking WASM files..."
    check_wasm_files
    ;;
  clean)
    read -p "Are you sure you want to clean up WASM files? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
      run_clean
    else
      log_info "Cleanup cancelled"
    fi
    ;;
  *)
    echo "Usage: $0 [setup|check|clean]"
    echo ""
    echo "  setup   - Build and install LSP WASM (default)"
    echo "  check   - Verify WASM files are present"
    echo "  clean   - Remove generated WASM files"
    exit 1
    ;;
esac
