#!/bin/bash

# Five VM Mito Comprehensive Test Runner
# 
# This script runs all VM unit tests and provides detailed reporting
# to help identify and fix the issues causing the 58.9% pass rate in .v tests

set -euo pipefail

echo "🚀 Five VM Mito - Comprehensive Test Suite"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to run test category with timing
run_test_category() {
    local category=$1
    local filter=$2
    echo -e "${BLUE}Running $category tests...${NC}"
    
    start_time=$(date +%s)
    
    if cargo test $filter -- --nocapture 2>&1; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo -e "${GREEN}✅ $category tests completed in ${duration}s${NC}"
        echo ""
        return 0
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo -e "${RED}❌ $category tests failed after ${duration}s${NC}"
        echo ""
        return 1
    fi
}

# Run individual test categories
echo "Running Core VM Operation Tests..."
echo "================================="
run_test_category "Core VM" "test_core_vm" || true

echo "Running Account System Tests..."
echo "=============================="
run_test_category "Account System" "test_account_system" || true

echo "Running PDA Operation Tests..."
echo "============================="
run_test_category "PDA Operations" "test_pda_operations" || true

echo "Running Function Call Tests..."
echo "=============================="
run_test_category "Function Calls" "test_function_calls" || true

echo "Running Array Operation Tests..."
echo "==============================="
run_test_category "Array Operations" "test_array_operations" || true

echo "Running Integration Tests..."
echo "============================"
run_test_category "Integration" "test_integration" || true

echo "Running Property-Based Tests..."
echo "==============================="
run_test_category "Property-Based" "test_property_based" || true

echo "Running Test Framework Tests..."
echo "==============================="
run_test_category "Test Framework" "test_framework" || true

echo "Running Legacy Tests..."
echo "======================"
run_test_category "Legacy" "tests::" || true

# Run all tests together for overall summary
echo "Running Complete Test Suite..."
echo "============================="
echo -e "${YELLOW}Running all tests together for comprehensive summary...${NC}"

start_time=$(date +%s)
if cargo test -- --nocapture; then
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    echo -e "${GREEN}🎉 All tests completed successfully in ${duration}s${NC}"
    exit_code=0
else
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    echo -e "${RED}❌ Some tests failed. Total runtime: ${duration}s${NC}"
    exit_code=1
fi

echo ""
echo "Test Summary"
echo "============"
echo "• Core VM Tests: Basic operations (arithmetic, logic, stack, control flow)"
echo "• Account System Tests: Account constraints, state management, authorization"
echo "• PDA Operation Tests: Program Derived Address operations and validation"
echo "• Function Call Tests: Function calls, parameter handling, call stack management"
echo "• Array Operation Tests: Array/string operations, memory management"
echo "• Integration Tests: End-to-end scenarios mirroring failing .v tests"
echo "• Property-Based Tests: Mathematical properties and invariants"
echo "• Test Framework Tests: Testing infrastructure validation"
echo ""

if [ $exit_code -eq 0 ]; then
    echo -e "${GREEN}🎯 Next Steps:${NC}"
    echo "1. All Rust unit tests are passing!"
    echo "2. Run the .v test suite to see if issues are resolved"
    echo "3. Focus on any remaining .v test failures"
else
    echo -e "${YELLOW}🔧 Next Steps:${NC}"
    echo "1. Fix failing Rust unit tests first (better debugging info)"
    echo "2. Focus on test categories with the most failures"
    echo "3. Implement missing VM operations identified by tests"
    echo "4. Add AccountInfo mock implementation for account tests"
    echo "5. Complete pubkey reference implementation for PDA tests"
fi

echo ""
echo -e "${BLUE}📊 For detailed test results, run:${NC}"
echo "   cargo test -- --nocapture | grep -E '(test result|FAILED|ERROR)'"
echo ""
echo -e "${BLUE}🐛 For debugging specific failures, run:${NC}"
echo "   cargo test <specific_test_name> -- --nocapture"

exit $exit_code
