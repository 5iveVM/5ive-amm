# User Story: Launchpad Template (Bonding Curve)

## Objective
Create a simple, engaging "Fair Launch" token template using a bonding curve. This template allows developers to build platforms like pump.fun, where tokens originate on a curve and migrate to a DEX upon graduation.

## Features
1.  **Bonding Curve Pricing**:
    - Exponential or Linear curve logic.
    - `buy`: SOL in -> Token out (Price goes UP).
    - `sell`: Token in -> SOL out (Price goes DOWN).
2.  **Fair Launch mechanics**:
    - No pre-mine.
    - `launch_token`: Creates Mint, Curve, and initial supply atomically.
3.  **Modular Architecture**:
    - Logic split: `launchpad_core.v`, `bonding_curve.v`, `token_mint.v`, `token_transfer.v`.
    - No explicit imports in sub-modules.
    - Global namespace linking.

## Project Structure
```
five-templates/launchpad/
├── five.toml                 # Config with entry_point-based discovery
├── src/
│   ├── main.v                # Entry point
│   ├── launchpad_types.v     # Structs (Curve, Mint)
│   ├── bonding_curve.v       # Pricing math
│   ├── launchpad_core.v      # Buy/Sell/Launch logic
│   ├── token_mint.v          # Token minting helpers
│   └── token_transfer.v      # Token transfer helpers
└── tests/
    └── user_story_launchpad_test.js
```

## User Flow
1.  **Launch**: Creator calls `launch_token("MoonCoin", "MOON")`.
    - Deploys Mint.
    - Initializes Bonding Curve with virtual liquidity.
2.  **Trade**:
    - Trader 1 calls `buy_token(1 SOL)`.
    - Trader 1 calls `sell_token(50% balance)`.
3.  **Graduate (Optional)**:
    - If bonding curve fills (~85 SOL), trigger migration (mocked for template).

## Verification Plan
- **Script**: `tests/user_story_launchpad_test.js`
- **Steps**:
    1.  Deploy template.
    2.  Launch Token.
    3.  Buy with 1 SOL -> Check balance increase.
    4.  Sell tokens -> Check SOL return.
    5.  Verify CU usage is efficient (~12k).
