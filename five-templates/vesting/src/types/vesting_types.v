// ============================================================================
// VESTING TYPES
// ============================================================================

account VestingSchedule {
    beneficiary: pubkey;
    total_amount: u64;
    released_amount: u64;
    start_slot: u64;
    cliff_slot: u64;
    end_slot: u64;
    revocable: bool;
    revoked: bool;
}
