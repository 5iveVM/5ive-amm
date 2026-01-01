// Test of documentation counter example using ultra-simple syntax

account StateAccount {
    count: u64;
}

pub initialize(state: StateAccount) {
    state.count = 0;
}

pub increment(state: StateAccount) {
    state.count = state.count + 1;
}

pub get_count(state: StateAccount) -> u64 {
    return state.count;
}