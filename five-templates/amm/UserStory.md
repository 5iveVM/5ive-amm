# User Story: AMM Template (DEX)

## Objective
Create a production-ready, modular AMM (Automated Market Maker) template that developers can use to bootstrap decentralized exchanges on Five. This template should embody the best practices discovered during the `five-dex-protocol` implementation.

## Features
1.  **Constant Product Swap (`x * y = k`)**:
    - Users can swap Token A for Token B and vice versa.
    - Slippage protection and fee collection (0.3%).
2.  **Liquidity Provision**:
    - `add_liquidity`: Users deposit A and B, receive LP tokens.
    - `remove_liquidity`: Users burn LP tokens, receive A and B.
3.  **Modular Architecture**:
    - Split logic into clear modules: `amm_core.v`, `amm_swap.v`, `amm_math.v`, `amm_types.v`.
    - Set `project.entry_point` in `five.toml` and rely on compiler import discovery.
    - Avoid explicit imports in sub-modules to prevent linker errors.
    - Use `account` keyword (no `pub`) for struct definitions.

## Project Structure
```
five-templates/amm/
├── five.toml                 # Config with entry_point-based discovery
├── src/
│   ├── main.v                # Entry point (imports only)
│   ├── amm_types.v           # Account structs (Pool, LP Token)
│   ├── amm_math.v            # Math helpers (sqrt, price impact)
│   ├── amm_swap.v            # Swap logic (buy/sell)
│   ├── amm_liquidity.v       # Add/Remove liquidity logic
│   └── pool_manager.v        # Admin/Init functions
└── tests/
    └── user_story_amm_test.js # E2E verification script
```

## User Flow
1.  **Initialize Pool**: User calls `initialize_pool(token_a, token_b, fee_bps)`.
2.  **Add Liquidity**: User calls `add_liquidity(amount_a, amount_b)` -> gets LP tokens.
3.  **Swap**: User calls `swap_a_to_b(amount_in, min_out)` -> gets Token B.
4.  **Remove Liquidity**: User calls `remove_liquidity(lp_amount)` -> gets Token A + B + fees.

## Verification Plan
- **Script**: `tests/user_story_amm_test.js`
- **Steps**:
    1.  Deploy `five-amm-template`.
    2.  Init pool with 1000 A / 1000 B.
    3.  Swap 100 A -> Expect ~90 B (incl fee).
    4.  Verify invariant `k` increased.
    5.  Check compute unit usage (Target < 15k).
