// std::types
// Shared value types used by builtin return signatures.

pub type Clock {
    slot: u64,
    epoch_start_timestamp: i64,
    epoch: u64,
    leader_schedule_epoch: u64,
    unix_timestamp: i64,
}

pub type ProgramAddress = (pubkey, u8);
