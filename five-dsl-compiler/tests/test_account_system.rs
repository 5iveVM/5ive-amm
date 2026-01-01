use five_dsl_compiler::ast::{StructField, TypeNode};
/// Account System Test Suite
///
/// Tests the account_system module which handles:
/// - Account definition processing
/// - Field offset calculation
/// - Field access code generation
/// - Built-in property access (lamports, owner, key, data)
/// - Account type validation
/// - Field info lookups
use five_dsl_compiler::bytecode_generator::{AccountSystem, FieldInfo, OpcodeEmitter};
use five_protocol::opcodes::{LOAD_FIELD, OPTIONAL_UNWRAP};
use std::collections::HashMap;

// Simple test emitter (reusing from call_patching tests)
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

    fn emit_vle_u32(&mut self, value: u32) {
        use five_protocol::VLE;
        let (size, bytes) = VLE::encode_u32(value);
        for i in 0..size {
            self.emit_u8(bytes[i]);
        }
    }

    fn emit_vle_u16(&mut self, value: u16) {
        use five_protocol::VLE;
        let (size, bytes) = VLE::encode_u16(value);
        for i in 0..size {
            self.emit_u8(bytes[i]);
        }
    }

    fn emit_vle_u64(&mut self, value: u64) {
        use five_protocol::VLE;
        let (size, bytes) = VLE::encode_u64(value);
        for i in 0..size {
            self.emit_u8(bytes[i]);
        }
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
}

// ============================================================================
// Test Group 1: Account System Creation & Configuration
// ============================================================================

#[test]
fn test_account_system_creation() {
    let account_system = AccountSystem::new();
    // Should create with default configuration
    assert!(account_system.is_builtin_account_property("lamports"));
    assert!(account_system.is_builtin_account_property("owner"));
    assert!(account_system.is_builtin_account_property("key"));
    assert!(account_system.is_builtin_account_property("data"));
}

#[test]
fn test_zerocopy_configuration() {
    let mut account_system = AccountSystem::new();

    // Test enabling/disabling zerocopy
    account_system.set_zerocopy_enabled(false);
    account_system.set_zerocopy_enabled(true);

    // Should not crash or error
}

#[test]
fn test_builtin_properties_recognized() {
    let account_system = AccountSystem::new();

    assert!(account_system.is_builtin_account_property("lamports"));
    assert!(account_system.is_builtin_account_property("owner"));
    assert!(account_system.is_builtin_account_property("key"));
    assert!(account_system.is_builtin_account_property("data"));

    assert!(!account_system.is_builtin_account_property("balance"));
    assert!(!account_system.is_builtin_account_property("invalid"));
    assert!(!account_system.is_builtin_account_property(""));
}

// ============================================================================
// Test Group 2: Account Definition Processing
// ============================================================================

#[test]
fn test_process_simple_account_definition() {
    let mut account_system = AccountSystem::new();

    let fields = vec![
        StructField {
            name: "amount".to_string(),
            field_type: TypeNode::Primitive("u64".to_string()),
            is_mutable: true,
            is_optional: false,
        },
        StructField {
            name: "recipient".to_string(),
            field_type: TypeNode::Primitive("pubkey".to_string()),
            is_mutable: false,
            is_optional: false,
        },
    ];

    let result = account_system.process_account_definition("Transfer", &fields);
    assert!(result.is_ok(), "Should process account definition");

    // Verify account was registered
    assert!(account_system.is_account_type("Transfer"));
}

#[test]
fn test_account_field_offset_calculation() {
    let mut account_system = AccountSystem::new();

    let fields = vec![
        StructField {
            name: "field1".to_string(),
            field_type: TypeNode::Primitive("u64".to_string()), // 8 bytes
            is_mutable: true,
            is_optional: false,
        },
        StructField {
            name: "field2".to_string(),
            field_type: TypeNode::Primitive("u32".to_string()), // 4 bytes
            is_mutable: true,
            is_optional: false,
        },
        StructField {
            name: "field3".to_string(),
            field_type: TypeNode::Primitive("u64".to_string()), // 8 bytes
            is_mutable: true,
            is_optional: false,
        },
    ];

    account_system
        .process_account_definition("TestAccount", &fields)
        .unwrap();

    // Verify offsets
    let field1_info = account_system.get_field_info("TestAccount", "field1");
    let field2_info = account_system.get_field_info("TestAccount", "field2");
    let field3_info = account_system.get_field_info("TestAccount", "field3");

    assert!(field1_info.is_some());
    assert!(field2_info.is_some());
    assert!(field3_info.is_some());

    assert_eq!(
        field1_info.unwrap().offset,
        0,
        "field1 should be at offset 0"
    );
    assert_eq!(
        field2_info.unwrap().offset,
        8,
        "field2 should be at offset 8 (after u64)"
    );
    assert_eq!(
        field3_info.unwrap().offset,
        12,
        "field3 should be at offset 12 (after u32)"
    );
}

#[test]
fn test_multiple_account_definitions() {
    let mut account_system = AccountSystem::new();

    // Define Transfer account
    let transfer_fields = vec![StructField {
        name: "amount".to_string(),
        field_type: TypeNode::Primitive("u64".to_string()),
        is_mutable: true,
        is_optional: false,
    }];
    account_system
        .process_account_definition("Transfer", &transfer_fields)
        .unwrap();

    // Define Vault account
    let vault_fields = vec![
        StructField {
            name: "balance".to_string(),
            field_type: TypeNode::Primitive("u64".to_string()),
            is_mutable: true,
            is_optional: false,
        },
        StructField {
            name: "owner".to_string(),
            field_type: TypeNode::Primitive("pubkey".to_string()),
            is_mutable: false,
            is_optional: false,
        },
    ];
    account_system
        .process_account_definition("Vault", &vault_fields)
        .unwrap();

    // Both should be registered
    assert!(account_system.is_account_type("Transfer"));
    assert!(account_system.is_account_type("Vault"));

    // Should be able to look up fields from both
    assert!(account_system
        .get_field_info("Transfer", "amount")
        .is_some());
    assert!(account_system.get_field_info("Vault", "balance").is_some());
    assert!(account_system.get_field_info("Vault", "owner").is_some());
}

#[test]
fn test_optional_field_handling() {
    let mut account_system = AccountSystem::new();

    let fields = vec![
        StructField {
            name: "required_field".to_string(),
            field_type: TypeNode::Primitive("u64".to_string()),
            is_mutable: true,
            is_optional: false,
        },
        StructField {
            name: "optional_field".to_string(),
            field_type: TypeNode::Primitive("u64".to_string()),
            is_mutable: true,
            is_optional: true,
        },
    ];

    account_system
        .process_account_definition("TestAccount", &fields)
        .unwrap();

    let required_info = account_system
        .get_field_info("TestAccount", "required_field")
        .unwrap();
    let optional_info = account_system
        .get_field_info("TestAccount", "optional_field")
        .unwrap();

    assert!(!required_info.is_optional);
    assert!(optional_info.is_optional);
}

#[test]
fn test_mutable_vs_immutable_fields() {
    let mut account_system = AccountSystem::new();

    let fields = vec![
        StructField {
            name: "mutable_field".to_string(),
            field_type: TypeNode::Primitive("u64".to_string()),
            is_mutable: true,
            is_optional: false,
        },
        StructField {
            name: "immutable_field".to_string(),
            field_type: TypeNode::Primitive("u64".to_string()),
            is_mutable: false,
            is_optional: false,
        },
    ];

    account_system
        .process_account_definition("TestAccount", &fields)
        .unwrap();

    let mutable_info = account_system
        .get_field_info("TestAccount", "mutable_field")
        .unwrap();
    let immutable_info = account_system
        .get_field_info("TestAccount", "immutable_field")
        .unwrap();

    assert!(mutable_info.is_mutable);
    assert!(!immutable_info.is_mutable);
}

// ============================================================================
// Test Group 3: Field Access Code Generation
// ============================================================================

#[test]
fn test_generate_simple_field_access() {
    let mut account_system = AccountSystem::new();
    let mut emitter = TestEmitter::new();

    // Define account
    let fields = vec![StructField {
        name: "amount".to_string(),
        field_type: TypeNode::Primitive("u64".to_string()),
        is_mutable: true,
        is_optional: false,
    }];
    account_system
        .process_account_definition("Transfer", &fields)
        .unwrap();

    // Create symbol table with account parameter
    let mut symbol_table = HashMap::new();
    symbol_table.insert(
        "transfer_account".to_string(),
        FieldInfo {
            offset: 0, // Account is parameter 0
            field_type: "Transfer".to_string(),
            is_mutable: true,
            is_optional: false,
            is_parameter: true,
        },
    );

    // Generate field access
    let result = account_system.generate_account_field_access(
        &mut emitter,
        "transfer_account",
        "Transfer",
        "amount",
        &symbol_table,
    );

    assert!(result.is_ok(), "Should generate field access");

    let bytecode = emitter.get_bytecode();
    assert!(!bytecode.is_empty(), "Should emit bytecode");
    assert_eq!(bytecode[0], LOAD_FIELD, "Should emit LOAD_FIELD opcode");
}

#[test]
fn test_generate_optional_field_access() {
    let mut account_system = AccountSystem::new();
    let mut emitter = TestEmitter::new();

    // Define account with optional field
    let fields = vec![StructField {
        name: "maybe_value".to_string(),
        field_type: TypeNode::Primitive("u64".to_string()),
        is_mutable: true,
        is_optional: true,
    }];
    account_system
        .process_account_definition("TestAccount", &fields)
        .unwrap();

    // Create symbol table
    let mut symbol_table = HashMap::new();
    symbol_table.insert(
        "test_account".to_string(),
        FieldInfo {
            offset: 0,
            field_type: "TestAccount".to_string(),
            is_mutable: true,
            is_optional: false,
            is_parameter: true,
        },
    );

    // Generate field access
    account_system
        .generate_account_field_access(
            &mut emitter,
            "test_account",
            "TestAccount",
            "maybe_value",
            &symbol_table,
        )
        .unwrap();

    let bytecode = emitter.get_bytecode();

    // Should contain OPTIONAL_UNWRAP after LOAD_FIELD
    let has_unwrap = bytecode.contains(&OPTIONAL_UNWRAP);
    assert!(has_unwrap, "Should emit OPTIONAL_UNWRAP for optional field");
}

#[test]
fn test_field_access_with_multiple_accounts() {
    let mut account_system = AccountSystem::new();
    let mut emitter = TestEmitter::new();

    // Define two account types
    let fields1 = vec![StructField {
        name: "field1".to_string(),
        field_type: TypeNode::Primitive("u64".to_string()),
        is_mutable: true,
        is_optional: false,
    }];
    account_system
        .process_account_definition("Account1", &fields1)
        .unwrap();

    let fields2 = vec![StructField {
        name: "field2".to_string(),
        field_type: TypeNode::Primitive("u32".to_string()),
        is_mutable: true,
        is_optional: false,
    }];
    account_system
        .process_account_definition("Account2", &fields2)
        .unwrap();

    // Symbol table with both accounts
    let mut symbol_table = HashMap::new();
    symbol_table.insert(
        "account1".to_string(),
        FieldInfo {
            offset: 0,
            field_type: "Account1".to_string(),
            is_mutable: true,
            is_optional: false,
            is_parameter: true,
        },
    );
    symbol_table.insert(
        "account2".to_string(),
        FieldInfo {
            offset: 1,
            field_type: "Account2".to_string(),
            is_mutable: true,
            is_optional: false,
            is_parameter: true,
        },
    );

    // Access field from first account
    account_system
        .generate_account_field_access(
            &mut emitter,
            "account1",
            "Account1",
            "field1",
            &symbol_table,
        )
        .unwrap();

    let pos1 = emitter.get_position();

    // Access field from second account
    account_system
        .generate_account_field_access(
            &mut emitter,
            "account2",
            "Account2",
            "field2",
            &symbol_table,
        )
        .unwrap();

    let bytecode = emitter.get_bytecode();
    assert!(pos1 < bytecode.len(), "Should emit for both accounts");
}

// ============================================================================
// Test Group 4: Built-in Property Access
// ============================================================================

#[test]
fn test_builtin_lamports_access() {
    let account_system = AccountSystem::new();
    let mut emitter = TestEmitter::new();

    let mut symbol_table = HashMap::new();
    symbol_table.insert(
        "vault".to_string(),
        FieldInfo {
            offset: 0,
            field_type: "SomeAccount".to_string(),
            is_mutable: true,
            is_optional: false,
            is_parameter: true,
        },
    );

    let result = account_system.generate_builtin_account_property_access(
        &mut emitter,
        "vault",
        "lamports",
        &symbol_table,
    );

    assert!(result.is_ok(), "Should generate lamports access");
    assert!(!emitter.get_bytecode().is_empty());
}

#[test]
fn test_builtin_owner_access() {
    let account_system = AccountSystem::new();
    let mut emitter = TestEmitter::new();

    let mut symbol_table = HashMap::new();
    symbol_table.insert(
        "vault".to_string(),
        FieldInfo {
            offset: 0,
            field_type: "SomeAccount".to_string(),
            is_mutable: true,
            is_optional: false,
            is_parameter: true,
        },
    );

    let result = account_system.generate_builtin_account_property_access(
        &mut emitter,
        "vault",
        "owner",
        &symbol_table,
    );

    assert!(result.is_ok(), "Should generate owner access");
    assert!(!emitter.get_bytecode().is_empty());
}

#[test]
fn test_builtin_key_access() {
    let account_system = AccountSystem::new();
    let mut emitter = TestEmitter::new();

    let mut symbol_table = HashMap::new();
    symbol_table.insert(
        "vault".to_string(),
        FieldInfo {
            offset: 0,
            field_type: "SomeAccount".to_string(),
            is_mutable: true,
            is_optional: false,
            is_parameter: true,
        },
    );

    let result = account_system.generate_builtin_account_property_access(
        &mut emitter,
        "vault",
        "key",
        &symbol_table,
    );

    assert!(result.is_ok(), "Should generate key access");
    assert!(!emitter.get_bytecode().is_empty());
}

#[test]
fn test_builtin_data_access() {
    let account_system = AccountSystem::new();
    let mut emitter = TestEmitter::new();

    let mut symbol_table = HashMap::new();
    symbol_table.insert(
        "vault".to_string(),
        FieldInfo {
            offset: 0,
            field_type: "SomeAccount".to_string(),
            is_mutable: true,
            is_optional: false,
            is_parameter: true,
        },
    );

    let result = account_system.generate_builtin_account_property_access(
        &mut emitter,
        "vault",
        "data",
        &symbol_table,
    );

    assert!(result.is_ok(), "Should generate data access");
    assert!(!emitter.get_bytecode().is_empty());
}

// ============================================================================
// Test Group 5: Account Type Validation
// ============================================================================

#[test]
fn test_validate_registered_account_type() {
    let mut account_system = AccountSystem::new();

    let fields = vec![StructField {
        name: "amount".to_string(),
        field_type: TypeNode::Primitive("u64".to_string()),
        is_mutable: true,
        is_optional: false,
    }];
    account_system
        .process_account_definition("Transfer", &fields)
        .unwrap();

    assert!(account_system.validate_account_type("Transfer"));
    assert!(!account_system.validate_account_type("NonExistent"));
}

#[test]
fn test_is_account_type() {
    let mut account_system = AccountSystem::new();

    let fields = vec![StructField {
        name: "balance".to_string(),
        field_type: TypeNode::Primitive("u64".to_string()),
        is_mutable: true,
        is_optional: false,
    }];
    account_system
        .process_account_definition("Vault", &fields)
        .unwrap();

    assert!(account_system.is_account_type("Vault"));
    assert!(!account_system.is_account_type("Wallet"));
    assert!(!account_system.is_account_type(""));
}

// ============================================================================
// Test Group 6: Field Info Lookups
// ============================================================================

#[test]
fn test_get_field_info_success() {
    let mut account_system = AccountSystem::new();

    let fields = vec![StructField {
        name: "amount".to_string(),
        field_type: TypeNode::Primitive("u64".to_string()),
        is_mutable: true,
        is_optional: false,
    }];
    account_system
        .process_account_definition("Transfer", &fields)
        .unwrap();

    let field_info = account_system.get_field_info("Transfer", "amount");
    assert!(field_info.is_some());

    let info = field_info.unwrap();
    assert_eq!(info.offset, 0);
    assert_eq!(info.field_type, "u64");
    assert!(info.is_mutable);
    assert!(!info.is_optional);
}

#[test]
fn test_get_field_info_nonexistent_account() {
    let account_system = AccountSystem::new();

    let field_info = account_system.get_field_info("NonExistent", "field");
    assert!(field_info.is_none());
}

#[test]
fn test_get_field_info_nonexistent_field() {
    let mut account_system = AccountSystem::new();

    let fields = vec![StructField {
        name: "amount".to_string(),
        field_type: TypeNode::Primitive("u64".to_string()),
        is_mutable: true,
        is_optional: false,
    }];
    account_system
        .process_account_definition("Transfer", &fields)
        .unwrap();

    let field_info = account_system.get_field_info("Transfer", "nonexistent_field");
    assert!(field_info.is_none());
}

// ============================================================================
// Test Group 7: Error Cases
// ============================================================================

#[test]
fn test_field_access_missing_account_parameter() {
    let mut account_system = AccountSystem::new();
    let mut emitter = TestEmitter::new();

    let fields = vec![StructField {
        name: "amount".to_string(),
        field_type: TypeNode::Primitive("u64".to_string()),
        is_mutable: true,
        is_optional: false,
    }];
    account_system
        .process_account_definition("Transfer", &fields)
        .unwrap();

    // Empty symbol table - parameter not found
    let symbol_table = HashMap::new();

    let result = account_system.generate_account_field_access(
        &mut emitter,
        "missing_account",
        "Transfer",
        "amount",
        &symbol_table,
    );

    assert!(
        result.is_err(),
        "Should error when parameter not in symbol table"
    );
}

#[test]
fn test_field_access_unregistered_account_type() {
    let mut account_system = AccountSystem::new();
    let mut emitter = TestEmitter::new();

    let mut symbol_table = HashMap::new();
    symbol_table.insert(
        "account".to_string(),
        FieldInfo {
            offset: 0,
            field_type: "UnregisteredType".to_string(),
            is_mutable: true,
            is_optional: false,
            is_parameter: true,
        },
    );

    let result = account_system.generate_account_field_access(
        &mut emitter,
        "account",
        "UnregisteredType",
        "some_field",
        &symbol_table,
    );

    assert!(
        result.is_err(),
        "Should error when account type not registered"
    );
}

// ============================================================================
// Test Group 8: Account Registry Access
// ============================================================================

#[test]
fn test_get_account_registry() {
    let mut account_system = AccountSystem::new();

    let fields = vec![StructField {
        name: "balance".to_string(),
        field_type: TypeNode::Primitive("u64".to_string()),
        is_mutable: true,
        is_optional: false,
    }];
    account_system
        .process_account_definition("Vault", &fields)
        .unwrap();

    let registry = account_system.get_account_registry();
    assert!(registry.account_types.contains_key("Vault"));
}

#[test]
fn test_get_account_registry_mut() {
    let mut account_system = AccountSystem::new();

    let fields = vec![StructField {
        name: "balance".to_string(),
        field_type: TypeNode::Primitive("u64".to_string()),
        is_mutable: true,
        is_optional: false,
    }];
    account_system
        .process_account_definition("Vault", &fields)
        .unwrap();

    let registry = account_system.get_account_registry_mut();
    assert!(registry.account_types.contains_key("Vault"));
}
