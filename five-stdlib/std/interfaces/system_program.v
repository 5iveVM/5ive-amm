// std::interfaces::system_program
// Solana System Program interface

interface SystemProgram @program("11111111111111111111111111111111") @serializer("raw") {
    transfer @discriminator_bytes([2, 0, 0, 0]) (
        from: Account,
        to: Account,
        lamports: u64
    );
}
