    // Global state for caching operations
    mut cached_value: u64;
    mut last_error: u64;
    mut operations_count: u64;

    init {
        cached_value = 0;
        last_error = 0;
        operations_count = 0;
    }

    // Test global state caching and retrieval
pub get_cached_value() -> u64 {
        return cached_value;
    }
    
    // Function that updates cached value
pub update_cached_value(new_value: u64) -> u64 {
        let old_value = cached_value;
        cached_value = new_value;
        operations_count = operations_count + 1;
        return old_value;
    }
    
    // Function that divides numbers safely - simplified version
pub divide_numbers(a: u64, b: u64) -> u64 {
        if (b == 0) {
            last_error = 1;
            return 0;
        }
        let result = a / b;
        operations_count = operations_count + 1;
        return result;
    }
    
    // Function that performs calculation - simplified to avoid nested conditionals
pub safe_calculation(a: u64, b: u64, c: u64) -> u64 {
        if (b == 0) {
            last_error = 1;
            return 0;
        }
        let div_result = a / b;
        operations_count = operations_count + 1;
        let final_result = div_result + c;
        return final_result;
    }
    
    // Utility functions
pub get_operations_count() -> u64 {
        return operations_count;
    }
    
pub clear_cache() -> u64 {
        let old_value = cached_value;
        cached_value = 0;
        last_error = 0;
        return old_value;
    }

    // Test function for test runner
pub test() -> u64 {
        // Test the functionality by performing a safe calculation
        let result = safe_calculation(10, 2, 3);
        return result;
    }