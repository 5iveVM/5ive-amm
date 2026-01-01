    account StateAccount {
        count: u64;
    }
    
pub test(state: StateAccount @mut) {
        state.count = 42;
    }
