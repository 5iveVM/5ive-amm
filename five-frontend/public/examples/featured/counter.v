// Simple Counter Contract - 5IVE VM Example

account StateAccount {
    authority: pubkey;
    count: u64;
    initialized: u64;
}

pub initialize(state: StateAccount @mut, owner: account @signer) {
    require(state.initialized == 0);
    state.authority = owner.key;
    state.count = 0;
    state.initialized = 1;
}

pub increment(state: StateAccount @mut, owner: account @signer) {
    require(state.initialized > 0);
    require(state.authority == owner.key);
    state.count = state.count + 1;
}

pub decrement(state: StateAccount @mut, owner: account @signer) {
    require(state.initialized > 0);
    require(state.authority == owner.key);
    if (state.count > 0) {
        state.count = state.count - 1;
    }
}

pub add_amount(state: StateAccount @mut, owner: account @signer, amount: u64) {
    require(state.initialized > 0);
    require(state.authority == owner.key);
    require(amount > 0);
    state.count = state.count + amount;
}

pub get_count(state: StateAccount) -> u64 {
    return state.count;
}

pub reset(state: StateAccount @mut, owner: account @signer) {
    require(state.initialized > 0);
    require(state.authority == owner.key);
    state.count = 0;
}
