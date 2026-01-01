    account StateAccount {
        count: u64;
    }
    
pub test(state: StateAccount @mut) {
        state.count = state.count + 1;
    }
