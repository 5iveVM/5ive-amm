// Simple tuple destructuring test

pub test_basic_destructuring(user: pubkey, seed: u64) -> pubkey {
    let (addr, bump) = derive_pda(user, "vault", seed);
    return addr;
}

// Test function that can be called without parameters - ENTRY POINT
test_simple_pda() -> (pubkey, u8) {
    return derive_pda("test", 123);
}