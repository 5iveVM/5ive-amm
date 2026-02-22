// std::interfaces::system_program
// Solana System Program interface

interface SystemProgram @program("11111111111111111111111111111111") @serializer("raw") {
    create_account @discriminator_bytes([0, 0, 0, 0]) (
        payer: Account,
        new_account: Account,
        lamports: u64,
        space: u64,
        owner: Account
    );

    assign @discriminator_bytes([1, 0, 0, 0]) (
        target_account: Account,
        owner: Account
    );

    transfer @discriminator_bytes([2, 0, 0, 0]) (
        source: Account,
        destination: Account,
        lamports: u64
    );

    create_account_with_seed @discriminator_bytes([3, 0, 0, 0]) (
        payer: Account,
        new_account: Account,
        base: Account,
        seed: u64,
        lamports: u64,
        space: u64,
        owner: Account
    );
}
