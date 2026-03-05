// ============================================================================
// VESTING CORE
// ============================================================================

pub init_vesting(
    schedule: VestingSchedule @mut @init,
    beneficiary: pubkey,
    total_amount: u64,
    start_slot: u64,
    cliff_slot: u64,
    end_slot: u64,
    revocable: bool
) -> pubkey {
    require(total_amount > 0);
    require(start_slot <= cliff_slot);
    require(cliff_slot <= end_slot);

    schedule.beneficiary = beneficiary;
    schedule.total_amount = total_amount;
    schedule.released_amount = 0;
    schedule.start_slot = start_slot;
    schedule.cliff_slot = cliff_slot;
    schedule.end_slot = end_slot;
    schedule.revocable = revocable;
    schedule.revoked = false;
    return schedule.key;
}

pub release(
    schedule: VestingSchedule @mut,
    beneficiary: account @signer
) -> u64 {
    require(schedule.beneficiary == beneficiary.key);
    require(!schedule.revoked);

    let now: u64 = get_clock().slot;
    if (now < schedule.cliff_slot) {
        return 0;
    }

    let vested: u64 = 0;
    if (now >= schedule.end_slot) {
        vested = schedule.total_amount;
    } else {
        let elapsed: u64 = now - schedule.start_slot;
        let duration: u64 = schedule.end_slot - schedule.start_slot;
        vested = (schedule.total_amount * elapsed) / duration;
    }

    require(vested >= schedule.released_amount);
    let releasable: u64 = vested - schedule.released_amount;
    schedule.released_amount = vested;
    return releasable;
}

pub revoke(
    schedule: VestingSchedule @mut,
    authority: account @signer
) {
    require(authority.key != 0);
    require(schedule.revocable);
    schedule.revoked = true;
}
