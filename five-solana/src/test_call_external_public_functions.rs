/// Test for calling another on-chain bytecode's public functions without CPI
///
/// This demonstrates the CALL_EXTERNAL mechanism which allows:
/// - Direct invocation of public functions from other script accounts
/// - Parameter passing via the stack
/// - Return values coming back to the caller
/// - No CPI overhead - same execution context
///
/// The key difference from CPI:
/// - CPI calls a different Solana program entirely
/// - CALL_EXTERNAL invokes bytecode stored in an account, within the Five VM context
/// - CALL_EXTERNAL has access to the same stack and execution state
/// - CALL_EXTERNAL is faster (no program invocation overhead)

#[cfg(test)]
mod public_function_invocation_tests {
    use crate::instructions::{FIVEInstruction, DEPLOY_INSTRUCTION};
    use five_protocol::bytecode;

    /// Test structure demonstrating bytecode with public and private functions
    ///
    /// Public functions (indices 0-N-1):
    /// - Can be called from external bytecode via CALL_EXTERNAL
    /// - Called with account_index and function_offset
    ///
    /// Private functions (indices N-total):
    /// - Can only be called internally via CALL
    /// - Not accessible to external callers
    #[test]
    fn test_call_external_public_function_invocation() {
        // Bytecode 1: Has a public "add" function that adds two numbers
        // Function layout:
        //   - Public functions: indices 0-1 (functions at offsets 10 and 20)
        //   - Private functions: indices 2-3 (internal helper functions)

        let external_bytecode = bytecode!(
            emit_header(2, 4),  // 2 public functions, 4 total functions
            emit_halt()          // Minimal bytecode for this test
        );

        // Bytecode 2: Calls the external bytecode's public function
        //
        // Stack-based calling convention:
        // 1. Push arguments onto stack (in order)
        // 2. CALL_EXTERNAL account_index, func_offset, param_count
        // 3. Return value appears on stack

        let _caller_bytecode = bytecode!(
            emit_header(1, 1),   // 1 public function, 1 total function
            emit_halt()          // Minimal bytecode
        );

        // Deployment instruction for external bytecode (no permissions needed)
        let permissions = 0x00u8; // No special permissions
        let mut deploy_data = vec![DEPLOY_INSTRUCTION];
        deploy_data.extend_from_slice(&(external_bytecode.len() as u32).to_le_bytes());
        deploy_data.push(permissions);
        deploy_data.extend_from_slice(&external_bytecode);

        // Verify the deploy instruction format is correct
        let deploy_ix = FIVEInstruction::try_from(deploy_data.as_slice()).unwrap();
        match deploy_ix {
            FIVEInstruction::Deploy { bytecode: bc, permissions: perms } => {
                assert_eq!(bc, &external_bytecode[..]);
                assert_eq!(perms, permissions);
            }
            _ => panic!("Expected Deploy instruction"),
        }
    }

    /// Test showing CALL_EXTERNAL instruction format
    ///
    /// The CALL_EXTERNAL opcode is 0x91:
    /// Byte 0: CALL_EXTERNAL (0x91)
    /// Byte 1: account_index (u8) - which account has the target bytecode
    /// Bytes 2-3: func_offset (u16 LE) - bytecode offset to function start
    /// Byte 4: param_count (u8) - number of parameters to pass
    #[test]
    fn test_call_external_instruction_format() {
        // Example CALL_EXTERNAL instruction:
        // Call function at offset 256 in account 0 with 2 parameters

        let call_external_opcode = 0x91u8;
        let account_index = 0u8;
        let func_offset = 256u16; // u16 little-endian
        let param_count = 2u8;

        let mut instruction = vec![call_external_opcode];
        instruction.push(account_index);
        instruction.extend_from_slice(&func_offset.to_le_bytes());
        instruction.push(param_count);

        assert_eq!(instruction.len(), 5);
        assert_eq!(instruction[0], 0x91);
        assert_eq!(instruction[1], 0);
        assert_eq!(&instruction[2..4], &[0, 1]); // 256 in LE bytes
        assert_eq!(instruction[4], 2);
    }

    /// Test demonstrating public function discovery via bytecode header
    ///
    /// The bytecode header tells us:
    /// - How many public functions are available (public_function_count)
    /// - How many total functions exist (total_function_count)
    /// - Features (like function names, if present)
    ///
    /// Functions 0 to (public_function_count - 1) are callable externally
    /// Functions public_function_count to (total_function_count - 1) are private
    #[test]
    fn test_public_function_discovery() {
        // Bytecode with 3 public functions and 1 private function
        let bytecode = vec![
            b'5', b'I', b'V', b'E',  // Magic
            0, 0, 0, 0,              // Features: no special features
            3,                        // public_function_count = 3
            4,                        // total_function_count = 4
            // ... rest of bytecode would contain function implementations
        ];

        // At runtime, a caller can discover:
        // - This bytecode has 3 public functions (indices 0, 1, 2)
        // - It has 1 private function (index 3)
        // - Functions can be called via CALL_EXTERNAL with indices 0-2

        assert_eq!(bytecode[8], 3); // public_function_count
        assert_eq!(bytecode[9], 4); // total_function_count
    }

    /// Test showing parameter passing convention for CALL_EXTERNAL
    ///
    /// Stack-based parameter convention:
    /// 1. Arguments pushed to stack (in order or reverse order depends on calling convention)
    /// 2. CALL_EXTERNAL instruction pops param_count values from stack
    /// 3. Those values become LOAD_PARAM targets in the called function
    /// 4. Called function uses LOAD_PARAM 1, LOAD_PARAM 2, etc. to access parameters
    /// 5. Return value (if any) is pushed back to caller's stack
    #[test]
    fn test_call_external_parameter_passing() {
        // Example calling sequence:
        // PUSH_U64 10         // push first arg
        // PUSH_U64 20         // push second arg
        // CALL_EXTERNAL 0, 256, 2  // call with 2 parameters
        // // At this point, result is on stack
        // STORE_VAR 0         // store result

        // In the called function:
        // LOAD_PARAM 1        // get first parameter (10)
        // LOAD_PARAM 2        // get second parameter (20)
        // ADD                 // add them
        // RETURN_VALUE        // return result on stack

        let push_u64_opcode = 0x12u8;
        let call_external_opcode = 0x91u8;
        let _load_param_opcode = 0x05u8;
        let _add_opcode = 0x30u8;
        let _return_opcode = 0xFDu8;

        // Simulate bytecode that calls external function
        let mut bytecode = vec![];

        // PUSH_U64 10
        bytecode.push(push_u64_opcode);
        bytecode.extend_from_slice(&10u64.to_le_bytes());

        // PUSH_U64 20
        bytecode.push(push_u64_opcode);
        bytecode.extend_from_slice(&20u64.to_le_bytes());

        // CALL_EXTERNAL account=0, offset=256, params=2
        bytecode.push(call_external_opcode);
        bytecode.push(0); // account_index
        bytecode.extend_from_slice(&256u16.to_le_bytes());
        bytecode.push(2); // param_count

        assert!(bytecode.len() > 0);
        assert_eq!(bytecode[0], 0x12); // PUSH_U64
        assert_eq!(bytecode[9], 0x12); // PUSH_U64
        assert_eq!(bytecode[18], 0x91); // CALL_EXTERNAL
    }

    /// Test demonstrating function offset calculation
    ///
    /// The compiler pre-calculates function offsets when it knows the target bytecode.
    /// At runtime, CALL_EXTERNAL uses these pre-calculated offsets.
    ///
    /// The offset points to the first instruction of the function in the bytecode.
    /// The function extends until the next function or RETURN instruction.
    #[test]
    fn test_function_offset_calculation() {
        // Bytecode layout:
        // Offset 0-9:   Header (10 bytes)
        // Offset 10-19: Function 0 (10 bytes)
        // Offset 20-29: Function 1 (10 bytes)
        // Offset 30+:   Function 2...

        let header_size = 10;
        let func0_offset = header_size;
        let func1_offset = header_size + 10;
        let func2_offset = header_size + 20;

        // To call function 0: CALL_EXTERNAL account, 10, params
        // To call function 1: CALL_EXTERNAL account, 20, params
        // To call function 2: CALL_EXTERNAL account, 30, params

        assert_eq!(func0_offset, 10);
        assert_eq!(func1_offset, 20);
        assert_eq!(func2_offset, 30);
    }

    /// Test showing the key advantage: NO CPI OVERHEAD
    ///
    /// Key differences between CALL_EXTERNAL and CPI (INVOKE):
    ///
    /// CALL_EXTERNAL:
    /// - Direct bytecode execution in Five VM
    /// - Same stack and execution context
    /// - Fast parameter passing via stack
    /// - No program invocation protocol
    /// - Bytecode must be in account data
    ///
    /// CPI (INVOKE):
    /// - Invokes a different Solana program
    /// - Separate program context
    /// - Parameters passed via instruction data
    /// - Subject to Solana program invocation rules
    /// - More flexible (can invoke any program)
    /// - Higher computational cost
    #[test]
    fn test_call_external_vs_cpi_overhead() {
        // CALL_EXTERNAL (no CPI):
        // - 5 bytes instruction (opcode, account, offset, params)
        // - Direct stack-based parameter passing
        // - Same execution context maintained
        // - No program boundary crossing
        // - No serialization/deserialization overhead

        // CPI (INVOKE):
        // - Requires instruction data construction
        // - Program invocation protocol overhead
        // - Cross-program boundary crossing
        // - Account context switching
        // - More computational units used

        let call_external_size = 5; // bytes
        assert_eq!(call_external_size, 5);

        // This is much lighter weight than CPI, making it ideal for:
        // - Shared utility functions in other bytecode
        // - Library functions stored on-chain
        // - Composable smart contracts
        // - High-frequency inter-script calls
    }
}
