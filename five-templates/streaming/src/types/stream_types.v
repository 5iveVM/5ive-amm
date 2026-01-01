// ============================================================================
// STREAM TYPES
// ============================================================================

account Stream {
    sender: pubkey;
    recipient: pubkey;
    deposit: u64;
    withdrawn: u64;
    start_slot: u64;
    end_slot: u64;
    is_cancelled: bool;
}
