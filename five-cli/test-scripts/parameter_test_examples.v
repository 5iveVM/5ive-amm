// Parameter Test Examples - Comprehensive collection of parameter testing approaches
    mut balance: u64;
    mut value: u64;
    mut secondary_balance: u64;
    
    init {
        balance = 0;
        value = 0;
        secondary_balance = 0;
    }
    
    // Example 1: Minimal parameter test (from parameter_test_minimal)
// @test-params 42
pub set_value(x: u64) -> u64 {
        value = x;
        return value;
    }

    // Example 2: Basic deposit function (from parameter_test_basic)
// @test-params 100
deposit(amount: u64) -> u64 {
require(amount > 0);
        balance = balance + amount;
        return balance;
    }

    // Example 3: No parameter function (from parameter_test_no_params)
increment_balance() -> u64 {
        balance = balance + 1;
        return balance;
    }

    // Example 4: Function with simple return (from parameter_test_simple_return)
// @test-params 25
calculate_double(input: u64) -> u64 {
        return input * 2;
    }

    // Example 5: Multiple parameter function
// @test-params 150 50
transfer_between_accounts(from_amount: u64, to_amount: u64) -> u64 {
require(balance >= from_amount);
        balance = balance - from_amount;
        secondary_balance = secondary_balance + to_amount;
        return balance;
    }

    // Example 6: Complex parameter handling
// @test-params 10 20 3
complex_operation(a: u64, b: u64, c: u64) -> u64 {
        let intermediate = a + b;
        let result = intermediate * c;
        value = result;
        return result;
    }
    
    // Utility functions
get_balance() -> u64 {
        return balance;
    }
    
get_secondary_balance() -> u64 {
        return secondary_balance;
    }
    
get_value() -> u64 {
        return value;
    }
    
reset_all() {
        balance = 0;
        value = 0;
        secondary_balance = 0;
    }
