// ============================================================================
// ORACLE TYPES
// ============================================================================

account OracleFeed {
    authority: pubkey;
    price: u64;
    last_update_slot: u64;
    max_staleness_slots: u64;
    is_paused: bool;
    name: string;
}
