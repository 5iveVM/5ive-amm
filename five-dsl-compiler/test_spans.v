script test_spans {
    // Test script to verify span capture
    
    instruction test_error() {
        let x = 10;
        let y = undefined_variable; // This should show error with proper span
        let z = x + y;
    }
    
    instruction test_type_error() {
        let a: u64 = "string"; // Type mismatch should show span
    }
}
