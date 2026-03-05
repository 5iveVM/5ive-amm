// std::interfaces::stake_program
// Solana stake program CPI surface used by staking/LST programs.

interface StakeProgram @program("Stake11111111111111111111111111111111111111") @serializer("raw") {
    delegate_stake @discriminator_bytes([2, 0, 0, 0]) (
        stake_account: Account @mut,
        vote_account: Account,
        clock_sysvar: Account,
        stake_history_sysvar: Account,
        stake_config_sysvar: Account,
        authority: Account @authority
    );

    split @discriminator_bytes([3, 0, 0, 0]) (
        source_stake_account: Account @mut,
        destination_stake_account: Account @mut,
        authority: Account @authority,
        lamports: u64
    );

    withdraw @discriminator_bytes([4, 0, 0, 0]) (
        stake_account: Account @mut,
        destination_account: Account @mut,
        authority: Account @authority,
        clock_sysvar: Account,
        stake_history_sysvar: Account,
        lamports: u64
    );

    merge @discriminator_bytes([7, 0, 0, 0]) (
        destination_stake_account: Account @mut,
        source_stake_account: Account @mut,
        clock_sysvar: Account,
        stake_history_sysvar: Account,
        authority: Account @authority
    );

    initialize_checked @discriminator_bytes([9, 0, 0, 0]) (
        stake_account: Account @mut,
        rent_sysvar: Account,
        staker: Account @authority,
        withdrawer: Account @authority
    );

    authorize_checked @discriminator_bytes([10, 0, 0, 0]) (
        stake_account: Account @mut,
        clock_sysvar: Account,
        authority: Account @authority,
        new_authority: Account @authority,
        stake_authorize_kind: u32
    );

    // Compatibility wrappers for fixed authorize kinds; generic authorize_checked(kind: u32) is now the canonical path.
    authorize_checked_staker @discriminator_bytes([10, 0, 0, 0, 0, 0, 0, 0]) (
        stake_account: Account @mut,
        clock_sysvar: Account,
        authority: Account @authority,
        new_authority: Account @authority
    );

    authorize_checked_withdrawer @discriminator_bytes([10, 0, 0, 0, 1, 0, 0, 0]) (
        stake_account: Account @mut,
        clock_sysvar: Account,
        authority: Account @authority,
        new_authority: Account @authority
    );
}
