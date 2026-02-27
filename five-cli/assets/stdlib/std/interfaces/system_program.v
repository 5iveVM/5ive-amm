// std::interfaces::system_program
// Solana System Program interface

interface SystemProgram @program("11111111111111111111111111111111") @serializer(raw) {
    create_account @discriminator_bytes([0, 0, 0, 0]) (
        payer: account,
        new_account: account,
        lamports: u64,
        space: u64,
        owner: account
    );

    assign @discriminator_bytes([1, 0, 0, 0]) (
        target_account: account,
        owner: account
    );

    transfer @discriminator_bytes([2, 0, 0, 0]) (
        source: account,
        destination: account,
        lamports: u64
    );

    create_account_with_seed @discriminator_bytes([3, 0, 0, 0]) (
        payer: account,
        new_account: account,
        base: account,
        seed: u64,
        lamports: u64,
        space: u64,
        owner: account
    );
}
