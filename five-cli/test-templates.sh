#!/usr/bin/env bash

# Five CLI Template Runner
# Compiles Five DSL templates under five-cli/templates

set -o pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CLI_ENTRY="$SCRIPT_DIR/dist/index.js"
TEMPLATE_DIR="$SCRIPT_DIR/templates"

VERBOSE=false
SHOW_OUTPUT=false
FAIL_FAST=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --verbose|-v) VERBOSE=true; shift ;;
    --show-output|-o) SHOW_OUTPUT=true; shift ;;
    --fail-fast) FAIL_FAST=true; shift ;;
    --help|-h)
      echo "Usage: $0 [--verbose|-v] [--show-output|-o] [--fail-fast]"
      exit 0
      ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

if [ ! -d "$TEMPLATE_DIR" ]; then
  echo -e "${RED}Template dir not found:${NC} $TEMPLATE_DIR"
  exit 1
fi

echo -e "${CYAN}Running template compilation in:${NC} $TEMPLATE_DIR"

total=0
passed=0
failed=0
declare -a FAILS

for file in "$TEMPLATE_DIR"/*.v; do
  [ -e "$file" ] || continue
  total=$((total+1))
  echo -e "\n${YELLOW}• Validating:${NC} ${file##*/}"
  if output=$(node "$CLI_ENTRY" compile "$file" --validate 2>&1); then
    echo -e "${GREEN}✓ Passed${NC} ${VERBOSE:+\n${output}}"
    passed=$((passed+1))
  else
    echo -e "${RED}✗ Failed${NC}"
    failed=$((failed+1))
    FAILS+=("$file")
    $SHOW_OUTPUT && echo "$output"
    $FAIL_FAST && break
  fi
done

echo -e "\n${CYAN}Summary:${NC} $passed passed, $failed failed, $total total"
if [ $failed -gt 0 ]; then
  echo -e "${RED}Failures:${NC}"
  for f in "${FAILS[@]}"; do echo "  - ${f##*/}"; done
  exit 1
fi
exit 0
