// System program lamports template
// Demonstrates reading and mutating account.lamports for SOL transfers.

// Quote a lamports transfer outcome without mutating state
pub quote_transfer(from: account, to: account, amount: u64) -> (u64, u64) {
    require(amount > 0);
    require(from.ctx.lamports >= amount);
    let new_from = from.ctx.lamports - amount;
    let new_to = to.ctx.lamports + amount;
    return (new_from, new_to);
}

// Read-only helper: ensure an account meets a minimum balance
pub check_min_balance(acc: account, min_balance: u64) -> bool { return acc.ctx.lamports >= min_balance; }

// Compute how much is needed to reach a minimum lamports threshold
pub topup_needed(acc: account, min_balance: u64) -> u64 {
    if (acc.ctx.lamports >= min_balance) { return 0; }
    return min_balance - acc.ctx.lamports;
}
