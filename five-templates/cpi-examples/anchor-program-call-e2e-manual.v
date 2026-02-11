// Anchor-style token CPI E2E (BPF harness, no validator)
// Uses explicit borsh serializer + 8-byte Anchor discriminators.

interface AnchorTokenComparison @program("EXYTTMwHkRziMdQ1guGGrThxzX6dJDvhJBzz57JGKmsw") @serializer(borsh) {
    mint_to @discriminator([0xF1, 0x22, 0x30, 0xBA, 0x25, 0xB3, 0x7B, 0xC0]) (
        mint: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );
    transfer @discriminator([0xA3, 0x34, 0xC8, 0xE7, 0x8C, 0x03, 0x45, 0xBA]) (
        source: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );
    burn @discriminator([0x74, 0x6E, 0x1D, 0x38, 0x6B, 0xDB, 0x2A, 0x5D]) (
        mint: Account,
        source: Account,
        owner: Account,
        amount: u64
    );
    freeze_account @discriminator([0xFD, 0x4B, 0x52, 0x85, 0xA7, 0xEE, 0x2B, 0x82]) (
        mint: Account,
        source: Account,
        authority: Account
    );
    thaw_account @discriminator([0x73, 0x98, 0x4F, 0xD5, 0xD5, 0xA9, 0xB8, 0x23]) (
        mint: Account,
        source: Account,
        authority: Account
    );
}

pub mint_to_user1(
    mint: account @mut,
    user1_token: account @mut,
    user1: account @signer
) {
    AnchorTokenComparison.mint_to(mint, user1_token, user1, 1000);
}

pub mint_to_user2(
    mint: account @mut,
    user2_token: account @mut,
    user1: account @signer
) {
    AnchorTokenComparison.mint_to(mint, user2_token, user1, 500);
}

pub mint_to_user3(
    mint: account @mut,
    user3_token: account @mut,
    user1: account @signer
) {
    AnchorTokenComparison.mint_to(mint, user3_token, user1, 500);
}

pub transfer_user2_to_user3(
    user2_token: account @mut,
    user3_token: account @mut,
    user2: account @signer
) {
    AnchorTokenComparison.transfer(user2_token, user3_token, user2, 100);
}

pub burn_user1(
    mint: account @mut,
    user1_token: account @mut,
    user1: account @signer
) {
    AnchorTokenComparison.burn(mint, user1_token, user1, 100);
}

pub freeze_user2(
    mint: account @mut,
    user2_token: account @mut,
    user1: account @signer
) {
    AnchorTokenComparison.freeze_account(mint, user2_token, user1);
}

pub thaw_user2(
    mint: account @mut,
    user2_token: account @mut,
    user1: account @signer
) {
    AnchorTokenComparison.thaw_account(mint, user2_token, user1);
}
