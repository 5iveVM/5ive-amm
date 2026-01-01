    account StateAccount {
        count: u64;
    }
    
test(state: StateAccount) -> u64 {
        return state.count;
    }
