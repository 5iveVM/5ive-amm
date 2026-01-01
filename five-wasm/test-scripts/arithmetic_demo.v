    base_number: u64;
    
    init {
        base_number = 100;
    }
    
test_addition() -> u64 {
        let result = 100 + 25;
require(result > 0);
        return result;
    }
    
test_subtraction() -> u64 {
        let result = 100 - 25;
require(result >= 0);
        return result;
    }
    
test_multiplication() -> u64 {
        let result = 25 * 5;
require(result > 0);
        return result;
    }
    
test_division() -> u64 {
        let result = 100 / 5;
require(result > 0);
        return result;
    }
    
calculate_with_base(a: u64, b: u64) -> u64 {
        let result = (a + b) * base_number;
        return result;
    }
