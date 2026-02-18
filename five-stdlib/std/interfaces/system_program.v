// std::interfaces::system_program
// Solana System Program interface

interface SystemProgram @program("11111111111111111111111111111111") {
    transfer @discriminator(2) (
        from: Account,
        to: Account,
        lamports: u64
    );
}
