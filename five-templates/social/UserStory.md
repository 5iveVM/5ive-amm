# User Story: Social Prediction Template

## Objective
Create a "Social Sentiment" or "Prediction Market" template where users can vote/bet on binary outcomes (Yes/No, Up/Down). This serves as a foundation for prediction markets, voting DAOs, or social games.

## Features
1.  **Binary Markets**:
    - `create_market`: Define a question and resolution time.
2.  **Voting/Betting**:
    - `vote_yes(amount)` / `vote_no(amount)`.
    - State tracking: Total YES vs Total NO shares.
3.  **Resolution**:
    - `resolve_market(outcome)`: Admin/Oracle sets winner.
    - `claim_winnings()`: Winners withdraw proportional share of the losing pool.
4.  **Simplicity**:
    - Focus on clear logic and state management.
    - Modular file structure.

## Project Structure
```
five-templates/social/
├── five.toml
├── src/
│   ├── main.v
│   ├── social_types.v        # Market struct, Vote struct
│   ├── market_logic.v        # Create, Resolve
│   └── voting_logic.v        # Vote, Claim
└── tests/
    └── user_story_social_test.js
```

## User Flow
1.  **Create**: User creates "Will SOL hit $200 by Friday?".
2.  **Vote**:
    - User A puts 100 USDC on YES.
    - User B puts 100 USDC on NO.
3.  **Resolve**: Admin resolves YES.
4.  **Claim**: User A claims 200 USDC (their 100 + User B's 100).

## Verification Plan
- **Script**: `tests/user_story_social_test.js`
- **Steps**:
    1.  Create Market.
    2.  Vote YES / Vote NO.
    3.  Check odds calculation.
    4.  Resolve and Claim.
    5.  Verify balance transfers.
