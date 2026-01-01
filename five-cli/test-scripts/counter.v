// Simple Counter Contract - Stacks VM Example
// Demonstrates basic state management and function calls

    account StateAccount {
        count: u64;
    }
    
pub initialize(@init state: StateAccount) {
        // Initialize counter to zero
        state.count = 0;
    }
    
pub increment(state: StateAccount @mut) {
        // Increment the counter by 1
        state.count = state.count + 1;
    }
    
pub decrement(state: StateAccount @mut) {
        // Decrement the counter by 1 (with underflow protection)
if (state.count > 0) {
            state.count = state.count - 1;
        }
    }
    
pub add_amount(state: StateAccount @mut, amount: u64) {
        // Add a specific amount to the counter
        state.count = state.count + amount;
    }
    
pub get_count(state: StateAccount) -> u64 {
        // Return the current count
        return state.count;
    }
    
pub reset(state: StateAccount @mut) {
        // Reset counter to zero
        state.count = 0;
    }
