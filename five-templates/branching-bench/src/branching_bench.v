// Branching-heavy benchmark: control flow stress test
// Tests conditional overhead with validation patterns

mut result: u64;
mut counter: u64;

init {
    result = 0;
    counter = 0;
}

// Public function with conditional and validation patterns
pub branching_workload() -> u64 {
    result = 0;
    counter = 0;

    // Pattern 1: Multiple conditional chains
    if result == 0 {
        result = result + 1;
    }
    if result == 1 {
        result = result + 2;
    }
    if result == 3 {
        result = result + 3;
    }

    // Pattern 2: Validation/require checks (expensive branches)
    require(result > 0);
    require(result < 1000000);
    require(result != 0);

    // Pattern 3: More conditional patterns
    if result > 5 {
        result = result + 10;
    }
    if result < 1000 {
        result = result + 5;
    }
    if result % 2 == 0 {
        result = result / 2;
    }

    // Pattern 4: Repeated conditionals (simulating loop overhead)
    if counter == 0 { counter = 1; }
    if counter == 1 { counter = 2; }
    if counter == 2 { counter = 3; }
    if counter == 3 { counter = 4; }
    if counter == 4 { counter = 5; }
    if counter == 5 { counter = 6; }
    if counter == 6 { counter = 7; }
    if counter == 7 { counter = 8; }
    if counter == 8 { counter = 9; }
    if counter == 9 { counter = 10; }

    if counter == 10 { counter = 11; }
    if counter == 11 { counter = 12; }
    if counter == 12 { counter = 13; }
    if counter == 13 { counter = 14; }
    if counter == 14 { counter = 15; }
    if counter == 15 { counter = 16; }
    if counter == 16 { counter = 17; }
    if counter == 17 { counter = 18; }
    if counter == 18 { counter = 19; }
    if counter == 19 { counter = 20; }

    // Pattern 5: More validation
    require(result > 0);
    require(counter > 0);

    return result + counter;
}
