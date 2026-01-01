    mut global_var: u64;
    
    init {
        global_var = 100;
    }
    
test_literal_local() -> u64 {
        // Test literal local variable in binary operation (WORKS)
        let local_var = 200;
        let result = local_var - global_var;
        return result;
    }
    
test_function_local() -> u64 {
        // Test function call local variable in binary operation (FAILS)
        let function_var = get_clock();
        let result = function_var - global_var;
        return result;
    }
