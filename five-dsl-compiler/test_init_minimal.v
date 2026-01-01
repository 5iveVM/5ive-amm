// Minimal test for @init constraint with custom account types

account TestMint {
    authority: pubkey;
    supply: u64;
}

pub test_init(
    mint_account: TestMint @mut @init,
    authority: account @signer
) -> pubkey {
    mint_account.authority = authority.key;
    mint_account.supply = 0;
    return mint_account.key;
}
