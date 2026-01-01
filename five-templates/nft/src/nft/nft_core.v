// ============================================================================
// NFT CORE
// ============================================================================

pub init_collection(
    collection: NFTCollection @mut @init,
    authority: account @signer,
    max_supply: u64,
    name: string,
    symbol: string
) -> pubkey {
    collection.authority = authority.key;
    collection.total_supply = 0;
    collection.max_supply = max_supply;
    collection.name = name;
    collection.symbol = symbol;
    return collection.key;
}

pub mint_nft(
    collection: NFTCollection @mut,
    nft: NFT @mut @init,
    authority: account @signer,
    owner: pubkey,
    token_id: u64,
    uri: string
) -> pubkey {
    require(collection.authority == authority.key);
    require(collection.total_supply < collection.max_supply);
    collection.total_supply = collection.total_supply + 1;
    nft.collection = collection.key;
    nft.owner = owner;
    nft.token_id = token_id;
    nft.uri = uri;
    nft.is_frozen = false;
    return nft.key;
}

pub transfer_nft(
    nft: NFT @mut,
    owner: account @signer,
    new_owner: pubkey
) {
    require(!nft.is_frozen);
    require(nft.owner == owner.key);
    nft.owner = new_owner;
}

pub freeze_nft(
    collection: NFTCollection,
    nft: NFT @mut,
    authority: account @signer
) {
    require(collection.authority == authority.key);
    require(nft.collection == collection.key);
    nft.is_frozen = true;
}

pub thaw_nft(
    collection: NFTCollection,
    nft: NFT @mut,
    authority: account @signer
) {
    require(collection.authority == authority.key);
    require(nft.collection == collection.key);
    nft.is_frozen = false;
}
