#!/bin/bash

# Five CLI Opcode Analysis Runner
# Analyzes all .v scripts to show comprehensive opcode usage across the entire test suite

set -e

# Check for bash 4.0+ (required for associative arrays)
if ((BASH_VERSINFO[0] < 4)); then
    echo "This script requires bash 4.0 or later (current: $BASH_VERSION)"
    echo "On macOS, install with: brew install bash"
    exit 1
fi

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
TEMP_DIR=".opcode-analysis-temp"
CLI_BIN="node dist/index.js"
RESULTS_FILE="opcode-analysis-results.json"

# Counters and data structures
TOTAL_SCRIPTS=0
ANALYZED_SUCCESS=0
ANALYZED_FAILED=0
declare -A OPCODE_USAGE_COUNT
declare -A OPCODE_SCRIPT_COUNT
declare -A ALL_OPCODES
declare -a ANALYZED_SCRIPTS=()
declare -a FAILED_SCRIPTS=()

# Create temp directory
mkdir -p "$TEMP_DIR"

echo -e "${BLUE}Five CLI Comprehensive Opcode Analysis Runner${NC}"
echo "=============================================="
echo ""

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

# Function to analyze a single script
analyze_script() {
    local script_file="$1"
    local script_name=$(basename "$script_file" .v)
    local temp_bin="$TEMP_DIR/${script_name}.bin"
    
    TOTAL_SCRIPTS=$((TOTAL_SCRIPTS + 1))
    
    echo -e "Analyzing: ${YELLOW}${script_name}.v${NC}"
    
    # Compile with analysis
    if analysis_output=$($CLI_BIN compile "$script_file" --output "$temp_bin" --analyze 2>&1); then
        ANALYZED_SUCCESS=$((ANALYZED_SUCCESS + 1))
        ANALYZED_SCRIPTS+=("$script_name")
        
        # Extract bytecode info
        bytecode_size=$(echo "$analysis_output" | grep -o "Bytecode size: [0-9]* bytes" | grep -o "[0-9]*" || echo "0")
        instructions=$(echo "$analysis_output" | grep -o "Instructions: [0-9]*" | grep -o "[0-9]*" || echo "0")
        compute_units=$(echo "$analysis_output" | grep -o "Compute units: [0-9]*" | grep -o "[0-9]*" || echo "0")
        
        echo -e "  ${GREEN}✓ Analysis complete${NC} (${bytecode_size} bytes, ${instructions} instructions, ${compute_units} CU)"
        
        # Extract instruction breakdown from the analysis output
        if echo "$analysis_output" | grep -q "Instruction Breakdown:"; then
            # Parse instruction frequency data
            echo "$analysis_output" | sed -n '/Instruction Breakdown:/,/^$/p' | grep ": [0-9]* times" | while read -r line; do
                if [[ $line =~ ([A-Z_0-9]+):[[:space:]]*([0-9]+)[[:space:]]*times ]]; then
                    opcode="${BASH_REMATCH[1]}"
                    count="${BASH_REMATCH[2]}"
                    
                    # Track total usage count
                    if [[ -n "${OPCODE_USAGE_COUNT[$opcode]}" ]]; then
                        OPCODE_USAGE_COUNT[$opcode]=$((OPCODE_USAGE_COUNT[$opcode] + count))
                    else
                        OPCODE_USAGE_COUNT[$opcode]=$count
                    fi
                    
                    # Track how many scripts use this opcode
                    if [[ -z "${OPCODE_SCRIPT_COUNT[$opcode]}" ]]; then
                        OPCODE_SCRIPT_COUNT[$opcode]=1
                    else
                        OPCODE_SCRIPT_COUNT[$opcode]=$((OPCODE_SCRIPT_COUNT[$opcode] + 1))
                    fi
                    
                    ALL_OPCODES[$opcode]=1
                fi
            done < <(echo "$analysis_output" | sed -n '/Instruction Breakdown:/,/^$/p' | grep ": [0-9]* times")
        fi
        
        # Save detailed analysis for this script
        echo "$analysis_output" > "$TEMP_DIR/${script_name}_analysis.txt"
        
    else
        ANALYZED_FAILED=$((ANALYZED_FAILED + 1))
        FAILED_SCRIPTS+=("$script_name")
        echo -e "  ${RED}✗ Analysis failed${NC}"
        echo "$analysis_output" > "$TEMP_DIR/${script_name}_error.txt"
    fi
    
    # Clean up temp files
    rm -f "$temp_bin"
    echo ""
}

# Start analysis
echo "Running opcode analysis on all .v files in $SCRIPT_DIR/"
echo ""

# Find and analyze all .v files
while IFS= read -r -d '' script_file; do
    if [ -f "$script_file" ]; then
        analyze_script "$script_file"
    fi
done < <(find "$SCRIPT_DIR" -name "*.v" -type f -print0 | sort -z)

# Generate comprehensive report
echo "=============================================="
echo -e "${BLUE}Comprehensive Opcode Analysis Report${NC}"
echo "=============================================="
echo ""

# Summary statistics
echo -e "${CYAN}Analysis Summary:${NC}"
echo -e "  Total scripts analyzed: ${YELLOW}$TOTAL_SCRIPTS${NC}"
echo -e "  Successful analyses: ${GREEN}$ANALYZED_SUCCESS${NC}"
echo -e "  Failed analyses: ${RED}$ANALYZED_FAILED${NC}"
echo -e "  Unique opcodes found: ${YELLOW}${#ALL_OPCODES[@]}${NC}"
echo ""

# Get the complete list of all possible opcodes from one successful analysis
COMPLETE_OPCODE_LIST=""
if [ ${#ANALYZED_SCRIPTS[@]} -gt 0 ]; then
    first_script="${ANALYZED_SCRIPTS[0]}"
    if [ -f "$TEMP_DIR/${first_script}_analysis.txt" ]; then
        # Extract the complete opcode list from "Sample Unused Opcodes" section
        COMPLETE_OPCODE_LIST=$(grep -A 20 "Sample Unused Opcodes:" "$TEMP_DIR/${first_script}_analysis.txt" | grep "•" | sed 's/.*• //' | tr '\n' ' ')
        
        # Also get used opcodes from any analysis
        for script in "${ANALYZED_SCRIPTS[@]}"; do
            if [ -f "$TEMP_DIR/${script}_analysis.txt" ]; then
                grep -A 20 "Used Opcodes:" "$TEMP_DIR/${script}_analysis.txt" | grep "•" | sed 's/.*• \([A-Z_0-9]*\).*/\1/' >> "$TEMP_DIR/all_used_opcodes.tmp"
            fi
        done
        
        # Combine all opcodes we've seen
        if [ -f "$TEMP_DIR/all_used_opcodes.tmp" ]; then
            COMPLETE_OPCODE_LIST="$COMPLETE_OPCODE_LIST $(cat "$TEMP_DIR/all_used_opcodes.tmp" | sort -u | tr '\n' ' ')"
        fi
    fi
fi

# Most frequently used opcodes
if [ ${#ALL_OPCODES[@]} -gt 0 ]; then
    echo -e "${CYAN}Most Frequently Used Opcodes:${NC}"
    
    # Create a temporary file with opcode usage data
    for opcode in "${!OPCODE_USAGE_COUNT[@]}"; do
        echo "${OPCODE_USAGE_COUNT[$opcode]} $opcode"
    done | sort -nr | head -10 | while read count opcode; do
        script_count=${OPCODE_SCRIPT_COUNT[$opcode]:-0}
        echo -e "  ${GREEN}$opcode${NC}: $count times (used in $script_count scripts)"
    done
    echo ""
fi

# Opcodes with zero usage - this requires checking all possible opcodes
echo -e "${CYAN}Zero Usage Analysis:${NC}"
echo "Checking for opcodes that are never used across all scripts..."
echo ""

# Define the complete list of Five VM opcodes from the WASM analysis
FIVE_VM_OPCODES=(
    "HALT" "PUSH_U64" "PUSH_U8" "PUSH_I64" "PUSH_BOOL" "PUSH_PUBKEY" "POP" "DUP" "SWAP"
    "ADD" "SUB" "MUL" "DIV" "MOD" "EQ" "NE" "LT" "LE" "GT" "GE" "AND" "OR" "NOT" "XOR"
    "JUMP" "JUMP_IF" "CALL" "RETURN" "GET_LOCAL" "SET_LOCAL" "LOAD_ACCOUNT" "STORE_ACCOUNT"
    "LOAD_ACCOUNT_FIELD" "STORE_ACCOUNT_FIELD" "CREATE_ACCOUNT" "GET_CLOCK" "GET_RENT"
    "DERIVE_PDA" "DERIVE_PDA_SEEDS" "CHECK_SIGNER" "CHECK_WRITABLE" "CHECK_OWNER"
    "CHECK_INITIALIZED" "CHECK_PDA" "REQUIRE" "EMIT_EVENT" "LOG_DATA" "PUSH_STRING_LITERAL"
    "STRING_LENGTH" "STRING_CONCAT" "PUSH_ARRAY_LITERAL" "ARRAY_INDEX" "ARRAY_LENGTH"
    "CREATE_ARRAY" "ARRAY_GET" "ARRAY_SET" "ARRAY_LEN" "OPTIONAL_SOME" "OPTIONAL_NONE"
    "RESULT_OK" "RESULT_ERR" "CPI" "CPI_SIGNED"
)

# Find opcodes with zero usage
declare -a ZERO_USAGE_OPCODES=()
declare -a USED_OPCODES=()

for opcode in "${FIVE_VM_OPCODES[@]}"; do
    if [[ -n "${OPCODE_USAGE_COUNT[$opcode]}" ]] && [[ "${OPCODE_USAGE_COUNT[$opcode]}" -gt 0 ]]; then
        USED_OPCODES+=("$opcode")
    else
        # Check if this opcode appears in any instruction breakdown (including unknown opcodes)
        found_in_analysis=false
        for script in "${ANALYZED_SCRIPTS[@]}"; do
            if [ -f "$TEMP_DIR/${script}_analysis.txt" ]; then
                if grep -q "$opcode:" "$TEMP_DIR/${script}_analysis.txt"; then
                    found_in_analysis=true
                    break
                fi
            fi
        done
        
        if [ "$found_in_analysis" = false ]; then
            ZERO_USAGE_OPCODES+=("$opcode")
        fi
    fi
done

# Report opcodes with zero usage
echo -e "${RED}Opcodes with ZERO usage across all $ANALYZED_SUCCESS scripts:${NC}"
if [ ${#ZERO_USAGE_OPCODES[@]} -gt 0 ]; then
    for opcode in "${ZERO_USAGE_OPCODES[@]}"; do
        echo -e "  ${RED}• $opcode${NC} (never used)"
    done
    echo ""
    echo -e "Total unused opcodes: ${RED}${#ZERO_USAGE_OPCODES[@]}${NC} out of ${#FIVE_VM_OPCODES[@]} total Five VM opcodes"
else
    echo -e "  ${GREEN}All opcodes are being used!${NC}"
fi
echo ""

# Report used opcodes
echo -e "${GREEN}Opcodes with usage across all scripts:${NC}"
if [ ${#USED_OPCODES[@]} -gt 0 ]; then
    for opcode in "${USED_OPCODES[@]}"; do
        count=${OPCODE_USAGE_COUNT[$opcode]:-0}
        script_count=${OPCODE_SCRIPT_COUNT[$opcode]:-0}
        echo -e "  ${GREEN}• $opcode${NC}: $count times (in $script_count scripts)"
    done
    echo ""
    echo -e "Total used opcodes: ${GREEN}${#USED_OPCODES[@]}${NC} out of ${#FIVE_VM_OPCODES[@]} total Five VM opcodes"
    echo -e "Usage percentage: ${GREEN}$(( ${#USED_OPCODES[@]} * 100 / ${#FIVE_VM_OPCODES[@]} ))%${NC}"
else
    echo -e "  ${RED}No opcodes detected in analysis${NC}"
fi
echo ""

# Failed scripts report
if [ ${#FAILED_SCRIPTS[@]} -gt 0 ]; then
    echo -e "${RED}Scripts that failed analysis:${NC}"
    for script in "${FAILED_SCRIPTS[@]}"; do
        echo -e "  • ${script}.v"
    done
    echo ""
fi

# Generate JSON report
echo "Generating detailed JSON report..."
cat > "$RESULTS_FILE" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "summary": {
    "total_scripts": $TOTAL_SCRIPTS,
    "analyzed_success": $ANALYZED_SUCCESS,
    "analyzed_failed": $ANALYZED_FAILED,
    "total_five_vm_opcodes": ${#FIVE_VM_OPCODES[@]},
    "opcodes_used": ${#USED_OPCODES[@]},
    "opcodes_unused": ${#ZERO_USAGE_OPCODES[@]},
    "usage_percentage": $(( ${#USED_OPCODES[@]} * 100 / ${#FIVE_VM_OPCODES[@]} ))
  },
  "used_opcodes": [
EOF

# Add used opcodes to JSON
first=true
for opcode in "${USED_OPCODES[@]}"; do
    count=${OPCODE_USAGE_COUNT[$opcode]:-0}
    script_count=${OPCODE_SCRIPT_COUNT[$opcode]:-0}
    if [ "$first" = true ]; then
        first=false
    else
        echo "    ," >> "$RESULTS_FILE"
    fi
    echo -n "    {\"opcode\": \"$opcode\", \"usage_count\": $count, \"script_count\": $script_count}" >> "$RESULTS_FILE"
done

cat >> "$RESULTS_FILE" << EOF

  ],
  "unused_opcodes": [
EOF

# Add unused opcodes to JSON
first=true
for opcode in "${ZERO_USAGE_OPCODES[@]}"; do
    if [ "$first" = true ]; then
        first=false
    else
        echo "    ," >> "$RESULTS_FILE"
    fi
    echo -n "    \"$opcode\"" >> "$RESULTS_FILE"
done

cat >> "$RESULTS_FILE" << EOF

  ],
  "failed_scripts": [
EOF

# Add failed scripts to JSON
first=true
for script in "${FAILED_SCRIPTS[@]}"; do
    if [ "$first" = true ]; then
        first=false
    else
        echo "    ," >> "$RESULTS_FILE"
    fi
    echo -n "    \"$script\"" >> "$RESULTS_FILE"
done

cat >> "$RESULTS_FILE" << EOF

  ]
}
EOF

echo -e "Detailed report saved to: ${BLUE}$RESULTS_FILE${NC}"
echo ""

# Summary
echo "=============================================="
echo -e "${BLUE}Final Summary${NC}"
echo "=============================================="
echo -e "Scripts analyzed: ${GREEN}$ANALYZED_SUCCESS${NC}/${YELLOW}$TOTAL_SCRIPTS${NC}"
echo -e "Opcodes used: ${GREEN}${#USED_OPCODES[@]}${NC}/${YELLOW}${#FIVE_VM_OPCODES[@]}${NC} (${GREEN}$(( ${#USED_OPCODES[@]} * 100 / ${#FIVE_VM_OPCODES[@]} ))%${NC})"
echo -e "Opcodes unused: ${RED}${#ZERO_USAGE_OPCODES[@]}${NC}/${YELLOW}${#FIVE_VM_OPCODES[@]}${NC} (${RED}$(( ${#ZERO_USAGE_OPCODES[@]} * 100 / ${#FIVE_VM_OPCODES[@]} ))%${NC})"

# Clean up temp directory
rm -rf "$TEMP_DIR"

if [ $ANALYZED_FAILED -eq 0 ]; then
    echo ""
    echo -e "${GREEN}Opcode analysis completed successfully!${NC}"
    exit 0
else
    echo ""
    echo -e "${YELLOW}Opcode analysis completed with some failures.${NC}"
    exit 1
fi