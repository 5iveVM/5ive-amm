    mut global_var: u64;
    
    init {
        global_var = 100;
    }
    
test_function_call_local() -> u64 {
        // This should work - function call result stored in local variable
        let function_result = get_clock();
        
        // This should work - return the local variable
        return function_result;
    }
    
test_literal_local_binary() -> u64 {
        // Test literal value stored in local variable
        let literal_result = 200;
        
        // Test using literal-based local variable in binary operation
        let arithmetic_result = literal_result - global_var;
        
        return arithmetic_result;
    }
    
test_function_call_binary() -> u64 {
        // This should work - function call result stored in local variable  
        let function_result = get_clock();
        
        // This FAILS - using local variable in binary operation
        let arithmetic_result = function_result - global_var;
        
        return function_result;
    }
