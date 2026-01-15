#!/bin/bash

# Run cargo test and capture output, while also showing it to the user
# We use a temporary file to store the output for processing
OUTPUT_FILE=$(mktemp)

echo "Running cargo test --workspace..."
# 2>&1 redirects stderr to stdout so we capture everything
# tee allows the user to see progress in real-time
cargo test --workspace 2>&1 | tee "$OUTPUT_FILE"

echo ""
echo "========================================"
echo "          FIVE VM TEST SUMMARY          "
echo "========================================"

# Initialize counters
PASSED_TOTAL=0
FAILED_TOTAL=0
IGNORED_TOTAL=0

# Process the output file
# We grep for "test result:" lines which look like:
# test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
while IFS= read -r line; do
    if [[ "$line" =~ ^test\ result:\ .* ]]; then
        # Extract the number before "passed;"
        # awk -F'passed;' splits the line at "passed;", take the first part ($1)
        # awk '{print $NF}' prints the last field (space-separated) of that part, which is the number
        p=$(echo "$line" | awk -F'passed;' '{print $1}' | awk '{print $NF}')
        
        # Extract the number before "failed;"
        f=$(echo "$line" | awk -F'failed;' '{print $1}' | awk '{print $NF}')
        
        # Extract the number before "ignored;"
        i=$(echo "$line" | awk -F'ignored;' '{print $1}' | awk '{print $NF}')
        
        # Ensure we only add if they are numbers (sanity check)
        if [[ "$p" =~ ^[0-9]+$ ]]; then PASSED_TOTAL=$((PASSED_TOTAL + p)); fi
        if [[ "$f" =~ ^[0-9]+$ ]]; then FAILED_TOTAL=$((FAILED_TOTAL + f)); fi
        if [[ "$i" =~ ^[0-9]+$ ]]; then IGNORED_TOTAL=$((IGNORED_TOTAL + i)); fi
    fi
done < "$OUTPUT_FILE"

# Clean up
rm "$OUTPUT_FILE"

TOTAL=$((PASSED_TOTAL + FAILED_TOTAL + IGNORED_TOTAL))

printf "% -15s %5d\n" "Total Tests:" "$TOTAL"
printf "% -15s \033[32m%5d\033[0m\n" "Passed:" "$PASSED_TOTAL"
printf "% -15s \033[31m%5d\033[0m\n" "Failed:" "$FAILED_TOTAL"
printf "% -15s \033[33m%5d\033[0m\n" "Ignored/Skipped:" "$IGNORED_TOTAL"

echo "========================================"

if [ "$FAILED_TOTAL" -gt 0 ]; then
    echo "Overall Status: ❌ FAILED"
    exit 1
else
    echo "Overall Status: ✅ SUCCESS"
    exit 0
fi
