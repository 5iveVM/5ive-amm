// ============================================================================
// STREAM CORE
// ============================================================================

pub create_stream(
    stream: Stream @mut @init,
    sender: account @signer,
    recipient: pubkey,
    deposit: u64,
    start_slot: u64,
    end_slot: u64
) -> pubkey {
    require(deposit > 0);
    require(start_slot < end_slot);
    stream.sender = sender.key;
    stream.recipient = recipient;
    stream.deposit = deposit;
    stream.withdrawn = 0;
    stream.start_slot = start_slot;
    stream.end_slot = end_slot;
    stream.is_cancelled = false;
    return stream.key;
}

pub withdraw(
    stream: Stream @mut,
    recipient: account @signer
) -> u64 {
    require(!stream.is_cancelled);
    require(stream.recipient == recipient.key);

    let now: u64 = get_clock().slot;
    let available: u64 = 0;
    if (now <= stream.start_slot) {
        available = 0;
    } else if (now >= stream.end_slot) {
        available = stream.deposit;
    } else {
        let elapsed: u64 = now - stream.start_slot;
        let duration: u64 = stream.end_slot - stream.start_slot;
        available = (stream.deposit * elapsed) / duration;
    }

    require(available >= stream.withdrawn);
    let claimable: u64 = available - stream.withdrawn;
    stream.withdrawn = available;
    return claimable;
}

pub cancel_stream(
    stream: Stream @mut,
    sender: account @signer
) {
    require(stream.sender == sender.key);
    require(!stream.is_cancelled);
    stream.is_cancelled = true;
}
