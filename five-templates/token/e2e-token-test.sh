#!/bin/bash

##############################################################################
# Token Template E2E Test Runner
#
# Automated script to build, deploy, and test the token template.
# Tests core token functionalities (mint, transfer, approve, burn, freeze).
#
# Usage:
#   ./e2e-token-test.sh [options]
#
# Examples:
#   ./e2e-token-test.sh                    # Build and test only
#   ./e2e-token-test.sh --deploy           # Build, deploy, and test
#   ./e2e-token-test.sh --deploy --verbose # Verbose output
#   ./e2e-token-test.sh --clean            # Clean build artifacts
#
##############################################################################

set -e

##############################################################################
# COLOR DEFINITIONS
##############################################################################

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

##############################################################################
# CONFIGURATION
##############################################################################

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
BUILD_DIR="$PROJECT_ROOT/build"
SOURCE_FILE="$PROJECT_ROOT/src/token.v"
COMPILED_FILE="$BUILD_DIR/five-token-template.five"
TEST_SCRIPT="$PROJECT_ROOT/e2e-token-test.mjs"
REPORT_FILE="$PROJECT_ROOT/test-state-fiveprogram.json"

# Options
VERBOSE=false
DEPLOY=false
CLEAN=false
SKIP_BUILD=false
RPC_URL="${FIVE_RPC_URL:-}"
PROGRAM_ID="${FIVE_PROGRAM_ID:-}"
VM_STATE_PDA="${VM_STATE_PDA:-}"
TOKEN_SCRIPT_ACCOUNT="${FIVE_TOKEN_SCRIPT_ACCOUNT:-${TOKEN_SCRIPT_ACCOUNT:-}}"
SHOW_HELP=false

# Counters & Status
BUILD_SUCCESSFUL=false
TEST_SUCCESSFUL=false
DEPLOYMENT_SUCCESSFUL=false

##############################################################################
# UTILITY FUNCTIONS
##############################################################################

print_header() {
    echo -e "\n${PURPLE}========================================${NC}"
    echo -e "${PURPLE}$1${NC}"
    echo -e "${PURPLE}========================================${NC}\n"
}

print_step() {
    echo -e "${CYAN}> $1${NC}"
}

print_success() {
    echo -e "${GREEN}[PASS] $1${NC}"
}

print_error() {
    echo -e "${RED}[FAIL] $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}[WARN] $1${NC}"
}

print_info() {
    echo -e "${BLUE}[INFO] $1${NC}"
}

print_separator() {
    echo -e "${MAGENTA}────────────────────────────────────────${NC}"
}

show_help() {
    cat << EOF
${PURPLE}Token Template E2E Test Runner${NC}

${CYAN}Usage:${NC}
  $0 [options]

${CYAN}Options:${NC}
  --deploy              Build, deploy to localnet, and test
  --clean               Remove build artifacts and reports
  --skip-build          Skip build step, use existing artifacts
  --verbose, -v         Show detailed output from build and test
  --rpc-url URL         Explicit RPC URL
  --program-id PUBKEY   Explicit Five VM program ID
  --vm-state PUBKEY     Explicit VM state PDA
  --token-script-account PUBKEY
                       Explicit deployed token script account
  --help, -h            Show this help message

${CYAN}Examples:${NC}
  # Build and test with explicit cluster inputs
  FIVE_RPC_URL=http://127.0.0.1:8899 FIVE_PROGRAM_ID=<pubkey> VM_STATE_PDA=<pubkey> FIVE_TOKEN_SCRIPT_ACCOUNT=<pubkey> $0

  # Build, deploy to localnet, and test
  FIVE_RPC_URL=http://127.0.0.1:8899 FIVE_PROGRAM_ID=<pubkey> VM_STATE_PDA=<pubkey> $0 --deploy

  # Clean all artifacts
  $0 --clean

  # Use existing build and run tests
  $0 --skip-build

${CYAN}Test Workflow:${NC}
  1. Check prerequisites (solana-cli, five, node)
  2. Clean old artifacts (optional)
  3. Build token template
  4. Deploy to localnet (optional)
  5. Run E2E test
  6. Verify on-chain state
  7. Display results

${CYAN}Requirements:${NC}
  - Solana CLI (solana --version)
  - Five CLI (five --version)
  - Node.js 18+ (node --version)
  - Running target validator at the explicit RPC URL
  - @solana/web3.js installed (npm install)

${CYAN}Output:${NC}
  - Console: Live test progress with TX IDs
  - JSON Report: test-state-fiveprogram.json (structured results)

EOF
}

##############################################################################
# PREREQUISITE CHECKS
##############################################################################

check_prerequisites() {
    print_header "Checking Prerequisites"

    # Check Five CLI
    # print_step "Checking Five CLI..."
    # if command -v five &> /dev/null; then
    #     FIVE_VERSION=$(five --version 2>/dev/null | head -1 || echo "unknown")
    #     print_success "Five CLI installed: $FIVE_VERSION"
    # else
    #     print_error "Five CLI not found. Install with: cargo install --git https://github.com/five-protocol/five-cli"
    #     exit 1
    # fi

    if [ "$CLEAN" = false ] && [ -z "${RPC_URL:-}" ]; then
        print_error "Missing explicit RPC URL. Set FIVE_RPC_URL or pass --rpc-url."
        exit 1
    fi

    export FIVE_RPC_URL="$RPC_URL"
    export FIVE_PROGRAM_ID="$PROGRAM_ID"
    export VM_STATE_PDA="$VM_STATE_PDA"
    export FIVE_TOKEN_SCRIPT_ACCOUNT="$TOKEN_SCRIPT_ACCOUNT"

    # Check Solana CLI
    print_step "Checking Solana CLI..."
    if command -v solana &> /dev/null; then
        SOLANA_VERSION=$(solana --version 2>/dev/null | head -1)
        print_success "Solana CLI installed: $SOLANA_VERSION"
    else
        print_error "Solana CLI not found. Install from: https://docs.solana.com/cli/install-solana-cli-tools"
        exit 1
    fi

    # Check Node.js
    print_step "Checking Node.js..."
    if command -v node &> /dev/null; then
        NODE_VERSION=$(node --version)
        print_success "Node.js installed: $NODE_VERSION"
    else
        print_error "Node.js not found. Install from: https://nodejs.org/"
        exit 1
    fi

    # Check if @solana/web3.js is installed
    print_step "Checking @solana/web3.js..."
    if [ -d "$PROJECT_ROOT/node_modules/@solana/web3.js" ] || npm ls @solana/web3.js > /dev/null 2>&1; then
        print_success "@solana/web3.js installed"
    else
        print_warning "@solana/web3.js not installed. Installing..."
        npm install @solana/web3.js
    fi

    # Check RPC connection
    if [ "$DEPLOY" = true ]; then
        print_step "Checking RPC connection..."
        if solana cluster-version --url "$RPC_URL" &> /dev/null; then
            SLOT=$(solana slot --url "$RPC_URL" || echo "unknown")
            print_success "Connected to validator (slot: $SLOT)"
        else
            print_warning "Cannot connect to $RPC_URL"
            print_warning "Make sure the target validator is running"
            if [ "$DEPLOY" = true ]; then
                echo ""
                print_error "Cannot deploy without a running validator"
                exit 1
            fi
        fi
    fi
}

##############################################################################
# BUILD FUNCTIONS
##############################################################################

clean_artifacts() {
    print_header "Cleaning Artifacts"

    print_step "Removing build directory..."
    if [ -d "$BUILD_DIR" ]; then
        rm -rf "$BUILD_DIR"
        print_success "Build directory removed"
    else
        print_info "No build directory found"
    fi

    print_step "Removing report files..."
    if [ -f "$REPORT_FILE" ]; then
        rm -f "$REPORT_FILE"
        print_success "Report file removed"
    fi

    print_step "Removing .five cache..."
    if [ -d "$PROJECT_ROOT/.five" ]; then
        rm -rf "$PROJECT_ROOT/.five"
        print_success "Cache removed"
    fi

    print_success "Cleanup complete"
}

build_template() {
    print_header "Building Token Template"

    if [ ! -f "$SOURCE_FILE" ]; then
        print_error "Source file not found: $SOURCE_FILE"
        exit 1
    fi

    print_step "Source: $SOURCE_FILE"
    print_step "Building with local debug_compile..."

    # 1. Compile with local debug_compile to get .bin and .abi.json
    # Assuming five-dsl-compiler is in a sibling directory standard structure
    COMPILER_DIR="$PROJECT_ROOT/../../five-dsl-compiler"
    
    if [ ! -d "$COMPILER_DIR" ]; then
         print_error "Compiler directory not found at $COMPILER_DIR"
         exit 1
    fi

    cd "$COMPILER_DIR"
    if cargo run -q --bin debug_compile -- "$PROJECT_ROOT/src/token.v"; then
        print_success "Compilation successful"
        
        # 2. Create the .five artifact
        cd "$PROJECT_ROOT"
        if node create-artifact.js; then
            print_success "Artifact created"
            BUILD_SUCCESSFUL=true
            
            # Show summary
            BYTECODE_SIZE=$(ls -lh "$COMPILED_FILE" 2>/dev/null | awk '{print $5}' || echo "0")
            print_info "Artifact: $COMPILED_FILE ($BYTECODE_SIZE)"
        else
            print_error "Artifact creation failed"
            exit 1
        fi
    else
        print_error "Compilation failed"
        exit 1
    fi
    
    cd "$PROJECT_ROOT"
}

##############################################################################
# DEPLOYMENT FUNCTIONS
##############################################################################

deploy_to_localnet() {
    print_header "Deploying to Localnet"
    DEPLOY_OUTPUT_FILE="/tmp/deploy_token_out.txt"

    if [ ! -f "$COMPILED_FILE" ]; then
        print_error "Compiled file not found: $COMPILED_FILE"
        exit 1
    fi

    print_step "Deploying $COMPILED_FILE..."

    if [ "$VERBOSE" = true ]; then
        if node deploy-to-five-vm.mjs | tee "${DEPLOY_OUTPUT_FILE}"; then
            DEPLOYMENT_SUCCESSFUL=true
            print_success "Deployment successful"
        else
            print_error "Deployment failed"
            exit 1
        fi
    else
        # Capture output but show minimal success
        if node deploy-to-five-vm.mjs > "${DEPLOY_OUTPUT_FILE}" 2>&1; then
            DEPLOYMENT_SUCCESSFUL=true
            print_success "Deployment successful"
            print_info "Deployment output captured in ${DEPLOY_OUTPUT_FILE}"
        else
            print_error "Deployment failed"
            cat "${DEPLOY_OUTPUT_FILE}"
            exit 1
        fi
    fi
    if [ -f "${DEPLOY_OUTPUT_FILE}" ]; then
        DEPLOYED_SCRIPT=$(grep -E '^tokenScriptAccount=' "${DEPLOY_OUTPUT_FILE}" | tail -1 | cut -d= -f2- || true)
        if [ -n "${DEPLOYED_SCRIPT}" ]; then
            TOKEN_SCRIPT_ACCOUNT="${DEPLOYED_SCRIPT}"
            export FIVE_TOKEN_SCRIPT_ACCOUNT="${TOKEN_SCRIPT_ACCOUNT}"
            print_info "Using deployed token script account: ${TOKEN_SCRIPT_ACCOUNT}"
        else
            print_error "Deployment completed but did not return tokenScriptAccount"
            exit 1
        fi
    fi
}

##############################################################################
# TEST FUNCTIONS
##############################################################################

run_e2e_test() {
    print_header "Running E2E Tests"

    if [ ! -f "$TEST_SCRIPT" ]; then
        print_error "Test script not found: $TEST_SCRIPT"
        exit 1
    fi

    print_step "Running: $TEST_SCRIPT"
    print_info "RPC URL: $RPC_URL"
    print_info "Program ID: $PROGRAM_ID"
    print_info "VM State: $VM_STATE_PDA"
    print_info "Token Script: $TOKEN_SCRIPT_ACCOUNT"
    print_separator

    if [ "$VERBOSE" = true ]; then
        if node "$TEST_SCRIPT"; then
            TEST_SUCCESSFUL=true
        else
            print_error "Tests failed"
            exit 1
        fi
    else
        if node "$TEST_SCRIPT" 2>&1 | tee /tmp/e2e-token-test.log; then
            TEST_SUCCESSFUL=true
        else
            print_error "Tests failed"
            exit 1
        fi
    fi
}

##############################################################################
# VERIFICATION
##############################################################################

run_verification() {
    print_header "Running On-Chain Verification"

    if [ -f "$PROJECT_ROOT/verify-on-chain.mjs" ]; then
        if node "$PROJECT_ROOT/verify-on-chain.mjs"; then
            print_success "On-Chain Verification Passed"
        else
            print_warning "On-Chain Verification Failed (non-fatal)"
        fi
    else
        print_info "verify-on-chain.mjs not found, skipping verification"
    fi
}

##############################################################################
# REPORT FUNCTIONS
##############################################################################

show_test_report() {
    print_header "Test Report"

    if [ -f "$REPORT_FILE" ]; then
        print_success "Report saved: $REPORT_FILE"
        echo ""

        # Parse and display summary if jq is available
        if command -v jq &> /dev/null; then
            print_step "Summary:"
            echo ""

            # The structure of test-state-fiveprogram.json in token template is diverse
            # We look for results block
            INIT_MINT=$(jq '.results.initMint' "$REPORT_FILE" 2>/dev/null || echo "?")
            TRANSFER=$(jq '.results.transfer' "$REPORT_FILE" 2>/dev/null || echo "?")
            BURN=$(jq '.results.burn' "$REPORT_FILE" 2>/dev/null || echo "?")
            
            echo "  Init Mint:                $INIT_MINT"
            echo "  Transfer:                 $TRANSFER"
            echo "  Burn:                     $BURN"
            echo ""
        else
            print_info "Install jq for better report parsing: brew install jq"
            # Just cat the file if no jq, but limit lines
            cat "$REPORT_FILE" | grep "results" -A 10
        fi
    else
        print_warning "Report file not found: $REPORT_FILE"
    fi
}

##############################################################################
# SUMMARY
##############################################################################

show_summary() {
    print_header "Summary"

    echo "Status:"
    [ "$BUILD_SUCCESSFUL" = true ] && print_success "Build" || print_error "Build"
    [ "$DEPLOYMENT_SUCCESSFUL" = true ] && print_success "Deployment" || print_info "Deployment (skipped)"
    [ "$TEST_SUCCESSFUL" = true ] && print_success "Tests" || print_error "Tests"

    echo ""
    echo "Artifacts:"
    [ -f "$COMPILED_FILE" ] && print_info "Bytecode: $COMPILED_FILE" || echo "  (not found)"
    [ -f "$REPORT_FILE" ] && print_info "Report: $REPORT_FILE" || echo "  (not found)"

    echo ""
    if [ "$TEST_SUCCESSFUL" = true ]; then
        print_success "All tests completed successfully!"
        return 0
    else
        print_error "Some tests failed"
        return 1
    fi
}

##############################################################################
# MAIN
##############################################################################

main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --deploy)
                DEPLOY=true
                shift
                ;;
            --clean)
                CLEAN=true
                shift
                ;;
            --skip-build)
                SKIP_BUILD=true
                shift
                ;;
            --verbose|-v)
                VERBOSE=true
                shift
                ;;
            --rpc-url)
                RPC_URL="$2"
                shift 2
                ;;
            --program-id)
                PROGRAM_ID="$2"
                shift 2
                ;;
            --vm-state)
                VM_STATE_PDA="$2"
                shift 2
                ;;
            --token-script-account)
                TOKEN_SCRIPT_ACCOUNT="$2"
                shift 2
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                echo ""
                show_help
                exit 1
                ;;
        esac
    done

    # Print intro
    print_header "Token Template E2E Test Runner"

    echo "Configuration:"
    echo "  Project Root:   $PROJECT_ROOT"
    echo "  Source:         $SOURCE_FILE"
    echo "  Build Output:   $COMPILED_FILE"
    echo "  RPC URL:        ${RPC_URL:-<unset>}"
    echo "  Program ID:     ${PROGRAM_ID:-<unset>}"
    echo "  VM State:       ${VM_STATE_PDA:-<unset>}"
    echo "  Token Script:   ${TOKEN_SCRIPT_ACCOUNT:-<unset>}"
    [ "$VERBOSE" = true ] && echo "  Verbose:        true"
    [ "$DEPLOY" = true ] && echo "  Deploy:         true"
    [ "$SKIP_BUILD" = true ] && echo "  Skip Build:     true"
    echo ""

    # Execute pipeline
    check_prerequisites

    if [ "$CLEAN" = true ]; then
        clean_artifacts
        exit 0
    fi

    if [ "$SKIP_BUILD" = false ]; then
        build_template
    else
        if [ -f "$COMPILED_FILE" ]; then
            print_success "Using existing build: $COMPILED_FILE"
        else
            print_error "Build artifacts not found. Remove --skip-build to build"
            exit 1
        fi
    fi

    if [ "$DEPLOY" = true ]; then
        if [ -z "${PROGRAM_ID:-}" ] || [ -z "${VM_STATE_PDA:-}" ]; then
            print_error "Deploy requires explicit FIVE_PROGRAM_ID and VM_STATE_PDA (or --program-id/--vm-state)."
            exit 1
        fi
        deploy_to_localnet
    elif [ -z "${TOKEN_SCRIPT_ACCOUNT:-}" ]; then
        print_error "Test requires explicit FIVE_TOKEN_SCRIPT_ACCOUNT (or --token-script-account) when not deploying."
        exit 1
    fi

    run_e2e_test
    run_verification
    show_test_report
    echo ""
    show_summary
}

# Run main function
main "$@"
