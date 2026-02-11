// Anchor Program Call E2E Test
//
// Demonstrates calling an Anchor program using the @anchor interface style.
// This test verifies that the compiler correctly:
// 1. Derives the 8-byte discriminator from the method name "global:initialize"
// 2. Encodes the arguments correctly (though serialization is standard)
// 3. Emits the correct INVOKE sequence

program("11111111111111111111111111111111");

// Define an Anchor interface
// The compiler should derive the discriminator for 'initialize'
@anchor
interface Counter {
    fn initialize(count: u64);
    
    // Explicit override should work too
    @discriminator_bytes([1, 2, 3, 4, 5, 6, 7, 8])
    fn reset();
}

// Main entry point
fn main() {
    // 1. Call initialize
    // Expected discriminator: sha256("global:initialize")[..8]
    // = afaf6d1f0d989bed
    Counter.initialize(42);

    // 2. Call reset
    // Expected discriminator: 0102030405060708
    Counter.reset();
}
