//script LendingPlatform {
    // --- Accounts ---
    // LendingPoolAccount: Stores global state of the lending pool
    account LendingPoolAccount {
        collateral_mint: pubkey; // Mint address of the collateral token
        borrow_mint: pubkey;     // Mint address of the borrow token
        total_collateral_deposited: u64;
        total_borrowed_amount: u64;
        interest_rate_bps: u64;  // Interest rate in basis points (e.g., 100 for 1%)
        liquidation_threshold_bps: u64; // Liquidation threshold in basis points (e.g., 150 for 150% collateral)
    }

    // UserLoanAccount: Stores individual user loan details
    account UserLoanAccount {
        user: pubkey;
        collateral_amount: u64;
        borrowed_amount: u64;
        loan_start_time: u64; // Unix timestamp when the loan was taken
        is_active: bool;
    }

    // --- Events ---
    event DepositEvent {
        user: pubkey;
        amount: u64;
    }

    event BorrowEvent {
        user: pubkey;
        amount: u64;
        collateral_used: u64;
    }

    event RepayEvent {
        user: pubkey;
        amount: u64;
    }

    event WithdrawEvent {
        user: pubkey;
        amount: u64;
    }

    // --- Interfaces (for CPI with SPL Token Program) ---
    // Assuming a simplified SPL Token Program interface for transfer
    interface SplTokenProgram program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5mW") {
        // Transfer instruction
        // function transfer(source: Account, destination: Account, authority: Account, amount: u64);
        // Note: Actual SPL Token program has more complex instruction data and accounts.
        // This is a conceptual representation for DSL.
        function transfer(discriminator(3) source: pubkey, destination: pubkey, authority: pubkey, amount: u64);
    }

    // --- Instructions ---

    // Initialize the lending pool
    instruction initialize_lending_pool(
        mut lending_pool: LendingPoolAccount,
        collateral_mint: pubkey,
        borrow_mint: pubkey,
        interest_rate_bps: u64,
        liquidation_threshold_bps: u64
    ) {
        require(lending_pool.collateral_mint == pubkey::new_from_array([0; 32]), "Lending pool already initialized");

        lending_pool.collateral_mint = collateral_mint;
        lending_pool.borrow_mint = borrow_mint;
        lending_pool.total_collateral_deposited = 0;
        lending_pool.total_borrowed_amount = 0;
        lending_pool.interest_rate_bps = interest_rate_bps;
        lending_pool.liquidation_threshold_bps = liquidation_threshold_bps;
    }

    // Deposit collateral into the lending pool
    instruction deposit_collateral(
        @signer user: Account,
        mut user_collateral_token_account: Account, // User's token account for collateral
        mut lending_pool_collateral_token_account: Account, // Lending pool's token account for collateral
        mut lending_pool: LendingPoolAccount,
        amount: u64
    ) {
        require(amount > 0, "Deposit amount must be greater than 0");

        // Transfer collateral from user to lending pool
        // This is a placeholder for a real CPI call to the SPL Token program
        // The actual CPI would involve passing the correct accounts and instruction data.
        // SplTokenProgram.transfer(user_collateral_token_account.key, lending_pool_collateral_token_account.key, user.key, amount);
        // Simulate the state change.
        lending_pool.total_collateral_deposited = lending_pool.total_collateral_deposited + amount;

        emit DepositEvent { user: user.key, amount: amount };
    }

    // Borrow tokens from the lending pool
    instruction borrow(
        @signer user: Account,
        mut user_borrow_token_account: Account, // User's token account for borrowed asset
        mut lending_pool_borrow_token_account: Account, // Lending pool's token account for borrowed asset
        mut user_loan: UserLoanAccount,
        mut lending_pool: LendingPoolAccount,
        borrow_amount: u64,
        collateral_deposited: u64 // Amount of collateral user has already deposited (for calculation)
    ) {
        require(borrow_amount > 0, "Borrow amount must be greater than 0");
        require(user_loan.is_active == false, "User already has an active loan");

        // Calculate max borrowable amount based on collateral and liquidation threshold
        // collateral_deposited * (10000 / liquidation_threshold_bps)
        let max_borrow_amount = collateral_deposited * 10000 / lending_pool.liquidation_threshold_bps;
        require(borrow_amount <= max_borrow_amount, "Insufficient collateral for this borrow amount");

        // Transfer borrowed tokens from lending pool to user
        // SplTokenProgram.transfer(lending_pool_borrow_token_account.key, user_borrow_token_account.key, lending_pool_authority.key, borrow_amount);
        // Simulate state change
        lending_pool.total_borrowed_amount = lending_pool.total_borrowed_amount + borrow_amount;

        user_loan.user = user.key;
        user_loan.collateral_amount = collateral_deposited; // This should be linked to actual deposited collateral
        user_loan.borrowed_amount = borrow_amount;
        user_loan.loan_start_time = get_clock().slot; // Get current timestamp
        user_loan.is_active = true;

        emit BorrowEvent { user: user.key, amount: borrow_amount, collateral_used: collateral_deposited };
    }

    // Repay a loan
    instruction repay_loan(
        @signer user: Account,
        mut user_borrow_token_account: Account, // User's token account for borrowed asset
        mut lending_pool_borrow_token_account: Account, // Lending pool's token account for borrowed asset
        mut user_loan: UserLoanAccount,
        mut lending_pool: LendingPoolAccount,
        repay_amount: u64
    ) {
        require(repay_amount > 0, "Repay amount must be greater than 0");
        require(user_loan.is_active == true, "User does not have an active loan");
        require(repay_amount <= user_loan.borrowed_amount, "Repay amount exceeds outstanding loan");

        // Calculate interest (simplified: proportional to time and interest rate)
        let current_time = get_clock().slot;
        let elapsed_time = current_time - user_loan.loan_start_time;
        // This is a very simplified interest calculation. Real-world would be more complex.
        let interest_due = user_loan.borrowed_amount * lending_pool.interest_rate_bps * elapsed_time / (10000 * 31536000); // 31536000 seconds in a year

        let total_repayment_due = user_loan.borrowed_amount + interest_due;
        require(repay_amount >= total_repayment_due, "Repay amount is less than total due (principal + interest)");

        // Transfer repaid tokens from user to lending pool
        // SplTokenProgram.transfer(user_borrow_token_account.key, lending_pool_borrow_token_account.key, user.key, repay_amount);
        // Simulate state change
        lending_pool.total_borrowed_amount = lending_pool.total_borrowed_amount - user_loan.borrowed_amount; // Subtract principal

        user_loan.borrowed_amount = 0;
        user_loan.is_active = false;

        emit RepayEvent { user: user.key, amount: repay_amount
    }

    // Withdraw collateral after loan is repaid
    instruction withdraw_collateral(
        @signer user: Account,
        mut user_collateral_token_account: Account, // User's token account for collateral
        mut lending_pool_collateral_token_account: Account, // Lending pool's token account for collateral
        mut lending_pool: LendingPoolAccount,
        mut user_loan: UserLoanAccount,
        withdraw_amount: u64
    ) {
        require(withdraw_amount > 0, "Withdraw amount must be greater than 0");
        require(user_loan.is_active == false, "Cannot withdraw collateral with an active loan");
        require(withdraw_amount <= user_loan.collateral_amount, "Withdraw amount exceeds deposited collateral");

        // Transfer collateral from lending pool back to user
        // SplTokenProgram.transfer(lending_pool_collateral_token_account.key, user_collateral_token_account.key, lending_pool_authority.key, withdraw_amount);
        // Simulate state change
        lending_pool.total_collateral_deposited = lending_pool.total_collateral_deposited - withdraw_amount;
        user_loan.collateral_amount = user_loan.collateral_amount - withdraw_amount;

        emit WithdrawEvent { user: user.key, amount: withdraw_amount };
    }
}
