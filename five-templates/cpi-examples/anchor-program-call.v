// Anchor Program Call Example
//
// This contract demonstrates CPI to an Anchor program.
// Anchor uses 8-byte discriminators (sighash of "global:method_name").
//
// Interface: Calls a custom Anchor program
// Serializer: Borsh (Anchor standard)
// Discriminator: 8-byte array (Anchor sighash format)
// Data Args: value (u64 literal)
// Account Args: counter, user

interface CounterProgram @program("CounterProgramIdHere111111111111111111111111") {
    increment @discriminator([0xAA, 0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD, 0xEF]) (
        counter: pubkey,
        user: pubkey,
        amount: u64
    );
}

pub increment_remote(
    counter: account @mut,
    user: account @signer,
    amount: u64
) {
    // Call Anchor program's increment instruction
    // - counter: the counter account to increment
    // - user: user account (optional, for tracking)
    // - amount: increment amount (as u64 literal)
    CounterProgram.increment(counter, user, 1);
}
