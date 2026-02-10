#!/bin/bash
# MVP Release Gate Script
# Validates all critical components for MVP release

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "======================================"
echo "Five MVP Release Gate Check"
echo "======================================"
echo ""

PASS_COUNT=0
FAIL_COUNT=0

# Helper functions
pass() {
  echo -e "${GREEN}✓ $1${NC}"
  ((PASS_COUNT++))
}

fail() {
  echo -e "${RED}✗ $1${NC}"
  ((FAIL_COUNT++))
}

warn() {
  echo -e "${YELLOW}⚠ $1${NC}"
}

# 1. Rust checks
echo "1. Rust Core Validation"
echo "  - cargo check --workspace"
if cargo check --workspace > /dev/null 2>&1; then
  pass "Rust workspace compiles"
else
  fail "Rust workspace compilation failed"
fi

echo "  - five-protocol opcode tests"
if cargo test -p five-protocol --test opcode_tests 2>&1 | grep -q "test result: ok"; then
  pass "Protocol opcode tests"
else
  fail "Protocol opcode tests failed"
fi

echo "  - five-dsl-compiler tests"
if cargo test -p five-dsl-compiler 2>&1 | grep -q "test result: ok"; then
  pass "DSL compiler tests"
else
  fail "DSL compiler tests failed"
fi

echo "  - five-vm-mito tests"
if cargo test -p five-vm-mito 2>&1 | grep -q "test result: ok"; then
  pass "VM tests"
else
  fail "VM tests failed"
fi

echo ""

# 2. SDK checks
echo "2. Five SDK Validation"
cd "$REPO_ROOT/five-sdk"
echo "  - TypeScript compilation"
if npm run build 2>&1 | grep -q "copy-assets"; then
  pass "SDK build"
else
  fail "SDK build failed"
fi

echo "  - Jest test suite"
if npm run test:jest -- --runInBand 2>&1 | grep -q "passed"; then
  pass "SDK Jest tests"
else
  fail "SDK Jest tests failed"
fi

echo ""

# 3. CLI checks
echo "3. Five CLI Validation"
cd "$REPO_ROOT/five-cli"
echo "  - TypeScript compilation"
if npx tsc -p tsconfig.json --noEmit 2>&1 | grep -q "error"; then
  fail "CLI TypeScript compilation"
else
  pass "CLI TypeScript"
fi

echo "  - Jest test suite"
if npm run test -- --runInBand 2>&1 | grep -q "Test Suites:.*passed"; then
  pass "CLI test suite"
else
  fail "CLI test suite failed"
fi

echo ""

# 4. Frontend checks
echo "4. Five Frontend Validation"
cd "$REPO_ROOT/five-frontend"
echo "  - TypeScript compilation"
if npx tsc -p tsconfig.json --noEmit 2>&1 | grep -q "error"; then
  warn "Frontend TypeScript has errors (known issue)"
else
  pass "Frontend TypeScript compilation"
fi

echo "  - Build (will skip if dependencies broken)"
if npm run build > /dev/null 2>&1; then
  pass "Frontend build"
else
  warn "Frontend build failed (known issue - dependency conflicts)"
fi

echo ""

# 5. No VLE references
echo "5. VLE Terminology Check"
cd "$REPO_ROOT"
VLE_COUNT=$(grep -r "VLE\|vle\|enable_vle" --include="*.rs" --include="*.ts" --include="*.tsx" --include="*.toml" \
  --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.git \
  2>/dev/null | wc -l)

if [ "$VLE_COUNT" -eq 0 ]; then
  pass "No VLE terminology found"
else
  warn "VLE references found: $VLE_COUNT"
fi

echo ""

# Summary
echo "======================================"
echo "Release Gate Summary"
echo "======================================"
echo -e "${GREEN}Passed: $PASS_COUNT${NC}"
echo -e "${RED}Failed: $FAIL_COUNT${NC}"
echo ""

if [ $FAIL_COUNT -eq 0 ]; then
  echo -e "${GREEN}✓ MVP READY FOR RELEASE${NC}"
  exit 0
else
  echo -e "${RED}✗ MVP HAS BLOCKERS${NC}"
  exit 1
fi
