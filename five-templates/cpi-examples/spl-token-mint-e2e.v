// SPL Token CPI E2E (BPF harness, no validator)
// Uses default serializer (bincode) for SPL CPI instructions.

interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    mint_to @discriminator(7) (
        mint: Account,
        to: Account,
        authority: Account,
        amount: u64
    );
    transfer @discriminator(3) (
        source: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );
    approve @discriminator(4) (
        source: Account,
        delegate: Account,
        authority: Account,
        amount: u64
    );
    revoke @discriminator(5) (
        source: Account,
        authority: Account
    );
    burn @discriminator(8) (
        source: Account,
        mint: Account,
        authority: Account,
        amount: u64
    );
    freeze_account @discriminator(10) (
        source: Account,
        mint: Account,
        authority: Account
    );
    thaw_account @discriminator(11) (
        source: Account,
        mint: Account,
        authority: Account
    );
}

pub mint_to_user1(
    mint: account @mut,
    user1_token: account @mut,
    user1: account @signer
) {
    SPLToken.mint_to(mint, user1_token, user1, 1000);
}

pub mint_to_user2(
    mint: account @mut,
    user2_token: account @mut,
    user1: account @signer
) {
    SPLToken.mint_to(mint, user2_token, user1, 500);
}

pub mint_to_user3(
    mint: account @mut,
    user3_token: account @mut,
    user1: account @signer
) {
    SPLToken.mint_to(mint, user3_token, user1, 500);
}

pub transfer_user2_to_user3(
    user2_token: account @mut,
    user3_token: account @mut,
    user2: account @signer
) {
    SPLToken.transfer(user2_token, user3_token, user2, 100);
}

pub approve_user3_to_user2(
    user3_token: account @mut,
    user2: account,
    user3: account @signer
) {
    SPLToken.approve(user3_token, user2, user3, 150);
}

pub transfer_from_user3_to_user1_by_user2(
    user3_token: account @mut,
    user1_token: account @mut,
    user2: account @signer
) {
    SPLToken.transfer(user3_token, user1_token, user2, 50);
}

pub revoke_user3(
    user3_token: account @mut,
    user3: account @signer
) {
    SPLToken.revoke(user3_token, user3);
}

pub burn_user1(
    mint: account @mut,
    user1_token: account @mut,
    user1: account @signer
) {
    SPLToken.burn(user1_token, mint, user1, 100);
}

pub freeze_user2(
    mint: account @mut,
    user2_token: account @mut,
    user1: account @signer
) {
    SPLToken.freeze_account(user2_token, mint, user1);
}

pub thaw_user2(
    mint: account @mut,
    user2_token: account @mut,
    user1: account @signer
) {
    SPLToken.thaw_account(user2_token, mint, user1);
}
