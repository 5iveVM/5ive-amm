//! Parameter loading helpers.

use crate::ast::InstructionParameter;
use super::OpcodeEmitter;
use five_vm_mito::error::VMError;
use five_protocol::opcodes::*;

/// Load function parameters into local slots.
pub fn load_function_parameters<T: OpcodeEmitter>(
    emitter: &mut T, 
    parameters: &[InstructionParameter]
) -> Result<(), VMError> {
    
    println!("DEBUG: load_function_parameters called with {} parameters", parameters.len());
    
    for (index, param) in parameters.iter().enumerate() {
        println!("DEBUG: Loading parameter {}: {} at index {}", index, param.name, index);
        
        // Load parameter from pre-parsed input data array
        // Use 1-based indexing to match VM expectations (VM translates N → parameters[N-1])
        // Optimized parameter loading using single-byte opcodes
        // 1-based indexing for params (VM expectation), 0-based for locals
        
        let param_index = (index + 1) as u8;
        
        // Optimize LOAD_PARAM
        match param_index {
            1 => emitter.emit_opcode(LOAD_PARAM_1),
            2 => emitter.emit_opcode(LOAD_PARAM_2),
            3 => emitter.emit_opcode(LOAD_PARAM_3),
            _ => {
                emitter.emit_opcode(LOAD_PARAM);
                emitter.emit_u8(param_index);
            }
        }
        
        // Optimize SET_LOCAL
        match index {
            0 => emitter.emit_opcode(SET_LOCAL_0),
            1 => emitter.emit_opcode(SET_LOCAL_1),
            2 => emitter.emit_opcode(SET_LOCAL_2),
            3 => emitter.emit_opcode(SET_LOCAL_3),
            _ => {
                emitter.emit_opcode(SET_LOCAL);
                emitter.emit_u8(index as u8);
            }
        }
        
        println!("DEBUG: Emitted LOAD_PARAM {} -> SET_LOCAL {} for parameter '{}'", 
                 index + 1, index, param.name);
    }
    
    println!("DEBUG: load_function_parameters completed successfully");
    Ok(())
}

/// Check if a list of parameters contains any account parameters
/// 
/// This is a utility function for determining if special account handling
/// is needed during parameter loading.
pub fn has_account_parameters(parameters: &[InstructionParameter]) -> bool {
    parameters.iter().any(|param| {
        matches!(&param.param_type, crate::ast::TypeNode::Account) ||
        matches!(&param.param_type, crate::ast::TypeNode::Named(name) if name == "Account")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{TypeNode, InstructionParameter};
    use std::collections::VecDeque;

    // Mock emitter for testing
    struct MockEmitter {
        opcodes: VecDeque<u8>,
    }

    impl MockEmitter {
        fn new() -> Self {
            Self { opcodes: VecDeque::new() }
        }

        fn get_opcodes(&self) -> Vec<u8> {
            self.opcodes.iter().cloned().collect()
        }
    }

    impl OpcodeEmitter for MockEmitter {
        fn emit_opcode(&mut self, opcode: u8) {
            self.opcodes.push_back(opcode);
        }

        fn emit_u8(&mut self, value: u8) {
            self.opcodes.push_back(value);
        }

        fn emit_u16(&mut self, value: u16) {
            self.opcodes.push_back((value & 0xFF) as u8);
            self.opcodes.push_back((value >> 8) as u8);
        }

        fn emit_u32(&mut self, value: u32) {
            self.opcodes.push_back((value & 0xFF) as u8);
            self.opcodes.push_back(((value >> 8) & 0xFF) as u8);
            self.opcodes.push_back(((value >> 16) & 0xFF) as u8);
            self.opcodes.push_back((value >> 24) as u8);
        }

        fn emit_u64(&mut self, value: u64) {
            for i in 0..8 {
                self.opcodes.push_back(((value >> (i * 8)) & 0xFF) as u8);
            }
        }

        fn emit_bytes(&mut self, bytes: &[u8]) {
            for b in bytes {
                self.opcodes.push_back(*b);
            }
        }

        fn get_position(&self) -> usize {
            self.opcodes.len()
        }

        fn patch_u32(&mut self, position: usize, value: u32) {
            let bytes = value.to_le_bytes();
            let slice = self.opcodes.make_contiguous();
            if position + 4 <= slice.len() {
                slice[position..position + 4].copy_from_slice(&bytes);
            }
        }

        fn patch_u16(&mut self, position: usize, value: u16) {
            let bytes = value.to_le_bytes();
            let slice = self.opcodes.make_contiguous();
            if position + 2 <= slice.len() {
                slice[position..position + 2].copy_from_slice(&bytes);
            }
        }

        fn should_include_tests(&self) -> bool {
            true
        }

        fn emit_const_u8(&mut self, _value: u8) -> Result<(), VMError> {
            Ok(())
        }
        fn emit_const_u16(&mut self, _value: u16) -> Result<(), VMError> {
            Ok(())
        }
        fn emit_const_u32(&mut self, _value: u32) -> Result<(), VMError> {
            Ok(())
        }
        fn emit_const_u64(&mut self, _value: u64) -> Result<(), VMError> {
            Ok(())
        }
        fn emit_const_i64(&mut self, _value: i64) -> Result<(), VMError> {
            Ok(())
        }
        fn emit_const_bool(&mut self, _value: bool) -> Result<(), VMError> {
            Ok(())
        }
        fn emit_const_u128(&mut self, _value: u128) -> Result<(), VMError> {
            Ok(())
        }
        fn emit_const_pubkey(&mut self, _value: &[u8; 32]) -> Result<(), VMError> {
            Ok(())
        }
        fn emit_const_string(&mut self, _value: &[u8]) -> Result<(), VMError> {
            Ok(())
        }
    }

    #[test]
    fn test_no_parameters() {
        let mut emitter = MockEmitter::new();
        let parameters = vec![];

        let result = load_function_parameters(&mut emitter, &parameters);
        assert!(result.is_ok());
        assert_eq!(emitter.get_opcodes(), vec![]);
    }

    #[test]
    fn test_single_parameter() {
        let mut emitter = MockEmitter::new();
        let parameters = vec![
            InstructionParameter {
                name: "value".to_string(),
                param_type: TypeNode::Primitive("u64".to_string()),
                is_optional: false,
                default_value: None,
                attributes: vec![],
                is_init: false,
                init_config: None,
                    pda_config: None,
            }
        ];

        let result = load_function_parameters(&mut emitter, &parameters);
        assert!(result.is_ok());
        
        let expected = vec![
            LOAD_PARAM, 1,  // Load parameter 0 using VM's 1-based indexing
            SET_LOCAL, 0,   // Store in local variable 0
        ];
        assert_eq!(emitter.get_opcodes(), expected);
    }

    #[test]
    fn test_multiple_parameters() {
        let mut emitter = MockEmitter::new();
        let parameters = vec![
            InstructionParameter {
                name: "a".to_string(),
                param_type: TypeNode::Primitive("u64".to_string()),
                is_optional: false,
                default_value: None,
                attributes: vec![],
                is_init: false,
                init_config: None,
                    pda_config: None,
            },
            InstructionParameter {
                name: "b".to_string(),
                param_type: TypeNode::Primitive("u64".to_string()),
                is_optional: false,
                default_value: None,
                attributes: vec![],
                is_init: false,
                init_config: None,
                    pda_config: None,
            }
        ];

        let result = load_function_parameters(&mut emitter, &parameters);
        assert!(result.is_ok());
        
        let expected = vec![
            LOAD_PARAM, 1,  // Load parameter 0 using VM's 1-based indexing
            SET_LOCAL, 0,   // Store in local variable 0
            LOAD_PARAM, 2,  // Load parameter 1 using VM's 1-based indexing
            SET_LOCAL, 1,   // Store in local variable 1
        ];
        assert_eq!(emitter.get_opcodes(), expected);
    }

    #[test]
    fn test_has_account_parameters() {
        let account_param = InstructionParameter {
            name: "signer".to_string(),
            param_type: TypeNode::Account,
            is_optional: false,
            default_value: None,
            attributes: vec![],
            is_init: false,
            init_config: None,
                    pda_config: None,
        };

        let regular_param = InstructionParameter {
            name: "amount".to_string(),
            param_type: TypeNode::Primitive("u64".to_string()),
            is_optional: false,
            default_value: None,
            attributes: vec![],
            is_init: false,
            init_config: None,
                    pda_config: None,
        };

        assert!(!has_account_parameters(&[]));
        assert!(!has_account_parameters(&[regular_param.clone()]));
        assert!(has_account_parameters(&[account_param.clone()]));
        assert!(has_account_parameters(&[regular_param, account_param]));
    }
}
