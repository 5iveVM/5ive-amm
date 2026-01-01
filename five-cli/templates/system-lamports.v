// System program lamports template
// Demonstrates reading and mutating account.lamports for SOL transfers.

// Quote a lamports transfer outcome without mutating state
quote_transfer(from: account, to: account, amount: u64) -> (u64, u64) {
    require(amount > 0);
    require(from.lamports >= amount);
    let new_from = from.lamports - amount;
    let new_to = to.lamports + amount;
    return (new_from, new_to);
}

// Read-only helper: ensure an account meets a minimum balance
check_min_balance(acc: account, min_balance: u64) -> bool { return acc.lamports >= min_balance; }

// Compute how much is needed to reach a minimum lamports threshold
topup_needed(acc: account, min_balance: u64) -> u64 {
    if (acc.lamports >= min_balance) { return 0; }
    return min_balance - acc.lamports;
}
