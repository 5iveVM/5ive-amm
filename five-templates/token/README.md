# Token Template

A comprehensive, production-ready SPL Token implementation in Five DSL with complete E2E testing infrastructure.

## Overview

This template provides everything needed to create and test a fully-featured token on Solana using the Five VM:

- **19 public functions** covering all core token operations
- **Single-file implementation** (~400 lines of Five DSL)
- **Automated E2E tests** with 3 users and compute unit tracking
- **Shell script automation** for build, deploy, and test pipeline
- **Detailed documentation** for setup, usage, and troubleshooting

## Quick Start

### One-Command Testing

```bash
./run-runtime-fixtures.sh              # Fast validator-free runtime fixture test
./e2e-token-test.sh                    # Build and test locally
./e2e-token-test.sh --deploy           # Build, deploy to localnet, and test
./e2e-token-test.sh --help             # Show all options
```

### What's Tested

- Mint creation with authority
- Account initialization
- Token minting (supply management)
- Transfers (direct and delegated)
- Delegation (approve/revoke)
- Account freezing/thawing
- Authority management and disabling

## Files

### Core Implementation

| File | Purpose | Lines |
|------|---------|-------|
| `src/token.v` | Complete SPL Token implementation | 400 |
| `src/main.v` | Entry point | 8 |
| `five.toml` | Build configuration | - |

### Testing

| File | Purpose |
|------|---------|
| `run-runtime-fixtures.sh` | Run token runtime fixture through five-solana harness (no validator) |
| `e2e-token-test.sh` | **Automated shell script runner** (easiest to use) |
| `e2e-token-test.mjs` | Node.js E2E test with 3 users, transaction IDs, CU tracking |
| `e2e-test-report.json` | Generated test results |

### Documentation

| File | Purpose |
|------|---------|
| `README.md` | This file - overview and quick reference |
| `QUICK_START.md` | Quick reference for token functions |
| `SHELL_SCRIPT_USAGE.md` | Complete shell script documentation |
| `E2E_TEST_README.md` | Detailed E2E test guide |

## Architecture

### Account Types

```
Mint Account:
  - authority: Minting authority
  - freeze_authority: Account freeze authority
  - supply: Total tokens in circulation
  - decimals: Decimal places (0-20)
  - name, symbol, uri: Token metadata

Token Account:
  - owner: Account owner
  - mint: Associated mint
  - balance: Token balance
  - is_frozen: Frozen status
  - delegate: Delegated authority
  - delegated_amount: Delegation amount
  - initialized: Init flag
```

### Operations (19 total)

**Initialization (4)**
- `init_mint()` - Create token mint
- `init_token_account()` - Create balance account
- `create_associated_token_account()` - ATA helper
- `create_deterministic_token_account()` - PDA-based account

**Transfers (4)**
- `transfer()` - Direct transfer
- `transfer_from()` - Transfer using delegation
- `approve()` - Grant delegation
- `revoke()` - Revoke delegation

**Mint Management (2)**
- `mint_to()` - Mint tokens with overflow protection
- `burn()` - Destroy tokens

**Account Control (2)**
- `freeze_account()` - Freeze account
- `thaw_account()` - Unfreeze account

**Authority Management (4)**
- `set_mint_authority()` - Change mint authority
- `set_freeze_authority()` - Change freeze authority
- `disable_mint()` - Permanently disable minting
- `disable_freeze()` - Permanently disable freezing

**Validation (3)**
- `verify_account_mint()` - Check mint match
- `verify_account_owner()` - Check owner
- `verify_account_full()` - Check both

## Building & Testing

### Using Shell Script (Recommended)

```bash
# Build and test
./e2e-token-test.sh

# Build, deploy, and test
./e2e-token-test.sh --deploy

# Verbose output
./e2e-token-test.sh --verbose

# Clean artifacts
./e2e-token-test.sh --clean

# Skip build
./e2e-token-test.sh --skip-build
```

See `SHELL_SCRIPT_USAGE.md` for complete options and examples.

### Manual Build & Test

```bash
# Build
five build

# Deploy (optional)
five deploy build/five-token-template.five

# Test
npm install @solana/web3.js
node e2e-token-test.mjs
```

## Test Scenario

The E2E test demonstrates a realistic workflow with 3 users:

| User | Role | Actions |
|------|------|---------|
| User1 | Authority | Creates mint, mints tokens, manages authorities |
| User2 | Holder | Receives tokens, transfers, acts as delegate |
| User3 | Holder | Receives tokens, delegates, transfers |

**Operations Tested (12 steps):**
1. Initialize mint
2. Initialize token accounts (×3)
3. Mint tokens (×3)
4. Transfer tokens
5. Approve delegation
6. Transfer as delegate
7. Revoke delegation
8. Burn tokens
9. Freeze account
10. Thaw account
11. Change mint authority
12. Disable minting

**Output Includes:**
- ✓ Transaction signatures (for on-chain verification)
- ✓ Compute units per operation
- ✓ Success/failure indicators
- ✓ JSON report with detailed metrics

Example output:
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
   Avg CU: 10,458 per transaction
```

## Features

### Security
- ✅ Authority validation on all operations
- ✅ Overflow protection on supply and balance
- ✅ Account freezing for emergency stops
- ✅ Ownership verification
- ✅ Delegation amount tracking

### Functionality
- ✅ Token minting with supply management
- ✅ Token burning (permanent removal)
- ✅ Direct transfers
- ✅ Delegation system (approve/revoke)
- ✅ Account freezing/thawing
- ✅ Authority transfer
- ✅ Permanent disabling (minting/freezing)
- ✅ Associated Token Accounts (ATA)
- ✅ PDA-based accounts

### Operations
- ✅ Variable decimal places (0-20)
- ✅ Token metadata (name, symbol, URI)
- ✅ Separate mint and freeze authorities
- ✅ Delegated transfers with amount tracking
- ✅ Frozen account checks

## Requirements

### System
- Solana CLI (https://docs.solana.com/cli/install-solana-cli-tools)
- Five CLI (https://github.com/five-protocol/five-cli)
- Node.js 18+ (https://nodejs.org/)
- Solana test validator (for deployment)

### Npm Packages
```bash
npm install @solana/web3.js
```

## Performance

Typical compute unit costs (localnet):
- Simple operations (revoke, thaw): 8K-12K CU
- Account initialization: 10K-15K CU
- State mutations (transfer, mint): 12K-20K CU
- Authority changes: 12K-18K CU

**Average transaction: ~10.5K CU**

## File Structure

```
token/
├── src/
│   ├── token.v              # 400-line SPL Token implementation
│   └── main.v               # Entry point
├── build/
│   └── five-token-template.five  # Compiled bytecode
├── e2e-token-test.sh        # Automated test runner
├── e2e-token-test.mjs       # E2E test code (Node.js)
├── e2e-test-report.json     # Generated test results
│
├── five.toml                # Build configuration
├── README.md                # This file
├── QUICK_START.md           # Function reference
├── SHELL_SCRIPT_USAGE.md    # Script documentation
├── E2E_TEST_README.md       # Test documentation
└── ARCHITECTURE.md          # Original architecture doc
```

## Getting Started

### 1. Test Locally

```bash
cd /Users/amberjackson/Documents/Development/five-org/five-templates/token
./e2e-token-test.sh
```

Expected output shows all operations succeeding with compute unit costs.

### 2. Deploy to Localnet

```bash
# Start localnet if not running
solana-test-validator &

# Build and deploy
./e2e-token-test.sh --deploy

# Update program IDs in test file if deploying fresh
```

### 3. Customize

Edit `src/token.v` to modify:
- Token name/symbol defaults
- Decimal places limits
- Validation rules
- Authority logic

Then rebuild: `./e2e-token-test.sh`

## Integration

Use this token template as:

1. **Reference implementation** for SPL Token operations
2. **Foundation** for custom token features (taxes, hooks, etc.)
3. **Testing base** for DEX, lending, or other protocols
4. **Educational material** for Five DSL patterns

## SPL Compatibility

Implements core SPL Token spec:
- ✅ Mint creation and management
- ✅ Account initialization
- ✅ Token minting
- ✅ Token burning
- ✅ Transfers
- ✅ Delegation
- ✅ Account freezing
- ✅ Authority management

**Not included:**
- Multi-signature operations
- Mint extensions (transfer fees, interest)
- Metadata program integration
- Confidential transfers

## Support & Troubleshooting

### Common Issues

**Build fails**
```bash
./e2e-token-test.sh --verbose  # See error details
```

**Test timeouts**
```bash
# Check validator is running
solana cluster-info

# Kill and restart if needed
pkill solana-test-validator
solana-test-validator
```

**Script not executable**
```bash
chmod +x e2e-token-test.sh
```

For more help, see:
- `SHELL_SCRIPT_USAGE.md` - Script troubleshooting
- `E2E_TEST_README.md` - Test troubleshooting
- `QUICK_START.md` - Function reference

## Example: Using the Token

```five
// In your program, you can use these functions:

// Create mint
let mint_key = init_mint(
    mint_account,
    authority,
    freeze_authority,
    6,                    // 6 decimals
    "MyToken",           // name
    "MYTKN",            // symbol
    "https://..."       // uri
);

// Create account
let account_key = init_token_account(
    token_account,
    user.publicKey,
    mint_key
);

// Mint tokens
mint_to(mint, token_account, authority, 1000);

// Transfer
transfer(from_account, to_account, owner, 100);

// Approve delegation
approve(from_account, owner, delegate, 50);

// Transfer as delegate
transfer_from(from_account, to_account, delegate, 25);

// Freeze account
freeze_account(mint, account, freeze_authority);

// Disable minting (irreversible!)
disable_mint(mint, authority);
```

## Advanced Usage

See individual documentation files:
- `QUICK_START.md` - Function parameters and examples
- `E2E_TEST_README.md` - Running tests manually
- `SHELL_SCRIPT_USAGE.md` - Script options and integration

## License

This token template is part of the Five Protocol project.

## See Also

- Five CLI: https://github.com/five-protocol/five-cli
- Five VM: https://github.com/five-protocol/five-vm
- Solana Docs: https://docs.solana.com/
- SPL Token: https://github.com/solana-labs/solana-program-library
