#!/bin/bash

# Five CLI Opcode Analysis Runner (Bash 3.2 Compatible)
# Analyzes all .v scripts to show comprehensive opcode usage across the entire test suite

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="test-scripts"
TEMP_DIR=".opcode-analysis-temp"
CLI_BIN="node dist/index.js"
RESULTS_FILE="opcode-usage-report.txt"

# Counters
TOTAL_SCRIPTS=0
ANALYZED_SUCCESS=0
ANALYZED_FAILED=0

# Create temp directory and files for tracking
mkdir -p "$TEMP_DIR"
rm -f "$TEMP_DIR/opcode_counts.txt"
rm -f "$TEMP_DIR/all_opcodes_used.txt"
rm -f "$TEMP_DIR/failed_scripts.txt"

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
        
        # Extract basic info
        bytecode_size=$(echo "$analysis_output" | grep -o "Bytecode size: [0-9]* bytes" | grep -o "[0-9]*" || echo "0")
        instructions=$(echo "$analysis_output" | grep -o "Instructions: [0-9]*" | grep -o "[0-9]*" || echo "0")
        compute_units=$(echo "$analysis_output" | grep -o "Compute units: [0-9]*" | grep -o "[0-9]*" || echo "0")
        
        echo -e "  ${GREEN}✓ Analysis complete${NC} (${bytecode_size} bytes, ${instructions} instructions, ${compute_units} CU)"
        
        # Save full analysis output
        echo "$analysis_output" > "$TEMP_DIR/${script_name}_analysis.txt"
        
        # Extract instruction breakdown
        echo "$analysis_output" | sed -n '/Instruction Breakdown:/,/^$/p' | grep ": [0-9]* times" | while read -r line; do
            if echo "$line" | grep -q ": [0-9]* times"; then
                opcode=$(echo "$line" | sed 's/: [0-9]* times//' | xargs)
                count=$(echo "$line" | grep -o "[0-9]*" | tail -1)
                echo "$opcode $count $script_name" >> "$TEMP_DIR/opcode_counts.txt"
                echo "$opcode" >> "$TEMP_DIR/all_opcodes_used.txt"
            fi
        done
        
    else
        ANALYZED_FAILED=$((ANALYZED_FAILED + 1))
        echo -e "  ${RED}✗ Analysis failed${NC}"
        echo "$script_name" >> "$TEMP_DIR/failed_scripts.txt"
        echo "$analysis_output" > "$TEMP_DIR/${script_name}_error.txt"
    fi
    
    # Clean up temp files
    rm -f "$temp_bin"
    echo ""
}

echo "Running opcode analysis on all .v files in $SCRIPT_DIR/"
echo ""

# Find and analyze all .v files
find "$SCRIPT_DIR" -name "*.v" -type f | sort | while read script_file; do
    analyze_script "$script_file"
done

# Wait for all background processes to complete
wait

# Generate report
echo "=============================================="
echo -e "${BLUE}Comprehensive Opcode Usage Report${NC}"
echo "=============================================="
echo ""

# Count successful analyses
ANALYZED_SUCCESS=$(find "$TEMP_DIR" -name "*_analysis.txt" | wc -l | xargs)
ANALYZED_FAILED=$([ -f "$TEMP_DIR/failed_scripts.txt" ] && cat "$TEMP_DIR/failed_scripts.txt" | wc -l | xargs || echo "0")
TOTAL_SCRIPTS=$((ANALYZED_SUCCESS + ANALYZED_FAILED))

echo -e "${CYAN}Analysis Summary:${NC}"
echo -e "  Total scripts analyzed: ${YELLOW}$TOTAL_SCRIPTS${NC}"
echo -e "  Successful analyses: ${GREEN}$ANALYZED_SUCCESS${NC}"
echo -e "  Failed analyses: ${RED}$ANALYZED_FAILED${NC}"

# Process opcode usage data
if [ -f "$TEMP_DIR/opcode_counts.txt" ]; then
    # Create opcode frequency summary
    echo ""
    echo -e "${CYAN}Most Frequently Used Opcodes:${NC}"
    
    # Sum up usage counts for each opcode
    cat "$TEMP_DIR/opcode_counts.txt" | awk '{opcodes[$1] += $2; scripts[$1]++} END {for (op in opcodes) print opcodes[op], op, scripts[op]}' | sort -nr | head -15 | while read count opcode script_count; do
        echo -e "  ${GREEN}$opcode${NC}: $count times (used in $script_count scripts)"
    done
    
    # Get list of all opcodes that were used
    cat "$TEMP_DIR/all_opcodes_used.txt" | sort -u > "$TEMP_DIR/unique_opcodes_used.txt"
    USED_OPCODE_COUNT=$(cat "$TEMP_DIR/unique_opcodes_used.txt" | wc -l | xargs)
    
    echo ""
    echo -e "Total unique opcodes found in analysis: ${YELLOW}$USED_OPCODE_COUNT${NC}"
else
    echo -e "${RED}No opcode usage data found${NC}"
    USED_OPCODE_COUNT=0
fi

# Define all Five VM opcodes
echo ""
echo -e "${CYAN}Zero Usage Analysis:${NC}"
echo "Checking for opcodes that are never used across all scripts..."
echo ""

# Create list of all Five VM opcodes
cat > "$TEMP_DIR/all_five_vm_opcodes.txt" << 'EOF'
HALT
PUSH_U64
PUSH_U8
PUSH_I64
PUSH_BOOL
PUSH_PUBKEY
POP
DUP
SWAP
ADD
SUB
MUL
DIV
MOD
EQ
NE
LT
LE
GT
GE
AND
OR
NOT
XOR
JUMP
JUMP_IF
CALL
RETURN
GET_LOCAL
SET_LOCAL
LOAD_ACCOUNT
STORE_ACCOUNT
LOAD_ACCOUNT_FIELD
STORE_ACCOUNT_FIELD
CREATE_ACCOUNT
GET_CLOCK
GET_RENT
DERIVE_PDA
DERIVE_PDA_SEEDS
CHECK_SIGNER
CHECK_WRITABLE
CHECK_OWNER
CHECK_INITIALIZED
CHECK_PDA
REQUIRE
EMIT_EVENT
LOG_DATA
PUSH_STRING_LITERAL
STRING_LENGTH
STRING_CONCAT
PUSH_ARRAY_LITERAL
ARRAY_INDEX
ARRAY_LENGTH
CREATE_ARRAY
ARRAY_GET
ARRAY_SET
ARRAY_LEN
OPTIONAL_SOME
OPTIONAL_NONE
RESULT_OK
RESULT_ERR
CPI
CPI_SIGNED
EOF

# Count total Five VM opcodes
TOTAL_FIVE_VM_OPCODES=$(cat "$TEMP_DIR/all_five_vm_opcodes.txt" | wc -l | xargs)

# Find unused opcodes by checking if they appear in any analysis
if [ -f "$TEMP_DIR/unique_opcodes_used.txt" ]; then
    # Find opcodes that are in Five VM but not in our used list
    comm -23 <(sort "$TEMP_DIR/all_five_vm_opcodes.txt") <(sort "$TEMP_DIR/unique_opcodes_used.txt") > "$TEMP_DIR/unused_opcodes.txt"
    
    # Also check for opcodes that appear in the instruction breakdown of any script
    for opcode in $(cat "$TEMP_DIR/all_five_vm_opcodes.txt"); do
        if grep -r "$opcode:" "$TEMP_DIR"/*_analysis.txt >/dev/null 2>&1; then
            echo "$opcode" >> "$TEMP_DIR/actually_used_opcodes.txt"
        fi
    done
    
    if [ -f "$TEMP_DIR/actually_used_opcodes.txt" ]; then
        sort -u "$TEMP_DIR/actually_used_opcodes.txt" > "$TEMP_DIR/really_used_opcodes.txt"
        comm -23 <(sort "$TEMP_DIR/all_five_vm_opcodes.txt") <(sort "$TEMP_DIR/really_used_opcodes.txt") > "$TEMP_DIR/truly_unused_opcodes.txt"
        UNUSED_COUNT=$(cat "$TEMP_DIR/truly_unused_opcodes.txt" | wc -l | xargs)
        USED_COUNT=$(cat "$TEMP_DIR/really_used_opcodes.txt" | wc -l | xargs)
    else
        UNUSED_COUNT=$TOTAL_FIVE_VM_OPCODES
        USED_COUNT=0
    fi
else
    UNUSED_COUNT=$TOTAL_FIVE_VM_OPCODES
    USED_COUNT=0
fi

# Report results
echo -e "${RED}Opcodes with ZERO usage across all $ANALYZED_SUCCESS scripts:${NC}"
if [ -f "$TEMP_DIR/truly_unused_opcodes.txt" ] && [ $UNUSED_COUNT -gt 0 ]; then
    cat "$TEMP_DIR/truly_unused_opcodes.txt" | while read opcode; do
        echo -e "  ${RED}• $opcode${NC} (never used)"
    done
    echo ""
    echo -e "Total unused opcodes: ${RED}$UNUSED_COUNT${NC} out of $TOTAL_FIVE_VM_OPCODES total Five VM opcodes"
else
    echo -e "  ${GREEN}All opcodes are being used!${NC}"
fi

echo ""
echo -e "${GREEN}Opcodes with usage:${NC}"
if [ -f "$TEMP_DIR/really_used_opcodes.txt" ] && [ $USED_COUNT -gt 0 ]; then
    cat "$TEMP_DIR/really_used_opcodes.txt" | while read opcode; do
        # Get usage count for this opcode
        usage_count=$(grep "^$opcode " "$TEMP_DIR/opcode_counts.txt" 2>/dev/null | awk '{sum += $2} END {print sum+0}')
        script_count=$(grep "^$opcode " "$TEMP_DIR/opcode_counts.txt" 2>/dev/null | wc -l | xargs)
        echo -e "  ${GREEN}• $opcode${NC}: $usage_count times (in $script_count scripts)"
    done
    echo ""
    echo -e "Total used opcodes: ${GREEN}$USED_COUNT${NC} out of $TOTAL_FIVE_VM_OPCODES total Five VM opcodes"
    echo -e "Usage percentage: ${GREEN}$(( USED_COUNT * 100 / TOTAL_FIVE_VM_OPCODES ))%${NC}"
fi

# Failed scripts
if [ -f "$TEMP_DIR/failed_scripts.txt" ] && [ $ANALYZED_FAILED -gt 0 ]; then
    echo ""
    echo -e "${RED}Scripts that failed analysis:${NC}"
    cat "$TEMP_DIR/failed_scripts.txt" | while read script; do
        echo -e "  • ${script}.v"
    done
fi

# Save detailed report
{
    echo "Five CLI Opcode Usage Analysis Report"
    echo "Generated: $(date)"
    echo "======================================"
    echo ""
    echo "SUMMARY:"
    echo "- Total scripts: $TOTAL_SCRIPTS"
    echo "- Successful analyses: $ANALYZED_SUCCESS"
    echo "- Failed analyses: $ANALYZED_FAILED"
    echo "- Total Five VM opcodes: $TOTAL_FIVE_VM_OPCODES"
    echo "- Opcodes used: $USED_COUNT"
    echo "- Opcodes unused: $UNUSED_COUNT"
    echo "- Usage percentage: $(( USED_COUNT * 100 / TOTAL_FIVE_VM_OPCODES ))%"
    echo ""
    
    if [ -f "$TEMP_DIR/truly_unused_opcodes.txt" ]; then
        echo "UNUSED OPCODES (Zero usage across all scripts):"
        cat "$TEMP_DIR/truly_unused_opcodes.txt" | sed 's/^/- /'
        echo ""
    fi
    
    if [ -f "$TEMP_DIR/really_used_opcodes.txt" ]; then
        echo "USED OPCODES:"
        cat "$TEMP_DIR/really_used_opcodes.txt" | while read opcode; do
            usage_count=$(grep "^$opcode " "$TEMP_DIR/opcode_counts.txt" 2>/dev/null | awk '{sum += $2} END {print sum+0}')
            script_count=$(grep "^$opcode " "$TEMP_DIR/opcode_counts.txt" 2>/dev/null | wc -l | xargs)
            echo "- $opcode: $usage_count times (in $script_count scripts)"
        done
        echo ""
    fi
    
    echo "MOST FREQUENTLY USED OPCODES:"
    if [ -f "$TEMP_DIR/opcode_counts.txt" ]; then
        cat "$TEMP_DIR/opcode_counts.txt" | awk '{opcodes[$1] += $2; scripts[$1]++} END {for (op in opcodes) print opcodes[op], op, scripts[op]}' | sort -nr | head -10 | while read count opcode script_count; do
            echo "- $opcode: $count times (used in $script_count scripts)"
        done
    fi
    
} > "$RESULTS_FILE"

echo ""
echo "=============================================="
echo -e "${BLUE}Final Summary${NC}"
echo "=============================================="
echo -e "Scripts analyzed: ${GREEN}$ANALYZED_SUCCESS${NC}/${YELLOW}$TOTAL_SCRIPTS${NC}"
echo -e "Opcodes used: ${GREEN}$USED_COUNT${NC}/${YELLOW}$TOTAL_FIVE_VM_OPCODES${NC} (${GREEN}$(( USED_COUNT * 100 / TOTAL_FIVE_VM_OPCODES ))%${NC})"
echo -e "Opcodes unused: ${RED}$UNUSED_COUNT${NC}/${YELLOW}$TOTAL_FIVE_VM_OPCODES${NC} (${RED}$(( UNUSED_COUNT * 100 / TOTAL_FIVE_VM_OPCODES ))%${NC})"
echo ""
echo -e "Detailed report saved to: ${BLUE}$RESULTS_FILE${NC}"

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