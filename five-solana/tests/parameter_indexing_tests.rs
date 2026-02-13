//! Execute payload parsing alignment tests.
//!
//! These tests lock `EXECUTE_INSTRUCTION` payload shape to the canonical VM
//! envelope:
//! [function_index:u32 LE][param_count:u32 LE][typed params...]

#[cfg(test)]
mod parameter_indexing_tests {
    use five::instructions::{FIVEInstruction, EXECUTE_INSTRUCTION};
    use five_protocol::types;

    fn encode_execute_payload(function_index: u32, param_count: u32, typed_params: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&function_index.to_le_bytes());
        payload.extend_from_slice(&param_count.to_le_bytes());
        payload.extend_from_slice(typed_params);
        payload
    }

    fn wrap_execute(payload: &[u8]) -> Vec<u8> {
        let mut out = vec![EXECUTE_INSTRUCTION];
        out.extend_from_slice(payload);
        out
    }

    #[test]
    fn execute_instruction_minimal_canonical_payload() {
        let payload = encode_execute_payload(0, 0, &[]);
        let exec_data = wrap_execute(&payload);
        let parsed = FIVEInstruction::try_from(exec_data.as_slice()).expect("should parse");
        if let FIVEInstruction::Execute { params } = parsed {
            assert_eq!(params, payload.as_slice());
            assert_eq!(u32::from_le_bytes(params[0..4].try_into().unwrap()), 0);
            assert_eq!(u32::from_le_bytes(params[4..8].try_into().unwrap()), 0);
        } else {
            panic!("expected execute instruction");
        }
    }

    #[test]
    fn execute_instruction_typed_u64_param_payload() {
        let mut typed = vec![types::U64];
        typed.extend_from_slice(&10u64.to_le_bytes());
        let payload = encode_execute_payload(3, 1, &typed);
        let exec_data = wrap_execute(&payload);

        let parsed = FIVEInstruction::try_from(exec_data.as_slice()).expect("should parse");
        if let FIVEInstruction::Execute { params } = parsed {
            assert_eq!(params, payload.as_slice());
            assert_eq!(u32::from_le_bytes(params[0..4].try_into().unwrap()), 3);
            assert_eq!(u32::from_le_bytes(params[4..8].try_into().unwrap()), 1);
            assert_eq!(params[8], types::U64);
            assert_eq!(u64::from_le_bytes(params[9..17].try_into().unwrap()), 10);
        } else {
            panic!("expected execute instruction");
        }
    }

    #[test]
    fn execute_instruction_multiple_typed_params_payload() {
        let mut typed = Vec::new();
        typed.push(types::U64);
        typed.extend_from_slice(&100u64.to_le_bytes());
        typed.push(types::BOOL);
        typed.extend_from_slice(&1u32.to_le_bytes());
        let payload = encode_execute_payload(3, 2, &typed);
        let exec_data = wrap_execute(&payload);

        let parsed = FIVEInstruction::try_from(exec_data.as_slice()).expect("should parse");
        if let FIVEInstruction::Execute { params } = parsed {
            assert_eq!(params, payload.as_slice());
            assert_eq!(u32::from_le_bytes(params[0..4].try_into().unwrap()), 3);
            assert_eq!(u32::from_le_bytes(params[4..8].try_into().unwrap()), 2);
            assert_eq!(params[8], types::U64);
            assert_eq!(params[17], types::BOOL);
        } else {
            panic!("expected execute instruction");
        }
    }

    #[test]
    fn execute_instruction_payload_is_opaque_to_parser() {
        // Execute parsing should preserve payload bytes exactly; VM validates semantics.
        let payload = vec![0xAA, 0xBB, 0xCC];
        let exec_data = wrap_execute(&payload);

        let parsed = FIVEInstruction::try_from(exec_data.as_slice()).expect("should parse");
        if let FIVEInstruction::Execute { params } = parsed {
            assert_eq!(params, payload.as_slice());
        } else {
            panic!("expected execute instruction");
        }
    }
}

#[cfg(test)]
mod comprehensive_instruction_tests {
    use five::instructions::{DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION, FIVEInstruction};
    use five_protocol::{bytecode, types};

    fn encode_execute_payload(function_index: u32, param_count: u32, typed_params: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&function_index.to_le_bytes());
        payload.extend_from_slice(&param_count.to_le_bytes());
        payload.extend_from_slice(typed_params);
        payload
    }

    #[test]
    fn deploy_instruction_parsing_still_works() {
        let bytecode = bytecode!(emit_header(0, 0), emit_halt());
        let permissions = 0u8;
        let mut deploy_data = vec![DEPLOY_INSTRUCTION];
        deploy_data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
        deploy_data.push(permissions);
        deploy_data.extend_from_slice(&0u32.to_le_bytes()); // metadata_len
        deploy_data.extend_from_slice(&bytecode);

        let result = FIVEInstruction::try_from(&deploy_data[..]);
        assert!(result.is_ok(), "deploy instruction should parse");
    }

    #[test]
    fn execute_instruction_with_varying_param_counts_uses_u32_header() {
        for param_count in 0..=3u32 {
            let mut typed = Vec::new();
            for i in 0..param_count {
                typed.push(types::U64);
                typed.extend_from_slice(&(1000u64 + i as u64).to_le_bytes());
            }

            let payload = encode_execute_payload(1, param_count, &typed);
            let mut exec_data = vec![EXECUTE_INSTRUCTION];
            exec_data.extend_from_slice(&payload);

            let result = FIVEInstruction::try_from(&exec_data[..]);
            assert!(result.is_ok(), "should parse canonical payload for count={param_count}");
        }
    }
}
