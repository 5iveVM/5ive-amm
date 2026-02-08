// Test without enum/match patterns

pub test() -> u64 {
    let input = 50;
    if (input > 100) {
        return input * 2;
    } else if (input > 10) {
        return input + 50;
    } else {
        return 0;
    }
}
