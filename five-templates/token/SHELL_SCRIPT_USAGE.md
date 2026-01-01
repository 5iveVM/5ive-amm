# Token Template E2E Test Shell Script

Automated testing script for building, deploying, and testing the token template.

## Quick Start

```bash
cd /Users/amberjackson/Documents/Development/five-org/five-templates/token

# Build and test (no deployment)
./e2e-token-test.sh

# Build, deploy to localnet, and test
./e2e-token-test.sh --deploy

# Show help
./e2e-token-test.sh --help
```

## Features

✅ **Automatic Prerequisites Check**
- Verifies Five CLI, Solana CLI, Node.js installed
- Checks @solana/web3.js dependency
- Validates RPC connection for deployment

✅ **Colored Output**
- Clear status indicators (✓ success, ✗ error, ⚠ warning)
- Color-coded sections and messages
- Progress tracking

✅ **Build Pipeline**
- Compiles Five DSL to bytecode
- Validates artifact creation
- Shows bytecode size

✅ **Deployment** (optional)
- Deploys to Solana localnet
- Extracts and displays program ID
- Validates deployment success

✅ **Test Execution**
- Runs 3-user E2E test scenario
- Captures transaction signatures
- Measures compute unit consumption

✅ **Reporting**
- Displays test summary
- Parses JSON report (if jq installed)
- Shows compute unit statistics

## Usage

### Basic Build and Test

```bash
./e2e-token-test.sh
```

**What it does:**
1. Checks prerequisites
2. Builds token template
3. Runs E2E tests with 3 users
4. Shows results

**Output:**
- Build status
- Test progress
- Transaction IDs and CU costs
- JSON report with detailed metrics

### Build, Deploy, and Test

```bash
./e2e-token-test.sh --deploy
```

**What it does:**
1. Checks prerequisites (including RPC connection)
2. Builds token template
3. **Deploys to localnet**
4. Runs E2E tests
5. Shows results

**Note:** Requires `solana-test-validator` running and updated program IDs in test file.

### Verbose Output

```bash
./e2e-token-test.sh --verbose
./e2e-token-test.sh --deploy --verbose
```

Shows detailed build and test output for debugging.

### Clean Artifacts

```bash
./e2e-token-test.sh --clean
```

Removes:
- Build directory
- Report files
- Cached files
- Previous artifacts

### Skip Build

```bash
./e2e-token-test.sh --skip-build
```

Uses existing compiled artifact. Useful for running tests multiple times without rebuilding.

### Custom RPC URL

```bash
./e2e-token-test.sh --deploy --rpc-url http://localhost:8899
```

### Show Help

```bash
./e2e-token-test.sh --help
```

Displays all options and examples.

## Pipeline Stages

### 1. Prerequisites Check

Verifies:
- ✓ Five CLI installed and working
- ✓ Solana CLI installed and working
- ✓ Node.js 18+ installed
- ✓ @solana/web3.js dependency
- ✓ RPC connection (if deploying)

Fails fast if requirements not met.

### 2. Build

```bash
cd <project-root>
five build
```

Output:
- Bytecode: `build/five-token-template.five`
- Size: Shown in output
- Status: Success/failure with details

### 3. Deploy (optional)

```bash
five deploy build/five-token-template.five
```

Output:
- Program ID
- Deployment status
- Warning to update test constants

### 4. Test Execution

Runs Node.js test with full environment.

Output per operation:
```
✅ init_mint: 10777 CU
   TX: 3xKq5mP2nL9vQ7rB8jG1kD5mN3pR6sT9vX2yZ4aB5cD6eF7gH8iJ9kL0mN1pQ2rS3
```

### 5. Report Generation

Saves `e2e-test-report.json` with:
- Individual test results
- Transaction signatures
- Compute unit costs
- Summary statistics

## Output Examples

### Successful Run

```
================================================================================
Token Template E2E Test Runner
================================================================================

Configuration:
  Project Root:   /path/to/five-templates/token
  Source:         /path/to/five-templates/token/src/token.v
  Build Output:   /path/to/five-templates/token/build/five-token-template.five
  RPC URL:        http://127.0.0.1:8899

================================================================================
Checking Prerequisites
================================================================================

▶ Checking Five CLI...
✓ Five CLI installed: Five CLI 0.1.0

▶ Checking Solana CLI...
✓ Solana CLI installed: solana-cli 1.18.22

▶ Checking Node.js...
✓ Node.js installed: v20.11.0

▶ Checking @solana/web3.js...
✓ @solana/web3.js installed

================================================================================
Building Token Template
================================================================================

▶ Source: /path/to/src/token.v
▶ Building with Five CLI...
✓ Build completed
ℹ Artifact: /path/to/build/five-token-template.five (1.6K)

================================================================================
Running E2E Tests
================================================================================

▶ Running: /path/to/e2e-token-test.mjs
ℹ RPC URL: http://127.0.0.1:8899
────────────────────────────────────────

🎭 Token Template E2E Test - 3 User Story

================================================================================
SETUP: Creating 3 Users
================================================================================

ℹ User1 (Authority): 9B8...
ℹ User2 (Holder):    5K3...
ℹ User3 (Holder):    2L7...

... [test output] ...

================================================================================
Summary
================================================================================

Status:
✓ Build
✓ Deployment (skipped)
✓ Tests

Artifacts:
ℹ Bytecode: /path/to/build/five-token-template.five
ℹ Report: /path/to/e2e-test-report.json

✓ All tests completed successfully!
```

### With Deployment

```
... [build output] ...

================================================================================
Deploying to Localnet
================================================================================

▶ Deploying /path/to/build/five-token-template.five...
✓ Deployment successful
ℹ Program ID: 9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH
⚠ Update PROGRAM_ID, VM_STATE_PDA, and TOKEN_SCRIPT_ACCOUNT in e2e-token-test.mjs

... [test output] ...
```

## Error Handling

### Five CLI Not Found

```
✗ Five CLI not found. Install with: cargo install --git https://github.com/five-protocol/five-cli
```

**Solution:** Install Five CLI following the instructions.

### Build Failed

```
✗ Build failed
ℹ Run with --verbose for details
```

**Solution:** Run with `--verbose` to see error details, then check `src/token.v`.

### RPC Connection Failed

```
⚠ Cannot connect to http://127.0.0.1:8899
⚠ Make sure solana-test-validator is running
✗ Cannot deploy without a running validator
```

**Solution:** Start localnet validator with `solana-test-validator`.

### Test Failed

```
✗ Tests failed
```

**Solution:** Check the generated `e2e-test-report.json` for which operations failed.

## Report Analysis

View detailed report:

```bash
# With jq (formatted JSON)
jq . e2e-test-report.json | less

# Raw JSON
cat e2e-test-report.json

# Extract summary
jq '.summary' e2e-test-report.json
```

Example summary:

```json
{
  "totalTests": 12,
  "successful": 12,
  "failed": 0,
  "successRate": "100.0%",
  "totalComputeUnits": 125487,
  "avgComputeUnitsPerTx": 10458,
  "minCU": 9234,
  "maxCU": 15892
}
```

## Advanced Usage

### Continuous Testing

```bash
# Watch for changes and retest
while true; do
    ./e2e-token-test.sh --skip-build
    sleep 10
done
```

### Compare Runs

```bash
# Save baseline
cp e2e-test-report.json baseline-report.json

# Make changes and test
./e2e-token-test.sh

# Compare results
diff baseline-report.json e2e-test-report.json
```

### Full Development Cycle

```bash
# Clean start
./e2e-token-test.sh --clean

# Edit src/token.v...

# Test locally first (no deploy)
./e2e-token-test.sh

# If satisfied, deploy and test on localnet
./e2e-token-test.sh --deploy

# Extract program ID for production
jq '.programId' e2e-test-report.json
```

## Troubleshooting

### Script not executable

```bash
chmod +x e2e-token-test.sh
```

### Permissions denied on localnet deploy

Ensure `~/.config/solana/id.json` has sufficient balance:

```bash
solana balance
solana airdrop 10  # if needed
```

### Tests hang or timeout

```bash
# Kill any stuck test processes
pkill -f e2e-token-test.mjs

# Check validator is still running
solana cluster-info

# Restart validator if needed
solana-test-validator
```

### @solana/web3.js not found

```bash
# Install in the token template directory
npm install @solana/web3.js

# Or globally
npm install -g @solana/web3.js
```

## Performance Tips

1. **Parallel runs**: Each run uses unique keypairs, can run multiple simultaneously
2. **Skip build**: Use `--skip-build` for repeated test runs
3. **Minimal logging**: Remove `--verbose` for faster output
4. **Local testing**: Test locally without `--deploy` first, then deploy

## Environment Variables

None required, but can override defaults:

```bash
# Custom RPC (use --rpc-url instead)
export RPC_URL="http://localhost:8899"
```

## Files Generated

- `build/five-token-template.five` - Compiled bytecode
- `e2e-test-report.json` - Detailed test results
- `.five/build.json` - Build metadata
- `.five/` - Build cache directory

## Integration with CI/CD

Example GitHub Actions workflow:

```yaml
name: Token Template Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '20'
      - name: Install Solana CLI
        run: sh -c "$(curl -sSfL https://release.solana.com/v1.18.0/install)"
      - name: Install Five CLI
        run: cargo install --git https://github.com/five-protocol/five-cli
      - name: Run tests
        run: cd five-templates/token && ./e2e-token-test.sh
```

## See Also

- `E2E_TEST_README.md` - Detailed test documentation
- `QUICK_START.md` - Quick reference
- `src/token.v` - Token implementation
- `e2e-token-test.mjs` - Test code
