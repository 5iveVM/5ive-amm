// Timelock/Vesting template (linear)

account VestingState {
    beneficiary: pubkey;
    start_time: u64;
    cliff_seconds: u64;
    duration_seconds: u64;
    total_amount: u64;
    released_amount: u64;
}

// Initialize a vesting schedule
init_vesting(state: VestingState @mut, beneficiary: pubkey, start: u64, cliff: u64, duration: u64, total: u64) {
    state.beneficiary = beneficiary;
    state.start_time = start;
    state.cliff_seconds = cliff;
    state.duration_seconds = duration;
    state.total_amount = total;
    state.released_amount = 0;
}

// Compute releasable amount at current time
// Release vested tokens (accounting only) - template simplified
release(state: VestingState @mut, amount: u64) -> u64 {
    require(amount > 0);
    state.released_amount = state.released_amount + amount;
    return amount;
}
