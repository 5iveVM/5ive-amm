// Test script that imports account types from another module
import test_types_module as types;

pub test_init_imported(
    mint_account: types::TestMint @mut @init,
    authority: account @signer
) -> pubkey {
    mint_account.authority = authority.key;
    mint_account.supply = 0;
    return mint_account.key;
}
