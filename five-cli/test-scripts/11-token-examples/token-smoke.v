account MintState {
    authority: pubkey;
    supply: u64;
}

account HolderAccount {
    owner: pubkey;
    balance: u64;
}

pub mint_tokens(
    mint: MintState @mut,
    holder: HolderAccount @mut,
    authority: pubkey,
    amount: u64
) -> u64 {
    require(amount > 0);
    mint.authority = authority;
    mint.supply = mint.supply + amount;
    holder.owner = authority;
    holder.balance = holder.balance + amount;
    return holder.balance;
}
