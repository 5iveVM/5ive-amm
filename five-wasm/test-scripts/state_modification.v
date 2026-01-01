    account StateAccount {
        count: u64;
    }
    
test(state: StateAccount @mut) {
        state.count = 42;
    }
