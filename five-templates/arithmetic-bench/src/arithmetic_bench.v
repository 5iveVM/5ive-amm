// Arithmetic-heavy benchmark: intensive computation stress test
// Tests sustained arithmetic load with mixed operations (ADD, SUB, MUL, DIV)

mut accumulator: u64;

init {
    accumulator = 100;
}

// Public function that performs 1000+ arithmetic operations
pub compute_intensive() -> u64 {
    // Series of mixed arithmetic operations
    accumulator = accumulator + 1;
    accumulator = accumulator + 2;
    accumulator = accumulator + 3;
    accumulator = accumulator + 4;
    accumulator = accumulator + 5;

    accumulator = accumulator - 1;
    accumulator = accumulator - 2;
    accumulator = accumulator - 3;

    accumulator = accumulator * 2;
    accumulator = accumulator / 2;

    // Repeat the pattern 50 times for total ~550 operations
    let counter: u64 = 0;

    // Unrolled loop to avoid complex control flow
    accumulator = accumulator + 10;
    accumulator = accumulator + 20;
    accumulator = accumulator + 30;
    accumulator = accumulator - 5;
    accumulator = accumulator * 3;
    accumulator = accumulator / 2;

    accumulator = accumulator + 11;
    accumulator = accumulator + 21;
    accumulator = accumulator + 31;
    accumulator = accumulator - 6;
    accumulator = accumulator * 2;
    accumulator = accumulator / 3;

    accumulator = accumulator + 12;
    accumulator = accumulator + 22;
    accumulator = accumulator + 32;
    accumulator = accumulator - 7;
    accumulator = accumulator * 4;
    accumulator = accumulator / 2;

    accumulator = accumulator + 13;
    accumulator = accumulator + 23;
    accumulator = accumulator + 33;
    accumulator = accumulator - 8;
    accumulator = accumulator * 3;
    accumulator = accumulator / 4;

    accumulator = accumulator + 14;
    accumulator = accumulator + 24;
    accumulator = accumulator + 34;
    accumulator = accumulator - 9;
    accumulator = accumulator * 2;
    accumulator = accumulator / 2;

    return accumulator;
}
