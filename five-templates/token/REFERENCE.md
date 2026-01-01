# Token Template - Quick Reference Card

## Files Overview

```
token/
├── 📄 README.md                 ← START HERE: Overview and getting started
├── 📄 QUICK_START.md            ← Token function reference
├── 📄 SHELL_SCRIPT_USAGE.md     ← Complete shell script guide
├── 📄 E2E_TEST_README.md        ← Detailed test guide
│
├── 🔧 e2e-token-test.sh         ← MAIN: Automated build/deploy/test
├── 🧪 e2e-token-test.mjs        ← Node.js test code (3 users)
│
├── 💾 src/
│   ├── token.v                  ← Implementation (400 lines, 19 functions)
│   └── main.v                   ← Entry point
│
├── 📦 five.toml                 ← Build config
└── 📦 build/
    └── five-token-template.five ← Compiled bytecode
```

## One-Liner Commands

```bash
# Build and test locally
./e2e-token-test.sh

# Build, deploy to localnet, test
./e2e-token-test.sh --deploy

# Show help
./e2e-token-test.sh --help

# Clean artifacts
./e2e-token-test.sh --clean

# Verbose output
./e2e-token-test.sh --verbose

# Skip rebuild
./e2e-token-test.sh --skip-build

# Custom RPC
./e2e-token-test.sh --rpc-url http://localhost:8899
```

## Shell Script Options

| Option | Purpose |
|--------|---------|
| `--deploy` | Build, deploy to localnet, test |
| `--clean` | Remove build artifacts and reports |
| `--skip-build` | Use existing compiled artifacts |
| `--verbose, -v` | Show detailed output |
| `--rpc-url URL` | Custom RPC endpoint |
| `--help, -h` | Show help message |

## Token Operations (19 total)

### Initialization (4)
- `init_mint(decimals)` - Create mint
- `init_token_account()` - Create account
- `create_associated_token_account()` - ATA
- `create_deterministic_token_account()` - PDA

### Minting (2)
- `mint_to(amount)` - Mint with overflow check
- `burn(amount)` - Destroy tokens

### Transfers (4)
- `transfer(amount)` - Direct transfer
- `transfer_from(amount)` - Delegated transfer
- `approve(amount)` - Grant delegation
- `revoke()` - Revoke delegation

### Freezing (2)
- `freeze_account()` - Lock account
- `thaw_account()` - Unlock account

### Authority (4)
- `set_mint_authority(new_auth)` - Change authority
- `set_freeze_authority(new_auth)` - Change freeze auth
- `disable_mint()` - Permanently disable minting
- `disable_freeze()` - Permanently disable freezing

### Validation (3)
- `verify_account_mint()` - Check mint
- `verify_account_owner()` - Check owner
- `verify_account_full()` - Check both

## Test Output Example

```
✅ init_mint: 10777 CU
   TX: 3xKq5mP2nL9vQ7rB8jG1kD5mN3pR6sT9vX2yZ4aB5cD6eF7gH8iJ9kL0mN1pQ2rS3

✅ mint_to User1 (1000): 12234 CU
   TX: 8mLp2kQ3rS4tU5vW6xY7zB8cD9eF1gH2iJ3kL4mN5pO6qR7sT8uV9wX0yZ1aB2cD3

📊 Summary:
   Total Tests: 12
   Successful: 12
   Success Rate: 100%
   Total CU: 125,487
   Avg CU: 10,458
```

## Test Scenario

```
User1 (Authority) → Creates mint, mints 1000 tokens
User2 (Holder)    → Receives 500, transfers 100 to User3
User3 (Holder)    → Receives 500 + 100, approves User2 as delegate

User2 as delegate → Transfers 50 from User3 to User1
User3             → Revokes delegation
User1             → Freezes User2, unfreezes User2
User1             → Transfers authority to User2
User2             → Disables minting (irreversible!)
```

## Compute Unit Baselines

| Operation | CU Range | Typical |
|-----------|----------|---------|
| Simple (revoke, thaw) | 8K-12K | 10K |
| Init (account, mint) | 10K-15K | 12K |
| State mutation (transfer, mint) | 12K-20K | 15K |
| Authority change | 12K-18K | 14K |

**Average per transaction: ~10.5K CU**

## Prerequisites

```bash
# Check Five CLI
five --version

# Check Solana CLI
solana --version

# Check Node.js
node --version

# Install web3.js
npm install @solana/web3.js

# Start localnet (for deployment)
solana-test-validator
```

## Generated Files

| File | Purpose | Created By |
|------|---------|-----------|
| `build/five-token-template.five` | Compiled bytecode | Build step |
| `e2e-test-report.json` | Test results | Test step |
| `.five/build.json` | Build metadata | Build step |

## Troubleshooting Checklist

- [ ] Five CLI installed? (`five --version`)
- [ ] Solana CLI installed? (`solana --version`)
- [ ] Node.js 18+? (`node --version`)
- [ ] @solana/web3.js installed? (`npm ls @solana/web3.js`)
- [ ] Validator running? (`solana cluster-info`)
- [ ] Script executable? (`chmod +x e2e-token-test.sh`)

## Documentation Map

```
You Are Here: REFERENCE.md
  ↓
README.md ..................... Overview and architecture
QUICK_START.md ................ Function reference
SHELL_SCRIPT_USAGE.md ......... Script documentation
E2E_TEST_README.md ............ Test setup and troubleshooting
src/token.v ................... Implementation source
e2e-token-test.sh ............. Build/deploy/test automation
e2e-token-test.mjs ............ Test code (Node.js)
```

## Common Workflows

### Local Development
```bash
# Initial setup
npm install @solana/web3.js

# Edit src/token.v...

# Test locally (no deployment)
./e2e-token-test.sh

# View results
cat e2e-test-report.json | jq '.summary'
```

### Production Deployment
```bash
# Start validator
solana-test-validator &

# Deploy to localnet
./e2e-token-test.sh --deploy

# Extract program ID
jq '.tests[0].signature' e2e-test-report.json

# Update your application with program ID
```

### Continuous Testing
```bash
# Watch and test
while true; do
    ./e2e-token-test.sh --skip-build
    sleep 5
done
```

### Performance Analysis
```bash
# Run test
./e2e-token-test.sh

# Extract compute unit stats
jq '.summary | {totalCU: .totalComputeUnits, avgCU: .avgComputeUnitsPerTx, minCU: .minCU, maxCU: .maxCU}' e2e-test-report.json

# Find most expensive operation
jq '.tests | sort_by(.computeUnits) | reverse[0]' e2e-test-report.json
```

## Account Structure

```
Mint {
  authority        → Can mint new tokens
  freeze_authority → Can freeze accounts
  supply           → Total in circulation
  decimals         → 0-20
  name, symbol, uri → Metadata
}

TokenAccount {
  owner              → Account owner
  mint               → Associated mint
  balance            → Token amount
  is_frozen          → Transfer locked?
  delegate           → Delegated to...
  delegated_amount   → How much can delegate transfer?
  initialized        → Setup complete?
}
```

## Quick Facts

- **Lines of Code**: 400 (src/token.v)
- **Public Functions**: 19
- **Account Types**: 2
- **E2E Test Duration**: ~30 seconds
- **Test Coverage**: 12+ operations
- **Users in Test**: 3
- **Build Size**: 1.6 KB
- **Average CU Cost**: 10,458 per transaction

## Support

1. **Script help**: `./e2e-token-test.sh --help`
2. **Script guide**: See `SHELL_SCRIPT_USAGE.md`
3. **Test guide**: See `E2E_TEST_README.md`
4. **Function reference**: See `QUICK_START.md`
5. **Full docs**: See `README.md`

## Pro Tips

✨ Use `--skip-build` for faster iteration during development
✨ Run locally first with `./e2e-token-test.sh`, then deploy with `--deploy`
✨ Check compute units with `jq '.summary.avgComputeUnitsPerTx' e2e-test-report.json`
✨ Save baseline report: `cp e2e-test-report.json baseline.json`
✨ Compare runs: `diff baseline.json e2e-test-report.json`

## Next Steps

1. **Read**: `README.md` for overview
2. **Run**: `./e2e-token-test.sh` to test
3. **Explore**: `src/token.v` for implementation details
4. **Customize**: Edit token.v for your needs
5. **Deploy**: `./e2e-token-test.sh --deploy` to localnet
6. **Integrate**: Use program ID in your application
