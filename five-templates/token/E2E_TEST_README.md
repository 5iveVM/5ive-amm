# Token Template E2E Test Guide

This e2e test demonstrates the full token lifecycle with 3 users, showing transaction IDs and compute unit consumption.

## Prerequisites

1. **Solana CLI** with localnet configured
2. **Node.js** 18+ with Solana Web3.js installed
3. **Five CLI** with token template built and deployed

## Setup Steps

### 1. Start Localnet

```bash
solana-test-validator
```

### 2. Build Token Template

```bash
cd /Users/amberjackson/Documents/Development/five-org/five-templates/token
five build
```

### 3. Deploy Token Template

```bash
# Using Five CLI
five deploy build/five-token-template.five

# Or using Solana CLI if you have the keypair
solana program deploy build/five-token-template.five
```

This will output your **PROGRAM_ID**.

### 4. Update Configuration in Test

Open `e2e-token-test.mjs` and update these constants:

```javascript
// After deploying, set these values
const FIVE_PROGRAM_ID = new PublicKey('YOUR_PROGRAM_ID_HERE');
const VM_STATE_PDA = new PublicKey('YOUR_VM_STATE_PDA_HERE');
const TOKEN_SCRIPT_ACCOUNT = new PublicKey('YOUR_SCRIPT_ACCOUNT_HERE');
```

The deploy command will show you these values.

### 5. Install Dependencies

```bash
npm install @solana/web3.js
```

### 6. Run Test

```bash
node e2e-token-test.mjs
```

## What the Test Does

### **Setup Phase**
- Creates 3 users (User1=authority, User2=holder, User3=holder)
- Funds each with 2 SOL via airdrop
- Creates token accounts for all users

### **Test Sequence**

1. **init_mint** - User1 creates token with 6 decimals
2. **init_token_account** (×3) - Each user initializes their token account
3. **mint_to** (×3) - User1 mints tokens to each user:
   - User1: 1000 tokens
   - User2: 500 tokens
   - User3: 500 tokens
4. **transfer** - User2 transfers 100 tokens to User3
5. **approve** - User3 approves User2 as delegate for 150 tokens
6. **transfer_from** - User2 transfers 50 tokens from User3 to User1 (as delegate)
7. **revoke** - User3 revokes User2's delegation
8. **burn** - User1 burns 100 tokens (permanent removal from supply)
9. **freeze_account** - User1 freezes User2's account
10. **thaw_account** - User1 unfreezes User2's account
11. **set_mint_authority** - User1 transfers mint authority to User2
12. **disable_mint** - User2 permanently disables minting

### **Output Includes**

For each operation:
- ✅/❌ Success indicator
- Function name and computed operation
- **Compute Units (CU)** consumed
- **Transaction ID (Signature)** for on-chain verification

Final report shows:
- Total tests run
- Success rate
- Total compute units used
- Average CU per transaction
- Min/Max CU values
- Detailed list of all transactions with signatures

## Example Output

```
================================================================================
STEP 2: Initialize Mint (init_mint)
================================================================================

✅ init_mint: 10777 CU
   TX: 3xKq5mP2nL9vQ7rB8jG1kD5mN3pR6sT9vX2yZ4aB5cD6eF7gH8iJ9kL0mN1pQ2rS3

================================================================================
STEP 3: Initialize Token Accounts (init_token_account)
================================================================================

✅ init_token_account (User1): 10753 CU
   TX: 7hQ3nR4sM5tU6vW7xY8zB9cD1eF2gH3iJ4kL5mN6pO7qR8sT9uV0wX1yZ2aB3cD4

✅ init_token_account (User2): 10753 CU
   TX: 8mLp2kQ3rS4tU5vW6xY7zB8cD9eF1gH2iJ3kL4mN5pO6qR7sT8uV9wX0yZ1aB2cD3

...

📊 Test Results Summary

╔════════════════════════════════════════════════════════════════════════════╗
║                    Token Template E2E Test Report                          ║
╠════════════════════════════════════════════════════════════════════════════╣
║                                                                            ║
║  Total Tests Run:                 12                                      ║
║  Successful:                       12                                      ║
║  Failed:                           0                                       ║
║  Success Rate:                     100.0%                                 ║
║                                                                            ║
║  Total Compute Units Used:         125,487                                ║
║  Avg CU per Transaction:           10,458                                 ║
║  Min CU:                           9,234                                  ║
║  Max CU:                           15,892                                 ║
║                                                                            ║
╚════════════════════════════════════════════════════════════════════════════╝

📈 Detailed Results:

  ✅ init_mint                          10777 CU
     3xKq5mP2nL9vQ7rB8jG1kD5mN3pR6sT9vX2yZ4aB5cD6eF7gH8iJ9kL0mN1pQ2rS3

  ✅ init_token_account (User1)         10753 CU
     7hQ3nR4sM5tU6vW7xY8zB9cD1eF2gH3iJ4kL5mN6pO7qR8sT9uV0wX1yZ2aB3cD4

  ✅ init_token_account (User2)         10753 CU
     8mLp2kQ3rS4tU5vW6xY7zB8cD9eF1gH2iJ3kL4mN5pO6qR7sT8uV9wX0yZ1aB2cD3

...

✅ Detailed report saved to ./e2e-test-report.json
```

## JSON Report

The test saves a detailed JSON report to `e2e-test-report.json` containing:

```json
{
  "timestamp": "2025-12-28T06:15:00.000Z",
  "tests": [
    {
      "success": true,
      "functionName": "init_mint",
      "functionIndex": 0,
      "signature": "3xKq5mP2...",
      "computeUnits": 10777,
      "error": null
    },
    ...
  ],
  "summary": {
    "totalTests": 12,
    "successful": 12,
    "failed": 0,
    "successRate": "100.0%",
    "totalComputeUnits": 125487,
    "avgComputeUnitsPerTx": 10458,
    "minCU": 9234,
    "maxCU": 15892
  }
}
```

## Troubleshooting

### "Transaction not found"
- Make sure localnet is running
- Verify RPC_URL is correct (default: http://127.0.0.1:8899)

### "Program not deployed"
- Run `five deploy` first
- Update PROGRAM_ID in test file

### "Account not created"
- Ensure payer keypair has sufficient SOL (~3-5 SOL recommended)
- Check ~/.config/solana/id.json exists

### "Instruction failed"
- Check function indices match your token.v implementation
- Verify account state hasn't changed between tests
- Check compute unit limits (default 1.4M)

## Token Function Indices

| Index | Function | Parameters |
|-------|----------|------------|
| 0 | init_mint | decimals |
| 1 | init_token_account | (none) |
| 2 | mint_to | amount |
| 3 | burn | amount |
| 4 | transfer | amount |
| 5 | transfer_from | amount |
| 6 | approve | amount |
| 7 | revoke | (none) |
| 8 | freeze_account | (none) |
| 9 | thaw_account | (none) |
| 10 | set_mint_authority | (none) |
| 11 | set_freeze_authority | (none) |
| 12 | disable_mint | (none) |
| 13 | disable_freeze | (none) |

## Performance Baseline

Expected compute unit ranges (on localnet):
- Simple operations (revoke, thaw, freeze): 8K-12K CU
- Account initialization: 10K-15K CU
- State mutations (transfer, approve, mint): 12K-20K CU
- Authority changes: 12K-18K CU

Actual values may vary based on:
- Network state
- Account state complexity
- Program optimization level
