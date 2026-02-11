// Anchor Token CPI E2E (BPF harness, no validator)
// Uses interface-based CPI into five-templates/anchor-token-comparison.
// Includes approve(delegate: pubkey, amount) to verify pubkey data args.

@anchor
interface AnchorToken @program("EXYTTMwHkRziMdQ1guGGrThxzX6dJDvhJBzz57JGKmsw") {
    fn mint_to(
        mint_state: Account,
        destination_account: Account,
        mint_authority: Account,
        amount: u64
    );
    fn transfer(
        source_account: Account,
        destination_account: Account,
        owner: Account,
        amount: u64
    );
    fn transfer_from(
        source_account: Account,
        destination_account: Account,
        authority: Account,
        amount: u64
    );
    fn approve(
        source_account: Account,
        owner: Account,
        delegate: pubkey,
        amount: u64
    );
    fn revoke(
        source_account: Account,
        owner: Account
    );
    fn burn(
        mint_state: Account,
        source_account: Account,
        owner: Account,
        amount: u64
    );
    fn freeze_account(
        mint_state: Account,
        account_to_freeze: Account,
        freeze_authority: Account
    );
    fn thaw_account(
        mint_state: Account,
        account_to_thaw: Account,
        freeze_authority: Account
    );
}

pub anchor_mint_to_user1(
    mint: account @mut,
    user1_token: account @mut,
    user1: account @signer
) {
    AnchorToken.mint_to(mint, user1_token, user1, 1000);
}

pub anchor_mint_to_user2(
    mint: account @mut,
    user2_token: account @mut,
    user1: account @signer
) {
    AnchorToken.mint_to(mint, user2_token, user1, 500);
}

pub anchor_mint_to_user3(
    mint: account @mut,
    user3_token: account @mut,
    user1: account @signer
) {
    AnchorToken.mint_to(mint, user3_token, user1, 500);
}

pub anchor_transfer_user2_to_user3(
    user2_token: account @mut,
    user3_token: account @mut,
    user2: account @signer
) {
    AnchorToken.transfer(user2_token, user3_token, user2, 100);
}

pub anchor_approve_user3_to_user2(
    user3_token: account @mut,
    user3: account @signer,
    delegate: pubkey
) {
    AnchorToken.approve(user3_token, user3, delegate, 150);
}

pub anchor_transfer_from_user3_to_user1_by_user2(
    user3_token: account @mut,
    user1_token: account @mut,
    user2: account @signer
) {
    AnchorToken.transfer_from(user3_token, user1_token, user2, 50);
}

pub anchor_revoke_user3(
    user3_token: account @mut,
    user3: account @signer
) {
    AnchorToken.revoke(user3_token, user3);
}

pub anchor_burn_user1(
    mint: account @mut,
    user1_token: account @mut,
    user1: account @signer
) {
    AnchorToken.burn(mint, user1_token, user1, 100);
}

pub anchor_freeze_user2(
    mint: account @mut,
    user2_token: account @mut,
    user1: account @signer
) {
    AnchorToken.freeze_account(mint, user2_token, user1);
}

pub anchor_thaw_user2(
    mint: account @mut,
    user2_token: account @mut,
    user1: account @signer
) {
    AnchorToken.thaw_account(mint, user2_token, user1);
}
