// ============================================================================
// NFT TYPES
// ============================================================================

account NFTCollection {
    authority: pubkey;
    total_supply: u64;
    max_supply: u64;
    name: string;
    symbol: string;
}

account NFT {
    collection: pubkey;
    owner: pubkey;
    token_id: u64;
    uri: string;
    is_frozen: bool;
}
