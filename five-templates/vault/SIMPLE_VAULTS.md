# Simple Vault Templates

Three identical vault implementations - the only difference is the token type they manage.

---

## Quick Summary

| Vault | File | Token Type | Size | Functions |
|-------|------|-----------|------|-----------|
| **Native SOL** | `main.v` | Native lamports | 654 B | init_vault, init_user, deposit, withdraw |
| **SPL Token** | `spl_token_vault.v` | Any SPL token | 668 B | init_vault, init_user, deposit, withdraw |
| **Five Token** | `five_token_vault.v` | Defined token types | 686 B | init_vault, init_user, deposit, withdraw |

---

## Architecture

All three vaults follow the exact same pattern:

```
┌─────────────────────────────────────────┐
│         Vault Program                   │
│                                         │
│  1. init_vault()     - Setup            │
│  2. init_user()      - Create account   │
│  3. deposit()        - Add tokens       │
│  4. withdraw()       - Remove tokens    │
└─────────────────────────────────────────┘
         ↓
    [CPI Call]
         ↓
┌─────────────────────────────────────────┐
│  External Program (System or SPLToken)  │
│  Actually transfers tokens              │
└─────────────────────────────────────────┘
```

---

## State Structure

All three use the same account pattern:

```
Vault {
  authority: pubkey;      // Who can manage the vault
  token_mint: pubkey;     // (SOL vaults skip this)
  total_deposited: u64;   // Total assets in vault
}

UserAccount {
  owner: pubkey;          // Account owner
  vault: pubkey;          // Which vault they use
  balance: u64;           // How much they deposited
}
```

---

## Function Signatures

All three have identical function signatures, only differing by:
1. Account type names (UserAccount vs UserTokenAccount)
2. Vault type names (Vault vs TokenVault)
3. CPI program (SystemProgram vs SPLToken)

### Initialize Functions
```five
init_vault(vault: Vault, authority, [token_mint?])
init_user(user_account: UserAccount, vault, owner)
```

### Deposit Function
```five
deposit(
  vault: Vault,
  user_account: UserAccount,
  [source_token_account?],      // Only for token vaults
  [vault_token_account?],       // Only for token vaults
  signer: account,
  amount: u64
)
```

### Withdraw Function
```five
withdraw(
  vault: Vault,
  user_account: UserAccount,
  [vault_token_account?],       // Only for token vaults
  [destination_token_account?], // Only for token vaults
  signer: account,
  amount: u64
)
```

---

## Comparison

### Native SOL Vault (`main.v`)
**Uses**: SystemProgram to transfer native lamports

**Key Differences**:
- No token_mint (native SOL only)
- Takes vault PDA accounts directly
- Signer is the payer/withdrawer

**Example Flow**:
```
User's Wallet → (SystemProgram.transfer) → Vault PDA
User must be @signer and @mut
```

### SPL Token Vault (`spl_token_vault.v`)
**Uses**: SPLToken Program to transfer any SPL token

**Key Differences**:
- Takes token_mint parameter
- Works with any SPL token
- Requires token account addresses

**Example Flow**:
```
User's Token Account → (SPLToken.transfer) → Vault Token Account
User must be @signer (authority)
```

### Five Token Vault (`five_token_vault.v`)
**Uses**: SPLToken Program + imports Five DSL token types

**Key Differences**:
- Imports token.v for type definitions
- Same as SPL Token Vault functionally
- Shows how to structure multi-file projects

**Example Flow**:
```
User's Token Account → (SPLToken.transfer) → Vault Token Account
Imports Mint and TokenAccount types from token.v
```

---

## Token Types

### Native SOL
Built-in, no special definition needed.

### SPL Token
```five
account TokenVault {
    authority: pubkey;
    token_mint: pubkey;
    total_deposited: u64;
}
```

### Five Token (from `token.v`)
```five
account Mint {
    authority: pubkey;
    supply: u64;
    decimals: u8;
}

account TokenAccount {
    owner: pubkey;
    mint: pubkey;
    balance: u64;
}
```

---

## CPI Interfaces

All use standard Solana program interfaces:

### SystemProgram (Native SOL)
```five
interface SystemProgram @program("11111111111111111111111111111112") {
    transfer @discriminator(2) (from: pubkey, to: pubkey, amount: u64);
}
```

### SPLToken Program (Token Vaults)
```five
interface SPLToken @program("TokenkegQfeZyiNwAJsyFbPVwwQQfzzTtKF2WwZvD") {
    transfer @discriminator(3) (source: pubkey, destination: pubkey, authority: pubkey, amount: u64);
}
```

---

## Validation

All three vaults do basic validation:

```five
require(amount > 0);                      // Amount must be positive
require(user_account.balance >= amount);  // User has enough (withdraw only)
```

---

## Compilation Sizes

```
Native SOL:   654 bytes
SPL Token:    668 bytes
Five Token:   686 bytes

Total:      2,008 bytes
```

All extremely lightweight, under 700 bytes each.

---

## How to Choose

### Use Native SOL if:
- You're only managing SOL
- You want the smallest bytecode
- You need the simplest implementation

### Use SPL Token if:
- You're managing any SPL token (USDC, USDT, etc.)
- You need generic token support
- You want a self-contained vault

### Use Five Token if:
- You want to use Five DSL token type definitions
- You're building a larger system with shared token types
- You prefer importing type definitions

---

## Deployment Example

```bash
# Compile
five compile src/main.v --output build/sol_vault.five
five compile src/spl_token_vault.v --output build/token_vault.five
five compile src/five_token_vault.v --output build/five_token_vault.five

# Deploy (all same process)
five deploy build/sol_vault.five --program <PROGRAM_ID>
five deploy build/token_vault.five --program <PROGRAM_ID>
five deploy build/five_token_vault.five --program <PROGRAM_ID>
```

---

## Key Takeaway

These three vaults are **functionally identical** - just operating on different token types:
- **main.v** = Native SOL vault
- **spl_token_vault.v** = SPL Token vault (works with any token)
- **five_token_vault.v** = SPL Token vault using Five DSL types

Choose based on your token type and architectural preferences. All are production-ready with ~650 bytes each.
