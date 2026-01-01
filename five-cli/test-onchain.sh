#!/bin/bash

# Five CLI Onchain Test Script Runner
# Extends the original test-runner.sh with onchain deployment and execution testing
# Tests compile -> deploy -> execute pipeline on Solana networks

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="test-scripts"
TEMP_DIR=".test-temp-onchain"
CLI_BIN="node dist/index.js"
VERBOSE=false
ERRORS_ONLY=false
SAVE_RESULTS=false
CATEGORY_FILTER=""
NETWORK="local"
WASM_ONLY=false
ONCHAIN_ONLY=false
COMPARE_MODE=false
INIT_REGISTRY=false
SKIP_FAILED_WASM=true

# Counters
TOTAL_SCRIPTS=0
COMPILED_SUCCESS=0
COMPILED_FAILED=0
WASM_SUCCESS=0
WASM_FAILED=0
DEPLOYED_SUCCESS=0
DEPLOYED_FAILED=0
ONCHAIN_SUCCESS=0
ONCHAIN_FAILED=0
WASM_ONCHAIN_MATCH=0
WASM_ONCHAIN_DIFFER=0

# Results storage
RESULTS_FILE="test-onchain-results.json"
declare -a FAILED_SCRIPTS=()
declare -a SUCCESS_SCRIPTS=()
declare -a PARTIAL_SUCCESS_SCRIPTS=()
declare -a DEPLOYED_ACCOUNTS=()

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --errors-only|-e)
            ERRORS_ONLY=true
            shift
            ;;
        --save-results|-s)
            SAVE_RESULTS=true
            shift
            ;;
        --category|-c)
            CATEGORY_FILTER="$2"
            shift 2
            ;;
        --network|-n)
            NETWORK="$2"
            shift 2
            ;;
        --wasm-only)
            WASM_ONLY=true
            shift
            ;;
        --onchain-only)
            ONCHAIN_ONLY=true
            shift
            ;;
        --compare)
            COMPARE_MODE=true
            shift
            ;;
        --init)
            INIT_REGISTRY=true
            shift
            ;;
        --no-skip-failed-wasm)
            SKIP_FAILED_WASM=false
            shift
            ;;
        --help|-h)
            echo "Five CLI Onchain Test Runner"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --verbose, -v          Show detailed error messages and outputs"
            echo "  --errors-only, -e      Only show failing scripts"
            echo "  --save-results, -s     Save detailed results to JSON"
            echo "  --category, -c <cat>   Test specific category (e.g., 01-language-basics)"
            echo "  --network, -n <net>    Target network (local, devnet, testnet, mainnet)"
            echo "  --wasm-only            Only run WASM execution tests (skip onchain)"
            echo "  --onchain-only         Only run onchain tests (skip WASM)"
            echo "  --compare              Compare WASM vs onchain execution results"
            echo "  --init                 Initialize Five VM registry before tests"
            echo "  --no-skip-failed-wasm  Don't skip onchain tests for WASM failures"
            echo "  --help, -h             Show this help message"
            echo ""
            echo "Test Pipeline:"
            echo "  1. Compile .v script to .bin bytecode"
            echo "  2. Execute locally with WASM VM (if not --onchain-only)"
            echo "  3. Deploy bytecode to Solana network"
            echo "  4. Execute deployed script onchain"
            echo "  5. Compare results (if --compare mode)"
            echo ""
            echo "Networks:"
            echo "  local     Local Solana test validator (default)"
            echo "  devnet    Solana devnet"
            echo "  testnet   Solana testnet" 
            echo "  mainnet   Solana mainnet (not recommended for testing)"
            echo ""
            echo "Categories:"
            echo "  01-language-basics         Basic language features"
            echo "  02-operators-expressions   Operators and expressions"
            echo "  03-control-flow           Control flow statements"
            echo "  04-account-system         Account definitions and constraints"
            echo "  05-blockchain-integration Blockchain-specific features"
            echo "  06-advanced-features      Advanced language features"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Check if CLI binary exists
if [[ "$CLI_BIN" == node* ]]; then
    JS_FILE=$(echo "$CLI_BIN" | sed 's/node //')
    if [ ! -f "$JS_FILE" ]; then
        echo -e "${RED}Error: Five CLI JavaScript file not found at $JS_FILE${NC}"
        echo "Please run 'npm run build' first"
        exit 1
    fi
else
    if [ ! -f "$CLI_BIN" ]; then
        echo -e "${RED}Error: Five CLI binary not found at $CLI_BIN${NC}"
        echo "Please run 'npm run build' first"
        exit 1
    fi
fi

# Create temp directory
mkdir -p "$TEMP_DIR"

# Initialize results JSON if saving
if [ "$SAVE_RESULTS" = true ]; then
    echo "{" > "$RESULTS_FILE"
    echo "  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"," >> "$RESULTS_FILE"
    echo "  \"network\": \"$NETWORK\"," >> "$RESULTS_FILE"
    echo "  \"test_mode\": \"$([ "$WASM_ONLY" = true ] && echo "wasm-only" || [ "$ONCHAIN_ONLY" = true ] && echo "onchain-only" || [ "$COMPARE_MODE" = true ] && echo "compare" || echo "full")\"," >> "$RESULTS_FILE"
    echo "  \"results\": [" >> "$RESULTS_FILE"
    FIRST_RESULT=true
fi

echo -e "${PURPLE}Five CLI Onchain Test Runner${NC}"
echo "======================================="
echo -e "Network: ${CYAN}$NETWORK${NC}"
echo -e "Mode: ${CYAN}$([ "$WASM_ONLY" = true ] && echo "WASM Only" || [ "$ONCHAIN_ONLY" = true ] && echo "Onchain Only" || [ "$COMPARE_MODE" = true ] && echo "Compare WASM vs Onchain" || echo "Full Pipeline")${NC}"
echo ""

# Initialize registry if requested
if [ "$INIT_REGISTRY" = true ] && [ "$ONCHAIN_ONLY" = false ]; then
    echo -e "${YELLOW}Initializing Five VM registry on $NETWORK...${NC}"
    if ! $CLI_BIN onchain deploy test-scripts/simple.bin --init -n "$NETWORK" >/dev/null 2>&1; then
        echo -e "${RED}Warning: Failed to initialize registry. Continuing anyway...${NC}"
    else
        echo -e "${GREEN}Registry initialized successfully${NC}"
    fi
    echo ""
fi

# Function to extract values from outputs
extract_value() {
    local output="$1"
    local pattern="$2"
    echo "$output" | grep -o "$pattern" | grep -o "[0-9]*" || echo "unknown"
}

extract_result() {
    local output="$1"
    echo "$output" | grep -o '"result": [^,}]*' | cut -d' ' -f2 | tr -d ',' || echo "null"
}

# Function to test a single script
test_script() {
    local script_file="$1"
    local script_name=$(basename "$script_file" .v)
    local script_dir=$(dirname "$script_file")
    local temp_bin="$script_dir/${script_name}.bin"
    local compile_success=false
    local wasm_success=false
    local deploy_success=false
    local onchain_success=false
    local compile_output=""
    local wasm_output=""
    local deploy_output=""
    local onchain_output=""
    local compile_error=""
    local wasm_error=""
    local deploy_error=""
    local onchain_error=""
    local script_account=""
    local wasm_result=""
    local onchain_result=""
    local results_match=false
    
    TOTAL_SCRIPTS=$((TOTAL_SCRIPTS + 1))
    
    if [ "$ERRORS_ONLY" = false ]; then
        echo -e "${PURPLE}Testing: ${YELLOW}${script_name}.v${NC}"
    fi
    
    # Step 1: Compilation (output will be created in same directory as source)
    if compile_output=$($CLI_BIN compile "$script_file" 2>&1); then
        compile_success=true
        COMPILED_SUCCESS=$((COMPILED_SUCCESS + 1))
        
        bytecode_size=$(extract_value "$compile_output" "Bytecode size: [0-9]* bytes")
        
        if [ "$ERRORS_ONLY" = false ]; then
            echo -e "  ${GREEN}✓ Compilation${NC} (${bytecode_size} bytes)"
        fi
    else
        compile_error="$compile_output"
        COMPILED_FAILED=$((COMPILED_FAILED + 1))
        echo -e "  ${RED}✗ Compilation failed${NC}"
        if [ "$VERBOSE" = true ]; then
            echo -e "    ${RED}Error:${NC} $compile_error"
        fi
        FAILED_SCRIPTS+=("$script_name")
        
        # Save results and skip remaining tests
        save_test_result "$script_name" "$compile_success" "$compile_output" "$compile_error" \
                        "$wasm_success" "$wasm_output" "$wasm_error" \
                        "$deploy_success" "$deploy_output" "$deploy_error" "$script_account" \
                        "$onchain_success" "$onchain_output" "$onchain_error" \
                        "$wasm_result" "$onchain_result" "$results_match"
        return
    fi
    
    # Step 2: WASM Execution (if not onchain-only)
    if [ "$ONCHAIN_ONLY" = false ]; then
        if wasm_output=$($CLI_BIN execute "$temp_bin" --format json 2>&1); then
            wasm_success=true
            WASM_SUCCESS=$((WASM_SUCCESS + 1))
            
            wasm_result=$(extract_result "$wasm_output")
            compute_units=$(extract_value "$wasm_output" '"computeUnitsUsed": [0-9]*')
            vm_status=$(echo "$wasm_output" | grep -o '"status": "[^"]*"' | cut -d'"' -f4 || echo "unknown")
            
            if [ "$ERRORS_ONLY" = false ]; then
                echo -e "  ${GREEN}✓ WASM execution${NC} (result: $wasm_result, ${compute_units} CU, ${vm_status})"
            fi
        else
            wasm_error="$wasm_output"
            WASM_FAILED=$((WASM_FAILED + 1))
            echo -e "  ${RED}✗ WASM execution failed${NC}"
            if [ "$VERBOSE" = true ]; then
                echo -e "    ${RED}Error:${NC} $wasm_error"
            fi
            
            # Skip onchain tests if WASM fails and skip flag is set
            if [ "$SKIP_FAILED_WASM" = true ]; then
                if [ "$ERRORS_ONLY" = false ]; then
                    echo -e "  ${YELLOW}⚠ Skipping onchain tests (WASM failed)${NC}"
                fi
                PARTIAL_SUCCESS_SCRIPTS+=("$script_name")
                save_test_result "$script_name" "$compile_success" "$compile_output" "$compile_error" \
                                "$wasm_success" "$wasm_output" "$wasm_error" \
                                "$deploy_success" "$deploy_output" "$deploy_error" "$script_account" \
                                "$onchain_success" "$onchain_output" "$onchain_error" \
                                "$wasm_result" "$onchain_result" "$results_match"
                return
            fi
        fi
    fi
    
    # Step 3: Onchain Deployment (if not wasm-only)
    if [ "$WASM_ONLY" = false ]; then
        local deploy_cmd="$CLI_BIN onchain deploy $temp_bin -n $NETWORK --format json"
        if deploy_output=$($deploy_cmd 2>&1); then
            deploy_success=true
            DEPLOYED_SUCCESS=$((DEPLOYED_SUCCESS + 1))
            
            script_account=$(echo "$deploy_output" | grep -o '"scriptAccount": "[^"]*"' | cut -d'"' -f4 || echo "unknown")
            deployment_cost=$(echo "$deploy_output" | grep -o '"deploymentCost": [0-9]*' | cut -d':' -f2 | tr -d ' ' || echo "unknown")
            
            if [ "$ERRORS_ONLY" = false ]; then
                echo -e "  ${GREEN}✓ Deployment${NC} (account: ${script_account:0:8}..., cost: $deployment_cost lamports)"
            fi
            
            DEPLOYED_ACCOUNTS+=("$script_account")
        else
            deploy_error="$deploy_output"
            DEPLOYED_FAILED=$((DEPLOYED_FAILED + 1))
            echo -e "  ${RED}✗ Deployment failed${NC}"
            if [ "$VERBOSE" = true ]; then
                echo -e "    ${RED}Error:${NC} $deploy_error"
            fi
            
            PARTIAL_SUCCESS_SCRIPTS+=("$script_name")
            save_test_result "$script_name" "$compile_success" "$compile_output" "$compile_error" \
                            "$wasm_success" "$wasm_output" "$wasm_error" \
                            "$deploy_success" "$deploy_output" "$deploy_error" "$script_account" \
                            "$onchain_success" "$onchain_output" "$onchain_error" \
                            "$wasm_result" "$onchain_result" "$results_match"
            return
        fi
        
        # Step 4: Onchain Execution
        if [ -n "$script_account" ] && [ "$script_account" != "unknown" ]; then
            local execute_cmd="$CLI_BIN onchain execute $script_account -n $NETWORK --format json"
            if onchain_output=$($execute_cmd 2>&1); then
                onchain_success=true
                ONCHAIN_SUCCESS=$((ONCHAIN_SUCCESS + 1))
                
                onchain_compute_units=$(echo "$onchain_output" | grep -o '"Program [^"]* consumed [0-9]* of' | grep -o '[0-9]*' || echo "unknown")
                transaction_id=$(echo "$onchain_output" | grep -o '"transactionId": "[^"]*"' | cut -d'"' -f4 || echo "unknown")
                
                if [ "$ERRORS_ONLY" = false ]; then
                    echo -e "  ${GREEN}✓ Onchain execution${NC} (tx: ${transaction_id:0:8}..., ${onchain_compute_units} CU)"
                fi
                
                # Compare results if in compare mode
                if [ "$COMPARE_MODE" = true ] && [ "$wasm_success" = true ] && [ "$onchain_success" = true ]; then
                    if [ "$wasm_result" = "$onchain_result" ]; then
                        results_match=true
                        WASM_ONCHAIN_MATCH=$((WASM_ONCHAIN_MATCH + 1))
                        if [ "$ERRORS_ONLY" = false ]; then
                            echo -e "  ${GREEN}✓ Results match${NC} (WASM: $wasm_result, Onchain: $onchain_result)"
                        fi
                    else
                        WASM_ONCHAIN_DIFFER=$((WASM_ONCHAIN_DIFFER + 1))
                        echo -e "  ${YELLOW}⚠ Results differ${NC} (WASM: $wasm_result, Onchain: $onchain_result)"
                    fi
                fi
                
                SUCCESS_SCRIPTS+=("$script_name")
            else
                onchain_error="$onchain_output"
                ONCHAIN_FAILED=$((ONCHAIN_FAILED + 1))
                echo -e "  ${RED}✗ Onchain execution failed${NC}"
                if [ "$VERBOSE" = true ]; then
                    echo -e "    ${RED}Error:${NC} $onchain_error"
                fi
                
                PARTIAL_SUCCESS_SCRIPTS+=("$script_name")
            fi
        fi
    fi
    
    # Save results
    save_test_result "$script_name" "$compile_success" "$compile_output" "$compile_error" \
                    "$wasm_success" "$wasm_output" "$wasm_error" \
                    "$deploy_success" "$deploy_output" "$deploy_error" "$script_account" \
                    "$onchain_success" "$onchain_output" "$onchain_error" \
                    "$wasm_result" "$onchain_result" "$results_match"
    
    # Clean up temp files
    rm -f "$temp_bin"
    
    if [ "$ERRORS_ONLY" = false ]; then
        echo ""
    fi
}

# Function to save test results to JSON
save_test_result() {
    local script_name="$1"
    local compile_success="$2"
    local compile_output="$3"
    local compile_error="$4"
    local wasm_success="$5"
    local wasm_output="$6" 
    local wasm_error="$7"
    local deploy_success="$8"
    local deploy_output="$9"
    local deploy_error="${10}"
    local script_account="${11}"
    local onchain_success="${12}"
    local onchain_output="${13}"
    local onchain_error="${14}"
    local wasm_result="${15}"
    local onchain_result="${16}"
    local results_match="${17}"
    
    if [ "$SAVE_RESULTS" = true ]; then
        if [ "$FIRST_RESULT" = false ]; then
            echo "    }," >> "$RESULTS_FILE"
        fi
        echo "    {" >> "$RESULTS_FILE"
        echo "      \"script\": \"$script_name\"," >> "$RESULTS_FILE"
        echo "      \"compilation\": {" >> "$RESULTS_FILE"
        echo "        \"success\": $compile_success," >> "$RESULTS_FILE"
        echo "        \"output\": \"$(echo "$compile_output" | sed 's/"/\\"/g')\"," >> "$RESULTS_FILE"
        echo "        \"error\": \"$(echo "$compile_error" | sed 's/"/\\"/g')\"" >> "$RESULTS_FILE"
        echo "      }," >> "$RESULTS_FILE"
        echo "      \"wasm_execution\": {" >> "$RESULTS_FILE"
        echo "        \"success\": $wasm_success," >> "$RESULTS_FILE"
        echo "        \"output\": \"$(echo "$wasm_output" | sed 's/"/\\"/g')\"," >> "$RESULTS_FILE"
        echo "        \"error\": \"$(echo "$wasm_error" | sed 's/"/\\"/g')\"," >> "$RESULTS_FILE"
        echo "        \"result\": \"$wasm_result\"" >> "$RESULTS_FILE"
        echo "      }," >> "$RESULTS_FILE"
        echo "      \"deployment\": {" >> "$RESULTS_FILE"
        echo "        \"success\": $deploy_success," >> "$RESULTS_FILE"
        echo "        \"output\": \"$(echo "$deploy_output" | sed 's/"/\\"/g')\"," >> "$RESULTS_FILE"
        echo "        \"error\": \"$(echo "$deploy_error" | sed 's/"/\\"/g')\"," >> "$RESULTS_FILE"
        echo "        \"script_account\": \"$script_account\"" >> "$RESULTS_FILE"
        echo "      }," >> "$RESULTS_FILE"
        echo "      \"onchain_execution\": {" >> "$RESULTS_FILE"
        echo "        \"success\": $onchain_success," >> "$RESULTS_FILE"
        echo "        \"output\": \"$(echo "$onchain_output" | sed 's/"/\\"/g')\"," >> "$RESULTS_FILE"
        echo "        \"error\": \"$(echo "$onchain_error" | sed 's/"/\\"/g')\"," >> "$RESULTS_FILE"
        echo "        \"result\": \"$onchain_result\"" >> "$RESULTS_FILE"
        echo "      }," >> "$RESULTS_FILE"
        echo "      \"comparison\": {" >> "$RESULTS_FILE"
        echo "        \"results_match\": $results_match" >> "$RESULTS_FILE"
        echo "      }" >> "$RESULTS_FILE"
        FIRST_RESULT=false
    fi
}

# Test all .v scripts
if [ -n "$CATEGORY_FILTER" ]; then
    echo "Running tests on category: $CATEGORY_FILTER"
    SEARCH_PATH="$SCRIPT_DIR/$CATEGORY_FILTER"
else
    echo "Running tests on all .v files in $SCRIPT_DIR/"
    SEARCH_PATH="$SCRIPT_DIR"
fi
echo ""

# Find all .v files in the search path (including subdirectories)
while IFS= read -r -d '' script_file; do
    if [ -f "$script_file" ]; then
        test_script "$script_file"
    fi
done < <(find "$SEARCH_PATH" -name "*.v" -type f -print0 | sort -z)

# Finalize JSON results
if [ "$SAVE_RESULTS" = true ]; then
    echo "    }" >> "$RESULTS_FILE"
    echo "  ]," >> "$RESULTS_FILE"
    echo "  \"summary\": {" >> "$RESULTS_FILE"
    echo "    \"total\": $TOTAL_SCRIPTS," >> "$RESULTS_FILE"
    echo "    \"compiled_success\": $COMPILED_SUCCESS," >> "$RESULTS_FILE"
    echo "    \"compiled_failed\": $COMPILED_FAILED," >> "$RESULTS_FILE"
    echo "    \"wasm_success\": $WASM_SUCCESS," >> "$RESULTS_FILE"
    echo "    \"wasm_failed\": $WASM_FAILED," >> "$RESULTS_FILE"
    echo "    \"deployed_success\": $DEPLOYED_SUCCESS," >> "$RESULTS_FILE"
    echo "    \"deployed_failed\": $DEPLOYED_FAILED," >> "$RESULTS_FILE"
    echo "    \"onchain_success\": $ONCHAIN_SUCCESS," >> "$RESULTS_FILE"
    echo "    \"onchain_failed\": $ONCHAIN_FAILED," >> "$RESULTS_FILE"
    echo "    \"wasm_onchain_match\": $WASM_ONCHAIN_MATCH," >> "$RESULTS_FILE" 
    echo "    \"wasm_onchain_differ\": $WASM_ONCHAIN_DIFFER," >> "$RESULTS_FILE"
    echo "    \"deployed_accounts\": [" >> "$RESULTS_FILE"
    for i in "${!DEPLOYED_ACCOUNTS[@]}"; do
        echo "      \"${DEPLOYED_ACCOUNTS[$i]}\"$([ $i -lt $((${#DEPLOYED_ACCOUNTS[@]} - 1)) ] && echo ",")" >> "$RESULTS_FILE"
    done
    echo "    ]" >> "$RESULTS_FILE"
    echo "  }" >> "$RESULTS_FILE"
    echo "}" >> "$RESULTS_FILE"
fi

# Print comprehensive summary
echo "======================================="
echo -e "${PURPLE}Comprehensive Test Summary${NC}"
echo "======================================="
echo -e "Total scripts tested: ${YELLOW}$TOTAL_SCRIPTS${NC}"
echo ""

echo -e "${BLUE}Compilation Results:${NC}"
echo -e "  ${GREEN}✓ $COMPILED_SUCCESS scripts compiled successfully${NC}"
echo -e "  ${RED}✗ $COMPILED_FAILED scripts failed compilation${NC}"
echo ""

if [ "$ONCHAIN_ONLY" = false ]; then
    echo -e "${BLUE}WASM Execution Results:${NC}"
    echo -e "  ${GREEN}✓ $WASM_SUCCESS scripts executed successfully${NC}"
    echo -e "  ${RED}✗ $WASM_FAILED scripts failed execution${NC}"
    echo ""
fi

if [ "$WASM_ONLY" = false ]; then
    echo -e "${BLUE}Onchain Deployment Results:${NC}"
    echo -e "  ${GREEN}✓ $DEPLOYED_SUCCESS scripts deployed successfully${NC}"
    echo -e "  ${RED}✗ $DEPLOYED_FAILED scripts failed deployment${NC}"
    echo ""
    
    echo -e "${BLUE}Onchain Execution Results:${NC}"
    echo -e "  ${GREEN}✓ $ONCHAIN_SUCCESS scripts executed successfully${NC}"
    echo -e "  ${RED}✗ $ONCHAIN_FAILED scripts failed execution${NC}"
    echo ""
fi

if [ "$COMPARE_MODE" = true ]; then
    echo -e "${BLUE}WASM vs Onchain Comparison:${NC}"
    echo -e "  ${GREEN}✓ $WASM_ONCHAIN_MATCH scripts had matching results${NC}"
    echo -e "  ${YELLOW}⚠ $WASM_ONCHAIN_DIFFER scripts had different results${NC}"
    echo ""
fi

# Calculate success rates
if [ $TOTAL_SCRIPTS -gt 0 ]; then
    compile_rate=$(echo "scale=1; $COMPILED_SUCCESS * 100 / $TOTAL_SCRIPTS" | bc)
    echo -e "${CYAN}Success Rates:${NC}"
    echo -e "  Compilation: ${compile_rate}% ($COMPILED_SUCCESS/$TOTAL_SCRIPTS)"
    
    if [ "$ONCHAIN_ONLY" = false ] && [ $COMPILED_SUCCESS -gt 0 ]; then
        wasm_rate=$(echo "scale=1; $WASM_SUCCESS * 100 / $COMPILED_SUCCESS" | bc)
        echo -e "  WASM Execution: ${wasm_rate}% ($WASM_SUCCESS/$COMPILED_SUCCESS)"
    fi
    
    if [ "$WASM_ONLY" = false ] && [ $COMPILED_SUCCESS -gt 0 ]; then
        deploy_rate=$(echo "scale=1; $DEPLOYED_SUCCESS * 100 / $COMPILED_SUCCESS" | bc)
        onchain_rate=$(echo "scale=1; $ONCHAIN_SUCCESS * 100 / $DEPLOYED_SUCCESS" | bc 2>/dev/null || echo "0")
        echo -e "  Deployment: ${deploy_rate}% ($DEPLOYED_SUCCESS/$COMPILED_SUCCESS)" 
        echo -e "  Onchain Execution: ${onchain_rate}% ($ONCHAIN_SUCCESS/$DEPLOYED_SUCCESS)"
    fi
    
    if [ "$COMPARE_MODE" = true ] && [ $WASM_SUCCESS -gt 0 ] && [ $ONCHAIN_SUCCESS -gt 0 ]; then
        match_total=$((WASM_ONCHAIN_MATCH + WASM_ONCHAIN_DIFFER))
        if [ $match_total -gt 0 ]; then
            match_rate=$(echo "scale=1; $WASM_ONCHAIN_MATCH * 100 / $match_total" | bc)
            echo -e "  Result Matching: ${match_rate}% ($WASM_ONCHAIN_MATCH/$match_total)"
        fi
    fi
    echo ""
fi

# Show categorized scripts
if [ ${#FAILED_SCRIPTS[@]} -gt 0 ]; then
    echo -e "${RED}Failed Scripts (compilation):${NC}"
    for script in "${FAILED_SCRIPTS[@]}"; do
        echo -e "  • ${script}.v"
    done
    echo ""
fi

if [ ${#PARTIAL_SUCCESS_SCRIPTS[@]} -gt 0 ]; then
    echo -e "${YELLOW}Partial Success Scripts (compiled but deployment/execution issues):${NC}"
    for script in "${PARTIAL_SUCCESS_SCRIPTS[@]}"; do
        echo -e "  • ${script}.v"
    done
    echo ""
fi

if [ ${#SUCCESS_SCRIPTS[@]} -gt 0 ]; then
    echo -e "${GREEN}Fully Successful Scripts:${NC}"
    for script in "${SUCCESS_SCRIPTS[@]}"; do
        echo -e "  • ${script}.v"
    done
    echo ""
fi

if [ ${#DEPLOYED_ACCOUNTS[@]} -gt 0 ]; then
    echo -e "${CYAN}Deployed Script Accounts on $NETWORK:${NC}"
    for account in "${DEPLOYED_ACCOUNTS[@]}"; do
        echo -e "  • $account"
    done
    echo ""
fi

if [ "$SAVE_RESULTS" = true ]; then
    echo -e "Detailed results saved to: ${BLUE}$RESULTS_FILE${NC}"
    echo ""
fi

# Clean up temp directory
rm -rf "$TEMP_DIR"

# Exit with appropriate code
total_failures=$((COMPILED_FAILED + WASM_FAILED + DEPLOYED_FAILED + ONCHAIN_FAILED))
if [ $total_failures -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed! Five CLI onchain functionality is working perfectly.${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ Some tests had issues. See detailed results above.${NC}"
    exit 1
fi