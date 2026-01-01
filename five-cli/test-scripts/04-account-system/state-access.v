    account StateAccount {
        count: u64;
    }
    
pub test(state: StateAccount) -> u64 {
        return state.count;
    }
