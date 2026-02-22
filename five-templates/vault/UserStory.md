# User Story: Native SOL Vault Template

## Objective
Create a simple yet secure "Vault" template where users can deposit and withdraw Native SOL. The vault program tracks individual user balances within the contract state.

## Features
1.  **Deposit SOL**:
    - Users send SOL to the vault.
    - Program updates the user's recorded balance.
2.  **Withdraw SOL**:
    - Users request validation withdrawal.
    - Program checks balance and transfers SOL back to user.
3.  **State Management**:
    - `Vault` account tracks total deposits.
    - `UserAccount` tracks individual user shares/balance.

## Project Structure
```
five-templates/vault/
├── five.toml                 # Config with entry_point-based discovery
├── src/
│   ├── main.v                # Entry point
│   ├── vault_types.v         # Vault and User structs
│   └── vault_logic.v         # Deposit/Withdraw logic
└── tests/
    └── user_story_vault_test.js
```

## User Flow
1.  **Initialize**: Admin initializes the global `Vault` account.
2.  **Deposit**: User calls `deposit(amount)`.
    - 5 SOL is transferred from User (signer) to Vault (PDA).
    - User's internal balance += 5 SOL.
3.  **Withdraw**: User calls `withdraw(amount)`.
    - Check internal balance >= amount.
    - Transfer amount from Vault (PDA) to User.
    - User's internal balance -= amount.

## Verification
- **Test Script**: `tests/user_story_vault_test.js`
- **Steps**:
    1.  Init Vault.
    2.  Deposit 10 SOL.
    3.  Check Vault SOL Balance == 10.
    4.  Withdraw 5 SOL.
    5.  Check User Wallet Balance increased.
