# Token Template - Quick Start

## Build & Test Token Template

```bash
cd /Users/amberjackson/Documents/Development/five-org/five-templates/token

# Build the template
five build

# Expected output:
# OK build (2 files, 1588 bytes)
```

## Files Structure

```
token/
├── src/
│   ├── token.v          # Complete SPL Token implementation (400 lines, 19 functions)
│   └── main.v          # Entry point
├── build/
│   └── five-token-template.five  # Compiled bytecode
├── five.toml            # Project configuration
├── e2e-token-test.mjs   # E2E test with 3 users
├── E2E_TEST_README.md   # Detailed test guide
└── QUICK_START.md       # This file
```

## Token Functions (19 total)

### Initialization (4)
- `init_mint()` - Create token mint
- `init_token_account()` - Create token account
- `create_associated_token_account()` - ATA helper
- `create_deterministic_token_account()` - PDA-based account

### Minting & Burning (2)
- `mint_to()` - Mint tokens
- `burn()` - Destroy tokens

### Transfers (2)
- `transfer()` - Direct transfer
- `transfer_from()` - Delegated transfer

### Delegation (2)
- `approve()` - Grant delegation
- `revoke()` - Revoke delegation

### Freezing (2)
- `freeze_account()` - Freeze account
- `thaw_account()` - Unfreeze account

### Authority Management (4)
- `set_mint_authority()` - Change mint authority
- `set_freeze_authority()` - Change freeze authority
- `disable_mint()` - Permanently disable minting
- `disable_freeze()` - Permanently disable freezing

### Validation (3)
- `verify_account_mint()` - Check mint match
- `verify_account_owner()` - Check owner
- `verify_account_full()` - Check both

## E2E Test Overview

The `e2e-token-test.mjs` file tests a complete workflow with 3 users:

**Setup:**
- User1 (Authority) - Creates mint, mints tokens
- User2 (Holder) - Receives and transfers tokens
- User3 (Holder) - Receives tokens, delegates

**Test Flow:**
1. Initialize mint (6 decimals)
2. Initialize token accounts (×3)
3. Mint tokens:
   - User1: 1000 tokens
   - User2: 500 tokens
   - User3: 500 tokens
4. User2 → User3: Transfer 100 tokens
5. User3 approves User2 as delegate for 150 tokens
6. User2 transfers 50 tokens from User3 to User1 (delegated)
7. User3 revokes User2's delegation
8. User1 burns 100 tokens
9. User1 freezes User2's account
10. User1 unfreezes User2's account
11. User1 transfers mint authority to User2
12. User2 disables minting (irreversible)

**Output Includes:**
- Transaction signatures (for on-chain verification)
- Compute units per operation
- Success/failure status
- Detailed report (JSON file)

## Features

✅ **19 Public Functions**
✅ **Overflow Protection** - u64 overflow checks
✅ **Delegation System** - Approve/revoke with amounts
✅ **Freeze Authority** - Emergency freeze/thaw
✅ **Authority Management** - Mint & freeze control
✅ **ATA Support** - Associated Token Accounts
✅ **PDA Support** - Deterministic accounts
✅ **Disable Functions** - Permanently disable authorities

## Key Implementation Details

### Account Types

```five
account Mint {
    authority: pubkey;           // Minting authority
    freeze_authority: pubkey;    // Freeze authority
    supply: u64;                 // Total supply
    decimals: u8;                // Decimal places
    name: string;                // Token name
    symbol: string;              // Token symbol
    uri: string;                 // Metadata URI
}

account TokenAccount {
    owner: pubkey;               // Account owner
    mint: pubkey;                // Associated mint
    balance: u64;                // Token balance
    is_frozen: bool;             // Freeze status
    delegated_amount: u64;       // Delegated amount
    delegate: pubkey;            // Delegate address
    initialized: bool;           // Init flag
}
```

### Core Operations

```five
// Mint tokens with overflow protection
pub mint_to(mint_state: Mint @mut, destination: TokenAccount @mut, authority: account @signer, amount: u64) {
    require(mint_state.authority == authority.key);
    require(destination.mint == mint_state.key);
    require(!destination.is_frozen);
    require(amount > 0);
    require(mint_state.supply <= 9223372036854775807 - amount);
    require(destination.balance <= 18446744073709551615 - amount);
    mint_state.supply = mint_state.supply + amount;
    destination.balance = destination.balance + amount;
}

// Transfer with delegation support
pub transfer_from(source: TokenAccount @mut, dest: TokenAccount @mut, authority: account @signer, amount: u64) {
    let is_owner = source.owner == authority.key;
    if (!is_owner) {
        require(source.delegate == authority.key);
        require(source.delegated_amount >= amount);
    }
    require(source.balance >= amount);
    require(source.mint == dest.mint);
    require(!source.is_frozen && !dest.is_frozen);
    if (!is_owner) {
        source.delegated_amount = source.delegated_amount - amount;
    }
    source.balance = source.balance - amount;
    dest.balance = dest.balance + amount;
}

// Disable minting permanently
pub disable_mint(mint_state: Mint @mut, authority: account @signer) {
    require(mint_state.authority == authority.key);
    mint_state.authority = 0;  // No one can mint after this
}
```

## Building Blocks

This token template provides the foundation for:
- ERC20-like tokens on Solana
- Memcoins with authority management
- Governance tokens with delegation
- Stablecoins with freeze capabilities
- Any token with SPL-compatible operations

## SPL Token Compatibility

This implementation covers the **core SPL Token operations**:
- ✅ Mint creation & management
- ✅ Account initialization
- ✅ Token minting
- ✅ Token burning
- ✅ Transfers
- ✅ Delegation (approve/revoke)
- ✅ Account freezing
- ✅ Authority management

**Not included** (out of scope for single-file template):
- Multi-signature operations
- Mint extensions (transfer fee, interest-bearing)
- Metadata management
- Confidential transfers

## Performance

Expected compute unit costs (localnet):
- Simple operations: 8K-12K CU
- Account initialization: 10K-15K CU
- State mutations: 12K-20K CU
- Authority changes: 12K-18K CU

## Next Steps

1. **Deploy**: `five deploy build/five-token-template.five`
2. **Test**: Update `e2e-token-test.mjs` with program ID, then run
3. **Integrate**: Use in your application
4. **Customize**: Modify `token.v` for specific requirements

## Reference

- **Full Source**: `src/token.v` (400 lines)
- **E2E Test**: `e2e-token-test.mjs` (Node.js)
- **Test Guide**: `E2E_TEST_README.md`
- **Config**: `five.toml`

## Support

For issues or questions:
- Check `E2E_TEST_README.md` for troubleshooting
- Review `src/token.v` for implementation details
- Look at test output for specific errors
