// ============================================================================
// NFT TYPES
// ============================================================================

account NFTCollection {
    authority: pubkey;
    total_supply: u64;
    max_supply: u64;
    name: string<32>;
    symbol: string<32>;
}

account NFT {
    collection: pubkey;
    owner: pubkey;
    token_id: u64;
    uri: string<128>;
    is_frozen: bool;
}
