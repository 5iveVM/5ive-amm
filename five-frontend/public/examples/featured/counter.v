// Simple Counter Contract - 5IVE VM Example

account StateAccount {
    count: u64;
}

pub initialize(state: StateAccount @mut) {
    state.count = 0;
}

pub increment(state: StateAccount @mut) {
    state.count = state.count + 1;
}

pub decrement(state: StateAccount @mut) {
    if (state.count > 0) {
        state.count = state.count - 1;
    }
}

pub add_amount(state: StateAccount @mut, amount: u64) {
    state.count = state.count + amount;
}

pub get_count(state: StateAccount) -> u64 {
    return state.count;
}

pub reset(state: StateAccount @mut) {
    state.count = 0;
}
