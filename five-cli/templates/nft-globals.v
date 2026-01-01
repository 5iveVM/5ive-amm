// NFT (globals) transfer-only template
// Assumes metadata is configured globally at deploy time; this script
// only handles ownership transfers for minted NFTs.

account NFT {
    token_id: pubkey;
    owner_key: pubkey;
    uri: string;
}

transfer(state: NFT @mut, from: pubkey, to: pubkey) {
    require(state.owner_key == from);
    state.owner_key = to;
}

