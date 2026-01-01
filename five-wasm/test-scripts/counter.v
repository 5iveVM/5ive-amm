// Simple Counter Contract - Stacks VM Example
// Demonstrates basic state management and function calls

    account StateAccount {
        count: u64;
    }
    
initialize(state: StateAccount) {
        // Initialize counter to zero
        state.count = 0;
    }
    
increment(state: StateAccount) {
        // Increment the counter by 1
        state.count = state.count + 1;
    }
    
decrement(state: StateAccount) {
        // Decrement the counter by 1 (with underflow protection)
if (state.count > 0) {
            state.count = state.count - 1;
        }
    }
    
add_amount(state: StateAccount, amount: u64) {
        // Add a specific amount to the counter
        state.count = state.count + amount;
    }
    
get_count(state: StateAccount) -> u64 {
        // Return the current count
        return state.count;
    }
    
reset(state: StateAccount) {
        // Reset counter to zero
        state.count = 0;
    }
