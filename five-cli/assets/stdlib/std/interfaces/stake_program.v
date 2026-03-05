// std::interfaces::stake_program
// Solana stake program CPI surface used by staking/LST programs.

interface StakeProgram @program("Stake11111111111111111111111111111111111111") @serializer(raw) {
    delegate_stake @discriminator_bytes([2, 0, 0, 0]) (
        stake_account: account @mut,
        vote_account: account,
        clock_sysvar: account,
        stake_history_sysvar: account,
        stake_config_sysvar: account,
        authority: account @authority
    );

    split @discriminator_bytes([3, 0, 0, 0]) (
        source_stake_account: account @mut,
        authority: account @authority,
        destination_stake_account: account @mut,
        lamports: u64
    );

    withdraw @discriminator_bytes([4, 0, 0, 0]) (
        stake_account: account @mut,
        destination_account: account @mut,
        authority: account @authority,
        clock_sysvar: account,
        stake_history_sysvar: account,
        lamports: u64
    );

    merge @discriminator_bytes([7, 0, 0, 0]) (
        destination_stake_account: account @mut,
        source_stake_account: account @mut,
        clock_sysvar: account,
        stake_history_sysvar: account,
        authority: account @authority
    );

    initialize_checked @discriminator_bytes([9, 0, 0, 0]) (
        stake_account: account @mut,
        rent_sysvar: account,
        staker: account @authority,
        withdrawer: account @authority
    );

    authorize_checked @discriminator_bytes([10, 0, 0, 0]) (
        stake_account: account @mut,
        clock_sysvar: account,
        authority: account @authority,
        new_authority: account @authority,
        stake_authorize_kind: u32
    );
}
