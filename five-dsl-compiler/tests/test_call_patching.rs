/// CALL Opcode & Patching Test Suite
///
/// Tests the call.rs module (CallSiteTracker) which handles:
/// - CALL opcode emission
/// - Function address recording
/// - CALL address patching (forward/backward references)
/// - CALL metadata (under call-metadata feature)
/// - Name deduplication
use five_dsl_compiler::bytecode_generator::call::CallSiteTracker;
use five_dsl_compiler::bytecode_generator::opcodes::OpcodeEmitter;
use five_protocol::opcodes::CALL;

// Simple test emitter for unit tests
struct TestEmitter {
    bytecode: Vec<u8>,
}

impl TestEmitter {
    fn new() -> Self {
        Self {
            bytecode: Vec::new(),
        }
    }

    fn get_bytecode(&self) -> &[u8] {
        &self.bytecode
    }
}

impl OpcodeEmitter for TestEmitter {
    fn emit_opcode(&mut self, opcode: u8) {
        self.bytecode.push(opcode);
    }

    fn emit_u8(&mut self, value: u8) {
        self.bytecode.push(value);
    }

    fn emit_u16(&mut self, value: u16) {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
    }

    fn emit_u32(&mut self, value: u32) {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
    }

    fn emit_u64(&mut self, value: u64) {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
    }

    fn emit_bytes(&mut self, bytes: &[u8]) {
        self.bytecode.extend_from_slice(bytes);
    }

    fn get_position(&self) -> usize {
        self.bytecode.len()
    }

    fn patch_u32(&mut self, position: usize, value: u32) {
        let bytes = value.to_le_bytes();
        if position + 3 < self.bytecode.len() {
            self.bytecode[position..position + 4].copy_from_slice(&bytes);
        }
    }

    fn patch_u16(&mut self, position: usize, value: u16) {
        let bytes = value.to_le_bytes();
        if position + 1 < self.bytecode.len() {
            self.bytecode[position..position + 2].copy_from_slice(&bytes);
        }
    }

    fn should_include_tests(&self) -> bool {
        false
    }

    fn emit_const_u8(&mut self, _value: u8) -> Result<(), five_vm_mito::error::VMError> {
        Ok(())
    }
    fn emit_const_u16(&mut self, _value: u16) -> Result<(), five_vm_mito::error::VMError> {
        Ok(())
    }
    fn emit_const_u32(&mut self, _value: u32) -> Result<(), five_vm_mito::error::VMError> {
        Ok(())
    }
    fn emit_const_u64(&mut self, _value: u64) -> Result<(), five_vm_mito::error::VMError> {
        Ok(())
    }
    fn emit_const_i64(&mut self, _value: i64) -> Result<(), five_vm_mito::error::VMError> {
        Ok(())
    }
    fn emit_const_bool(&mut self, _value: bool) -> Result<(), five_vm_mito::error::VMError> {
        Ok(())
    }
    fn emit_const_u128(&mut self, _value: u128) -> Result<(), five_vm_mito::error::VMError> {
        Ok(())
    }
    fn emit_const_pubkey(&mut self, _value: &[u8; 32]) -> Result<(), five_vm_mito::error::VMError> {
        Ok(())
    }
    fn emit_const_string(&mut self, _value: &[u8]) -> Result<(), five_vm_mito::error::VMError> {
        Ok(())
    }
}

// ============================================================================
// Test Group 1: Basic CallSiteTracker Operations
// ============================================================================

#[test]
fn test_call_site_tracker_creation() {
    let tracker = CallSiteTracker::new();
    // Should be empty on creation
    let mut emitter = TestEmitter::new();
    assert!(
        tracker.patch_calls(&mut emitter).is_ok(),
        "Should succeed with no patches"
    );
}

#[test]
fn test_record_function_position() {
    let mut tracker = CallSiteTracker::new();

    tracker.record_function_position("add".to_string(), 100);
    tracker.record_function_position("sub".to_string(), 200);
    tracker.record_function_position("mul".to_string(), 300);

    // Patching should work after recording positions
    let mut emitter = TestEmitter::new();
    assert!(tracker.patch_calls(&mut emitter).is_ok());
}

#[test]
fn test_record_patch_at_position() {
    let mut tracker = CallSiteTracker::new();

    // Record a patch before the function position is known
    tracker.record_patch_at_position(10, "helper".to_string());

    // Later, record the function position
    tracker.record_function_position("helper".to_string(), 100);

    // Should be able to patch now
    let mut emitter = TestEmitter::new();
    emitter.bytecode.resize(20, 0); // Make space for patch
    assert!(tracker.patch_calls(&mut emitter).is_ok());
}

#[test]
fn test_reset_clears_state() {
    let mut tracker = CallSiteTracker::new();

    tracker.record_patch_at_position(10, "func1".to_string());
    tracker.record_function_position("func1".to_string(), 100);

    tracker.reset();

    // After reset, should have no patches
    let mut emitter = TestEmitter::new();
    assert!(tracker.patch_calls(&mut emitter).is_ok());
}

// ============================================================================
// Test Group 2: CALL Emission (without metadata feature)
// ============================================================================

#[test]
fn test_emit_call_basic() {
    let mut tracker = CallSiteTracker::new();
    let mut emitter = TestEmitter::new();

    // Emit CALL with 2 parameters to address 100
    let metadata_size = tracker.emit_call_with_metadata(
        &mut emitter,
        2,     // param_count
        100,   // function_address
        "add", // function_name
    );

    let bytecode = emitter.get_bytecode();

    // Without call-metadata feature, should just emit standard CALL
    assert_eq!(bytecode[0], CALL, "Should emit CALL opcode");
    assert_eq!(bytecode[1], 2, "Should have param_count=2");
    assert_eq!(
        u16::from_le_bytes([bytecode[2], bytecode[3]]),
        100,
        "Should have address=100"
    );

    #[cfg(not(feature = "call-metadata"))]
    {
        assert_eq!(metadata_size, 0, "Should not emit metadata");
        assert_eq!(bytecode.len(), 4, "CALL should be 4 bytes");
    }
}

#[test]
fn test_emit_multiple_calls() {
    let mut tracker = CallSiteTracker::new();
    let mut emitter = TestEmitter::new();

    tracker.emit_call_with_metadata(&mut emitter, 0, 100, "func1");
    tracker.emit_call_with_metadata(&mut emitter, 1, 200, "func2");
    tracker.emit_call_with_metadata(&mut emitter, 2, 300, "func3");

    let bytecode = emitter.get_bytecode();

    #[cfg(not(feature = "call-metadata"))]
    {
        // Should be 3 CALL opcodes, each 4 bytes
        assert_eq!(bytecode.len(), 12, "Should have 3 CALL opcodes");
        assert_eq!(bytecode[0], CALL);
        assert_eq!(bytecode[4], CALL);
        assert_eq!(bytecode[8], CALL);
    }
}

// ============================================================================
// Test Group 3: Forward Reference Patching
// ============================================================================

#[test]
fn test_forward_reference_single_call() {
    let mut tracker = CallSiteTracker::new();
    let mut emitter = TestEmitter::new();

    // Emit CALL before function address is known (forward reference)
    tracker.emit_call_with_metadata(&mut emitter, 2, 0, "helper");

    // Record that we need to patch this CALL
    let call_position = emitter.get_position() - 2; // Address is last 2 bytes
    tracker.record_patch_at_position(call_position, "helper".to_string());

    // Later, when we generate the function body, record its position
    tracker.record_function_position("helper".to_string(), 150);

    // Patch all CALLs
    assert!(tracker.patch_calls(&mut emitter).is_ok());

    // Verify the patch was applied
    let bytecode = emitter.get_bytecode();
    let patched_address =
        u16::from_le_bytes([bytecode[call_position], bytecode[call_position + 1]]);
    assert_eq!(patched_address, 150, "Should patch address to 150");
}

#[test]
fn test_forward_reference_multiple_calls_to_same_function() {
    let mut tracker = CallSiteTracker::new();
    let mut emitter = TestEmitter::new();

    // Three calls to the same function before it's defined
    for _ in 0..3 {
        tracker.emit_call_with_metadata(&mut emitter, 1, 0, "common");
        let call_position = emitter.get_position() - 2;
        tracker.record_patch_at_position(call_position, "common".to_string());
    }

    // Function is defined at position 200
    tracker.record_function_position("common".to_string(), 200);

    // Patch all calls
    assert!(tracker.patch_calls(&mut emitter).is_ok());

    // Verify all three calls were patched
    let bytecode = emitter.get_bytecode();

    #[cfg(not(feature = "call-metadata"))]
    {
        let addr1 = u16::from_le_bytes([bytecode[2], bytecode[3]]);
        let addr2 = u16::from_le_bytes([bytecode[6], bytecode[7]]);
        let addr3 = u16::from_le_bytes([bytecode[10], bytecode[11]]);

        assert_eq!(addr1, 200);
        assert_eq!(addr2, 200);
        assert_eq!(addr3, 200);
    }
}

// ============================================================================
// Test Group 4: Backward Reference Patching
// ============================================================================

#[test]
fn test_backward_reference() {
    let mut tracker = CallSiteTracker::new();
    let mut emitter = TestEmitter::new();

    // Function is defined first at position 50
    tracker.record_function_position("defined_early".to_string(), 50);

    // Later, we call it (backward reference)
    emitter.bytecode.resize(100, 0); // Simulate some bytecode
    tracker.emit_call_with_metadata(&mut emitter, 0, 0, "defined_early");
    let call_position = emitter.get_position() - 2;
    tracker.record_patch_at_position(call_position, "defined_early".to_string());

    // Patch
    assert!(tracker.patch_calls(&mut emitter).is_ok());

    // Verify
    let bytecode = emitter.get_bytecode();
    let patched_address =
        u16::from_le_bytes([bytecode[call_position], bytecode[call_position + 1]]);
    assert_eq!(patched_address, 50);
}

// ============================================================================
// Test Group 5: Mixed Forward/Backward References
// ============================================================================

#[test]
fn test_mixed_forward_backward_references() {
    let mut tracker = CallSiteTracker::new();
    let mut emitter = TestEmitter::new();

    // Function A is defined at position 0
    tracker.record_function_position("func_a".to_string(), 0);

    // Call to A (backward ref)
    emitter.bytecode.resize(50, 0);
    tracker.emit_call_with_metadata(&mut emitter, 0, 0, "func_a");
    let call_a_pos = emitter.get_position() - 2;
    tracker.record_patch_at_position(call_a_pos, "func_a".to_string());

    // Call to B (forward ref - not defined yet)
    tracker.emit_call_with_metadata(&mut emitter, 1, 0, "func_b");
    let call_b_pos = emitter.get_position() - 2;
    tracker.record_patch_at_position(call_b_pos, "func_b".to_string());

    // Function B is defined at position 150
    tracker.record_function_position("func_b".to_string(), 150);

    // Patch all
    assert!(tracker.patch_calls(&mut emitter).is_ok());

    // Verify both patches
    let bytecode = emitter.get_bytecode();
    let addr_a = u16::from_le_bytes([bytecode[call_a_pos], bytecode[call_a_pos + 1]]);
    let addr_b = u16::from_le_bytes([bytecode[call_b_pos], bytecode[call_b_pos + 1]]);

    assert_eq!(addr_a, 0, "Backward reference should patch to 0");
    assert_eq!(addr_b, 150, "Forward reference should patch to 150");
}

// ============================================================================
// Test Group 6: Error Cases
// ============================================================================

#[test]
fn test_patch_with_missing_function() {
    let mut tracker = CallSiteTracker::new();
    let mut emitter = TestEmitter::new();

    // Record a patch but never define the function
    emitter.bytecode.resize(10, 0);
    tracker.record_patch_at_position(5, "undefined_function".to_string());

    // Patching should fail
    assert!(
        tracker.patch_calls(&mut emitter).is_err(),
        "Should error when function is not defined"
    );
}

#[test]
fn test_patch_address_too_large() {
    let mut tracker = CallSiteTracker::new();
    let mut emitter = TestEmitter::new();

    // Try to patch with address > u16::MAX
    emitter.bytecode.resize(10, 0);
    tracker.record_patch_at_position(5, "far_function".to_string());
    tracker.record_function_position("far_function".to_string(), 0x10000); // > u16::MAX

    // Should fail due to address overflow
    assert!(
        tracker.patch_calls(&mut emitter).is_err(),
        "Should error when address exceeds u16::MAX"
    );
}

// ============================================================================
// Test Group 7: Nested Function Calls
// ============================================================================

#[test]
fn test_nested_call_chain() {
    let mut tracker = CallSiteTracker::new();
    let mut emitter = TestEmitter::new();

    // main() calls helper1(), which calls helper2(), which calls helper3()
    // Simulate this with multiple CALL emissions

    // main at 0
    tracker.record_function_position("main".to_string(), 0);

    // main calls helper1 (forward ref)
    emitter.bytecode.resize(20, 0);
    tracker.emit_call_with_metadata(&mut emitter, 0, 0, "helper1");
    let call1_pos = emitter.get_position() - 2;
    tracker.record_patch_at_position(call1_pos, "helper1".to_string());

    // helper1 at 100
    tracker.record_function_position("helper1".to_string(), 100);
    emitter.bytecode.resize(120, 0);

    // helper1 calls helper2 (forward ref)
    tracker.emit_call_with_metadata(&mut emitter, 0, 0, "helper2");
    let call2_pos = emitter.get_position() - 2;
    tracker.record_patch_at_position(call2_pos, "helper2".to_string());

    // helper2 at 200
    tracker.record_function_position("helper2".to_string(), 200);
    emitter.bytecode.resize(220, 0);

    // helper2 calls helper3 (forward ref)
    tracker.emit_call_with_metadata(&mut emitter, 0, 0, "helper3");
    let call3_pos = emitter.get_position() - 2;
    tracker.record_patch_at_position(call3_pos, "helper3".to_string());

    // helper3 at 300
    tracker.record_function_position("helper3".to_string(), 300);

    // Patch all
    assert!(tracker.patch_calls(&mut emitter).is_ok());

    // Verify chain
    let bytecode = emitter.get_bytecode();
    let addr1 = u16::from_le_bytes([bytecode[call1_pos], bytecode[call1_pos + 1]]);
    let addr2 = u16::from_le_bytes([bytecode[call2_pos], bytecode[call2_pos + 1]]);
    let addr3 = u16::from_le_bytes([bytecode[call3_pos], bytecode[call3_pos + 1]]);

    assert_eq!(addr1, 100);
    assert_eq!(addr2, 200);
    assert_eq!(addr3, 300);
}

// ============================================================================
// Test Group 8: Recursive Calls
// ============================================================================

#[test]
fn test_recursive_function_call() {
    let mut tracker = CallSiteTracker::new();
    let mut emitter = TestEmitter::new();

    // factorial function calls itself
    tracker.record_function_position("factorial".to_string(), 50);

    // Inside factorial, it calls itself (backward ref to same function)
    emitter.bytecode.resize(100, 0);
    tracker.emit_call_with_metadata(&mut emitter, 1, 0, "factorial");
    let call_pos = emitter.get_position() - 2;
    tracker.record_patch_at_position(call_pos, "factorial".to_string());

    // Patch
    assert!(tracker.patch_calls(&mut emitter).is_ok());

    // Verify
    let bytecode = emitter.get_bytecode();
    let patched_address = u16::from_le_bytes([bytecode[call_pos], bytecode[call_pos + 1]]);
    assert_eq!(
        patched_address, 50,
        "Recursive call should patch to function start"
    );
}

// ============================================================================
// Test Group 9: Many Functions
// ============================================================================

#[test]
fn test_many_functions_patching() {
    let mut tracker = CallSiteTracker::new();
    let mut emitter = TestEmitter::new();

    // Create 20 functions, each calling the next
    for i in 0..20 {
        let func_name = format!("func_{}", i);
        let next_func = format!("func_{}", i + 1);

        // Define current function
        tracker.record_function_position(func_name.clone(), i * 100);

        // Current function calls next function (forward ref)
        if i < 19 {
            emitter.bytecode.resize((i * 100) + 50, 0);
            tracker.emit_call_with_metadata(&mut emitter, 0, 0, &next_func);
            let call_pos = emitter.get_position() - 2;
            tracker.record_patch_at_position(call_pos, next_func);
        }
    }

    // Define last function
    tracker.record_function_position("func_20".to_string(), 2000);

    // Patch all
    assert!(tracker.patch_calls(&mut emitter).is_ok());
}

// ============================================================================
// Test Group 10: Integration with Real Compiler Flow
// ============================================================================

#[test]
fn test_realistic_multi_function_program() {
    // Simulates a real program with multiple functions
    let mut tracker = CallSiteTracker::new();
    let mut emitter = TestEmitter::new();

    // Program structure:
    // pub main() -> u64 {
    //     return add(mul(2, 3), 4);
    // }
    //
    // add(a: u64, b: u64) -> u64 {
    //     return a + b;
    // }
    //
    // mul(a: u64, b: u64) -> u64 {
    //     return a * b;
    // }

    // main is public, starts at position 0
    tracker.record_function_position("main".to_string(), 0);

    // In main, we call mul first
    emitter.bytecode.resize(20, 0);
    tracker.emit_call_with_metadata(&mut emitter, 2, 0, "mul");
    let mul_call_pos = emitter.get_position() - 2;
    tracker.record_patch_at_position(mul_call_pos, "mul".to_string());

    // Then call add
    tracker.emit_call_with_metadata(&mut emitter, 2, 0, "add");
    let add_call_pos = emitter.get_position() - 2;
    tracker.record_patch_at_position(add_call_pos, "add".to_string());

    // add is private, at position 100
    tracker.record_function_position("add".to_string(), 100);

    // mul is private, at position 150
    tracker.record_function_position("mul".to_string(), 150);

    // Patch all calls
    assert!(tracker.patch_calls(&mut emitter).is_ok());

    // Verify patches
    let bytecode = emitter.get_bytecode();
    let mul_addr = u16::from_le_bytes([bytecode[mul_call_pos], bytecode[mul_call_pos + 1]]);
    let add_addr = u16::from_le_bytes([bytecode[add_call_pos], bytecode[add_call_pos + 1]]);

    assert_eq!(mul_addr, 150, "mul call should patch to 150");
    assert_eq!(add_addr, 100, "add call should patch to 100");
}
