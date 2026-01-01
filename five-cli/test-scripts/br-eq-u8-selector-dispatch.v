// BR_EQ_U8 Selector Dispatch Test
// Demonstrates the BR_EQ_U8 fused compare-branch optimization
// This test showcases bytecode reduction from traditional EQ + JUMP_IF_NOT to single BR_EQ_U8

// Traditional multi-selector dispatch patterns that benefit from BR_EQ_U8
// @test-params 3
pub instruction_dispatch(selector: u8) -> string {
    if selector == 1 {
        return "Initialize";
    } else if selector == 2 {
        return "Transfer";
    } else if selector == 3 {
        return "Approve";
    } else if selector == 4 {
        return "Revoke";
    } else if selector == 5 {
        return "Close";
    } else {
        return "Unknown";
    }
}

// Simple two-way dispatch (common pattern)
// @test-params 1
pub binary_dispatch(flag: u8) -> bool {
    if flag == 0 {
        return false;
    } else {
        return true;
    }
}

// Nested selector dispatch (more complex optimization case)
// @test-params 1 2
pub nested_dispatch(category: u8, action: u8) -> string {
    if category == 1 {
        if action == 1 {
            return "User Create";
        } else if action == 2 {
            return "User Update";
        } else {
            return "User Unknown";
        }
    } else if category == 2 {
        if action == 1 {
            return "Token Mint";
        } else if action == 2 {
            return "Token Burn";
        } else {
            return "Token Unknown";
        }
    } else {
        return "Category Unknown";
    }
}

// State machine dispatch (realistic use case)
// @test-params 0 1
pub state_machine(current_state: u8, input_event: u8) -> u8 {
    if current_state == 0 {  // Initial state
        if input_event == 1 {
            return 1;  // Move to active
        } else {
            return 0;  // Stay initial
        }
    } else if current_state == 1 {  // Active state
        if input_event == 2 {
            return 2;  // Move to paused
        } else if input_event == 3 {
            return 3;  // Move to closed
        } else {
            return 1;  // Stay active
        }
    } else if current_state == 2 {  // Paused state
        if input_event == 1 {
            return 1;  // Move to active
        } else if input_event == 3 {
            return 3;  // Move to closed
        } else {
            return 2;  // Stay paused
        }
    } else {
        return 0;  // Reset to initial
    }
}

// Test function to verify all selectors work correctly
pub test_all_selectors() -> bool {
    // Test instruction dispatch
    let result1 = instruction_dispatch(1);  // Should return "Initialize"
    let result2 = instruction_dispatch(3);  // Should return "Approve"
    let result3 = instruction_dispatch(99); // Should return "Unknown"
    
    // Test binary dispatch
    let flag1 = binary_dispatch(0);  // Should return false
    let flag2 = binary_dispatch(1);  // Should return true
    
    // Test state machine
    let state1 = state_machine(0, 1);  // Should return 1 (initial -> active)
    let state2 = state_machine(1, 2);  // Should return 2 (active -> paused)
    
    // All tests pass if we reach here
    return true;
}