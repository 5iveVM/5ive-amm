# Five Token Examples - Index

Complete token and AMM swap implementations for the Five Protocol.

## 📦 What's Included

This directory contains a production-ready token system and DEX AMM implementation in Five DSL.

### 1. **five_native_token.v** (352 lines)
A complete fungible token implementation with all standard token operations.

**Features:**
- Mint creation and management with authority controls
- Token account creation and initialization
- Token minting (create new supply)
- Token burning (destroy supply)
- Direct transfers between accounts
- Delegated transfers (approval-based spending)
- Account freezing/thawing (emergency controls)
- Authority management (transfer control)
- Query functions for token state

**Use Cases:**
- Custom token creation
- Token distribution systems
- Stablecoin implementation
- Governance tokens
- Community currencies

---

### 2. **five_spl_token_amm.v** (503 lines)
An Automated Market Maker with constant product formula and SPL token integration.

**Features:**
- Constant product AMM (x*y=k) with configurable fees
- Liquidity provider system with LP share tracking
- Token swaps with slippage protection
- LP token minting for share tracking
- SPL Token CPI integration for cross-token operations
- Price quotes and pool information
- Invariant checking and validation
- Mint, transfer, and burn SPL tokens via CPI

**Use Cases:**
- Native Five <-> SPL token swaps
- Decentralized exchange pools
- Liquidity provision and yield farming
- Price discovery mechanisms
- Token bridging systems

---

## 📚 Documentation

### README.md (400 lines)
**Complete architecture and design documentation**

Contains:
- Detailed feature explanations
- Account type descriptions
- All 40+ function definitions
- Code examples and patterns
- AMM formula explanations
- SPL integration guide
- Comparison with SPL Token
- Extension possibilities

### USAGE_EXAMPLES.md (649 lines)
**Real-world practical scenarios**

Demonstrates:
- Part 1: Native Token Operations (6 scenarios)
- Part 2: AMM Swap Operations (8 scenarios)
- Part 3: Cross-Program Integration (3 scenarios)
- Part 4: Error Handling (4 scenarios)
- Part 5: Security Considerations (4 scenarios)
- Deployment checklist
- Troubleshooting guide

### QUICK_REFERENCE.md (359 lines)
**Fast lookup guide for developers**

Includes:
- Function tables (20+ functions)
- Common patterns and snippets
- AMM mathematical formulas
- Error message reference
- Decimal handling guide
- Fee structure reference
- Typical workflows
- Security checklist
- Compilation commands

---

## 🚀 Quick Start

### Compile

```bash
# Compile native token
five compile five_native_token.v

# Compile AMM
five compile five_spl_token_amm.v
```

### Test Locally

```bash
# Test token functions
five local execute five_native_token.v 0

# Test AMM functions
five local execute five_spl_token_amm.v 0
```

### Deploy On-Chain

```bash
# Deploy token to devnet
five deploy five_native_token.v --network devnet

# Deploy AMM to devnet
five deploy five_spl_token_amm.v --network devnet
```

---

## 📊 Statistics

| Item | Count |
|------|-------|
| Total Lines of Code | 2,263 |
| Code Files (.v) | 2 |
| Documentation Files (.md) | 4 |
| Token Functions | 30+ |
| AMM Functions | 20+ |
| Example Scenarios | 25+ |
| Account Types | 4 |

---

## 🔍 File Structure

```
11-token-examples/
├── five_native_token.v      # Native token implementation
├── five_spl_token_amm.v     # AMM with SPL integration
├── README.md                # Complete documentation
├── USAGE_EXAMPLES.md        # Practical examples
├── QUICK_REFERENCE.md       # Developer reference
└── INDEX.md                 # This file
```

---

## 🎯 Key Features

### Native Token
- ✅ Full token lifecycle management
- ✅ Mint and authority controls
- ✅ Account freezing for emergency stops
- ✅ Delegation support (approve pattern)
- ✅ Configurable decimals
- ✅ Supply tracking and burning
- ✅ Safe transfer guards

### AMM
- ✅ Constant product formula (x*y=k)
- ✅ Configurable fee structure
- ✅ LP share tracking
- ✅ Liquidity addition/removal
- ✅ Slippage protection
- ✅ Price quoting
- ✅ Invariant validation
- ✅ SPL token CPI calls
- ✅ Spot price calculation

---

## 🔐 Security Features

- Account ownership verification on all operations
- Balance checks before transfers
- Authority validation for privileged operations
- Frozen account detection
- Invariant checking in AMM
- Slippage tolerance enforcement
- Delegation amount limits
- Safe arithmetic operations

---

## 💡 Design Patterns

### Solana Account Model
Both scripts follow Solana's native account model:
- One mint per token type
- Individual token accounts per user
- Authority-based access control
- PDA support for derived addresses

### Constant Product AMM
The AMM uses the proven constant product formula:
- `x * y = k` (invariant)
- Fair pricing without oracle
- Liquidity provider rewards via fees
- Slippage increases with trade size

---

## 📖 Documentation Guide

**Choose your path based on needs:**

1. **Learning the Architecture?**
   → Start with [README.md](README.md)

2. **Building a Specific Feature?**
   → Check [USAGE_EXAMPLES.md](USAGE_EXAMPLES.md) for your use case

3. **Need Function Reference?**
   → Use [QUICK_REFERENCE.md](QUICK_REFERENCE.md)

4. **Building Something New?**
   → Read README.md → Copy patterns from USAGE_EXAMPLES.md

---

## 🔗 Related Resources

- **[Five DSL Programming Guide](../../FIVE_DSL_PROGRAMMING_GUIDE.md)** - Language documentation
- **[Five CLI Manual](../../../five-cli/CLAUDE.md)** - Compilation and deployment
- **[SPL Token Program Docs](https://docs.rs/spl-token/)** - SPL token standard
- **[Uniswap V2 Whitepaper](https://uniswap.org/whitepaper.pdf)** - AMM theory

---

## 🛠️ Example Code Snippets

### Create and Mint a Token (5 lines)
```v
let mint = init_mint(mint_acct, auth, freeze_auth, 8, "MyToken");
let acct = init_token_account(token_acct, owner, mint);
mint_to(mint, acct, auth, 1_000_000_000);
let balance = get_balance(acct);  // 1_000_000_000
```

### Swap with Slippage Protection (3 lines)
```v
let quote = quote_swap_a_to_b(pool, amount);
let min_out = (quote * 98) / 100;  // 2% slippage
swap_a_to_b(pool, amount, min_out);
```

### Provide Liquidity (2 lines)
```v
let shares = add_liquidity(pool, lp, owner, amt_a, amt_b, 0);
// Later: let (back_a, back_b) = remove_liquidity(pool, lp, owner, shares, 0, 0);
```

---

## ✅ Validation Checklist

All scripts include:

- [x] Proper account constraint handling
- [x] Authority validation
- [x] Balance verification
- [x] Mint/account consistency checks
- [x] Frozen account detection
- [x] Slippage protection mechanisms
- [x] Invariant checking (AMM)
- [x] Safe arithmetic operations
- [x] Comprehensive error messages
- [x] Clear documentation

---

## 🚢 Production Readiness

These implementations are suitable for:
- ✅ Learning Five DSL token development
- ✅ Building on top of in educational environments
- ✅ Testing AMM mechanics
- ✅ Reference implementations

For production mainnet use:
- ⚠️ Conduct security audits
- ⚠️ Add oracle integration if needed
- ⚠️ Implement governance mechanisms
- ⚠️ Add emergency pause functions
- ⚠️ Consider multi-sig authorities

---

## 💬 Key Takeaways

1. **Token Development**: Follows Solana's established patterns
2. **AMM Design**: Uses proven constant product formula
3. **Type Safety**: Full compile-time checking via Five DSL
4. **Ultra-Lightweight**: Compiles to minimal bytecode
5. **Production Patterns**: Real Solana architecture integration

---

## 📝 Notes

- Scripts use Five DSL syntax (Rust-like)
- Account constraints: `@mut`, `@init`, `@signer`
- All functions include comprehensive documentation
- Error messages are descriptive and helpful
- Examples cover common and advanced use cases

---

**Created:** December 11, 2025
**Five Protocol Version:** Latest
**Status:** Production-Ready Examples

For questions or improvements, refer to the Five documentation and Five Protocol community resources.
