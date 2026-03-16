// Account-related operations for the DSL compiler.

use super::types::*;
use super::OpcodeEmitter;
use crate::ast::{AstNode, StructField, TypeNode};
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;
use std::collections::HashMap;

const DEFAULT_ACCOUNT_VEC_CAPACITY: u32 = 16;

/// Account System for managing account definitions and operations
#[derive(Clone)]
pub struct AccountSystem {
    /// Account type registry for storing account definitions
    account_registry: AccountRegistry,

    /// Enable zerocopy optimization for performance
    zerocopy_enabled: bool,

    /// Built-in account properties
    builtin_properties: HashMap<String, u8>,
}

impl AccountSystem {
    fn register_builtin_session_type(registry: &mut AccountRegistry) {
        if registry.account_types.contains_key("Session") {
            return;
        }

        let mut fields = HashMap::new();
        fields.insert(
            "authority".to_string(),
            FieldInfo {
                offset: 0,
                field_type: "pubkey".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "delegate".to_string(),
            FieldInfo {
                offset: 32,
                field_type: "pubkey".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "target_program".to_string(),
            FieldInfo {
                offset: 64,
                field_type: "pubkey".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "expires_at_slot".to_string(),
            FieldInfo {
                offset: 96,
                field_type: "u64".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "scope_hash".to_string(),
            FieldInfo {
                offset: 104,
                field_type: "u64".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "nonce".to_string(),
            FieldInfo {
                offset: 112,
                field_type: "u64".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "bind_account".to_string(),
            FieldInfo {
                offset: 120,
                field_type: "pubkey".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "status".to_string(),
            FieldInfo {
                offset: 152,
                field_type: "u8".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "version".to_string(),
            FieldInfo {
                offset: 153,
                field_type: "u8".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );

        registry.account_types.insert(
            "Session".to_string(),
            AccountTypeInfo {
                name: "Session".to_string(),
                fields,
                total_size: 154,
                serializer: None,
            },
        );
    }

    fn vec_capacity_from_args(&self, args: &[TypeNode]) -> Result<u32, VMError> {
        match args {
            [_elem] => Ok(DEFAULT_ACCOUNT_VEC_CAPACITY),
            [_elem, TypeNode::Sized { base_type, size }] if base_type == "__const" => {
                if *size == 0 {
                    return Err(VMError::TypeMismatch);
                }
                u32::try_from(*size).map_err(|_| VMError::TypeMismatch)
            }
            _ => Err(VMError::TypeMismatch),
        }
    }

    /// Create a new account system
    pub fn new() -> Self {
        Self::with_registry(AccountRegistry::new())
    }

    /// Create a new account system with existing registry
    pub fn with_registry(registry: AccountRegistry) -> Self {
        let mut registry = registry;
        Self::register_builtin_session_type(&mut registry);

        let mut builtin_properties = HashMap::new();
        builtin_properties.insert("lamports".to_string(), FIELD_LAMPORTS);
        builtin_properties.insert("owner".to_string(), FIELD_OWNER);
        builtin_properties.insert("key".to_string(), FIELD_KEY);
        builtin_properties.insert("data".to_string(), FIELD_DATA);

        Self {
            account_registry: registry,
            zerocopy_enabled: true, // Enable by default for performance
            builtin_properties,
        }
    }

    /// Configure zerocopy optimization
    pub fn set_zerocopy_enabled(&mut self, enabled: bool) {
        self.zerocopy_enabled = enabled;
    }

    /// Process all account definitions in the AST
    pub fn process_account_definitions(&mut self, ast: &AstNode) -> Result<(), VMError> {
        if let AstNode::Program {
            account_definitions,
            ..
        } = ast
        {
            for account_def in account_definitions {
                self.process_account_definition_node(account_def)?;
            }
        }
        Ok(())
    }

    /// Process a single account definition node
    fn process_account_definition_node(&mut self, account_def: &AstNode) -> Result<(), VMError> {
        match account_def {
            AstNode::AccountDefinition {
                name,
                fields,
                serializer,
                visibility: _,
            } => {
                self.process_account_definition(name, fields, *serializer)?;
            }
            _ => {} // Skip non-account definitions
        }
        Ok(())
    }

    /// Process individual account definition and add to registry
    pub fn process_account_definition(
        &mut self,
        name: &str,
        fields: &[StructField],
        serializer: Option<crate::ast::AccountSerializer>,
    ) -> Result<(), VMError> {
        println!("AccountSystem: Processing account definition '{}'", name);
        let mut account_fields = HashMap::new();
        let mut total_size = 0u32;

        // Process each field in the account definition
        for field in fields {
            let field_type = self.type_node_to_string(&field.field_type);
            let field_size = self.calculate_type_size(&field.field_type)?;
            let is_option_type = matches!(
                &field.field_type,
                TypeNode::Generic { base, args } if base == "Option" && args.len() == 1
            );

            let field_info = FieldInfo {
                offset: total_size,
                field_type: field_type.clone(),
                is_mutable: field.is_mutable,
                // Treat Option<T> account fields as optional even without `?` marker.
                // This aligns account metadata with type-level optional semantics.
                is_optional: field.is_optional || is_option_type,
                is_parameter: false,
            };

            println!(
                "AccountSystem: Adding field '{}' type '{}' at offset {} (size: {})",
                field.name, field_type, total_size, field_size
            );
            account_fields.insert(field.name.clone(), field_info);
            total_size += field_size;
        }

        let field_count = account_fields.len();

        // Validate field count limit (64 fields max for account definitions)
        const MAX_ACCOUNT_FIELDS: usize = 64;
        if field_count > MAX_ACCOUNT_FIELDS {
            eprintln!(
                "ERROR: Account definition '{}' exceeds 64-field limit: {} fields defined",
                name, field_count
            );
            return Err(VMError::InvalidScript);
        }

        // Create account type info
        let account_type_info = AccountTypeInfo {
            name: name.to_string(),
            fields: account_fields,
            total_size,
            serializer: serializer.map(|mode| match mode {
                crate::ast::AccountSerializer::Raw => AccountDecodingSerializer::Raw,
                crate::ast::AccountSerializer::Borsh => AccountDecodingSerializer::Borsh,
                crate::ast::AccountSerializer::Bincode => AccountDecodingSerializer::Bincode,
                crate::ast::AccountSerializer::Anchor => AccountDecodingSerializer::Anchor,
            }),
        };

        // Add to registry
        self.account_registry
            .account_types
            .insert(name.to_string(), account_type_info);
        println!(
            "AccountSystem: Registered account type '{}' with {} fields (total size: {})",
            name, field_count, total_size
        );

        Ok(())
    }

    /// Generate bytecode for account field access
    pub fn generate_account_field_access<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        account_param: &str,
        account_type: &str,
        field_name: &str,
        symbol_table: &HashMap<String, FieldInfo>,
    ) -> Result<(), VMError> {
        // Check if it's a built-in property first
        if self.is_builtin_account_property(field_name) {
            return self.generate_builtin_account_property_access(
                emitter,
                account_param,
                field_name,
                symbol_table,
            );
        }

        if let Some(account_info) = self.resolve_account_type(account_type) {
            if let Some(field_info) = account_info.fields.get(field_name) {
                let account_index = self.resolve_account_index(symbol_table, account_param)?;

                if self.should_use_zerocopy_optimization(account_param) {
                    // Use zerocopy optimization for better performance
                    // Fall back to standard access until zerocopy is implemented.
                    emitter.emit_opcode(LOAD_FIELD);
                    emitter.emit_u8(account_index);
                    emitter.emit_u32(field_info.offset);
                    if field_info.is_optional {
                        emitter.emit_opcode(OPTIONAL_UNWRAP);
                    }
                } else {
                    // Standard account field access
                    emitter.emit_opcode(LOAD_FIELD);
                    emitter.emit_u8(account_index);
                    emitter.emit_u32(field_info.offset);
                    if field_info.is_optional {
                        emitter.emit_opcode(OPTIONAL_UNWRAP);
                    }
                }

                return Ok(());
            }
        }

        Err(VMError::InvalidScript) // Field not found
    }

    /// Generate bytecode for account field assignment
    pub fn generate_account_field_assignment<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        account_param: &str,
        account_type: &str,
        field_name: &str,
        value: &AstNode,
        symbol_table: &HashMap<String, FieldInfo>,
    ) -> Result<(), VMError> {
        // Check if it's a built-in property
        if self.is_builtin_account_property(field_name) {
            return self.generate_builtin_account_property_assignment(
                emitter,
                account_param,
                field_name,
                value,
                symbol_table,
            );
        }

        if let Some(account_info) = self.resolve_account_type(account_type) {
            if let Some(field_info) = account_info.fields.get(field_name) {
                // Check mutability
                if !field_info.is_mutable {
                    return Err(VMError::InvalidScript); // Cannot assign to immutable field
                }

                let account_index = self.resolve_account_index(symbol_table, account_param)?;

                // Generate value expression first (this would be handled by AST generator)
                // Assume the value is already on the stack.

                if self.should_use_zerocopy_optimization(account_param) {
                    // Use zerocopy optimization for better performance
                    // Fall back to standard access until zerocopy is implemented.
                    emitter.emit_opcode(STORE_FIELD);
                    emitter.emit_u8(account_index);
                    emitter.emit_u32(field_info.offset);
                } else {
                    // Standard account field storage
                    emitter.emit_opcode(STORE_FIELD);
                    emitter.emit_u8(account_index);
                    emitter.emit_u32(field_info.offset);
                }

                if field_info.is_optional {
                    emitter.emit_opcode(OPTIONAL_SOME);
                }

                return Ok(());
            }
        }

        Err(VMError::InvalidScript)
    }

    /// Helper to resolve account type
    fn resolve_account_type(&self, account_type: &str) -> Option<&AccountTypeInfo> {
        let namespace_suffix = format!("::{}", account_type);
        self.account_registry
            .account_types
            .get(account_type)
            .or_else(|| {
                self.account_registry
                    .account_types
                    .iter()
                    .find(|(k, _)| k.ends_with(&namespace_suffix))
                    .map(|(_, v)| v)
            })
    }

    /// Helper to resolve account index
    fn resolve_account_index(
        &self,
        symbol_table: &HashMap<String, FieldInfo>,
        account_param: &str,
    ) -> Result<u8, VMError> {
        if let Some(param_info) = symbol_table.get(account_param) {
            Ok(super::account_utils::account_index_from_param_offset(
                param_info.offset,
            ))
        } else {
            Err(VMError::InvalidScript) // Parameter not found
        }
    }

    /// Generate built-in account property access
    pub fn generate_builtin_account_property_access<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        account_param: &str,
        property: &str,
        symbol_table: &HashMap<String, FieldInfo>,
    ) -> Result<(), VMError> {
        let account_index = self.resolve_account_index(symbol_table, account_param)?;

        if let Some(_field_id) = self.builtin_properties.get(property) {
            // Use existing account property opcodes
            match property {
                "lamports" => emitter.emit_opcode(GET_LAMPORTS),
                "key" => emitter.emit_opcode(GET_KEY),
                "owner" => emitter.emit_opcode(GET_OWNER),
                "data" => emitter.emit_opcode(GET_DATA),
                _ => return Err(VMError::InvalidInstruction),
            }
            emitter.emit_u8(account_index);
        } else {
            return Err(VMError::InvalidScript);
        }

        Ok(())
    }

    /// Generate built-in account property assignment
    fn generate_builtin_account_property_assignment<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        account_param: &str,
        property: &str,
        _value: &AstNode,
        symbol_table: &HashMap<String, FieldInfo>,
    ) -> Result<(), VMError> {
        let account_index = self.resolve_account_index(symbol_table, account_param)?;

        // Check if the property is writable
        match property {
            "lamports" => {
                // Generate value expression (handled externally)
                emitter.emit_opcode(SET_LAMPORTS);
                emitter.emit_u8(account_index);
            }
            "data" => {
                // Data can be modified if account is mutable
                // Data modification uses generic store field with account index
                emitter.emit_opcode(STORE_FIELD);
                emitter.emit_u8(account_index); // Use resolved account index
                emitter.emit_u32(0); // Data field offset
            }
            "owner" | "key" => {
                // These are typically read-only
                return Err(VMError::InvalidScript);
            }
            _ => return Err(VMError::InvalidScript),
        }

        Ok(())
    }

    /// Check if a type is an account type
    pub fn is_account_type(&self, type_str: &str) -> bool {
        // Check built-in account types
        if matches!(type_str, "Account" | "TokenAccount" | "ProgramAccount") {
            return true;
        }

        // Check if it's a registered custom account type
        self.account_registry.account_types.contains_key(type_str)
    }

    /// Check if a field name is a built-in account property
    pub fn is_builtin_account_property(&self, field_name: &str) -> bool {
        self.builtin_properties.contains_key(field_name)
    }

    /// Determine if zerocopy optimization should be used
    fn should_use_zerocopy_optimization(&self, _account_param: &str) -> bool {
        self.zerocopy_enabled // Can be made more sophisticated based on account analysis
    }

    /// Calculate the size of a type in bytes
    fn calculate_type_size(&self, type_node: &TypeNode) -> Result<u32, VMError> {
        println!("DEBUG: calculate_type_size for {:?}", type_node);
        match type_node {
            TypeNode::Primitive(name) => {
                match name.as_str() {
                    "u8" => Ok(1),
                    "u16" => Ok(2),
                    "u32" => Ok(4),
                    "u64" | "i64" => Ok(8),
                    "u128" | "i128" => Ok(16),
                    "bool" => Ok(1),
                    "pubkey" => Ok(32),
                    // "string" fallback removed - require TypeNode::Sized
                    _ => Err(VMError::TypeMismatch),
                }
            }
            TypeNode::Sized { base_type, size } => match base_type.as_str() {
                "string" => Ok(*size as u32),
                _ => self.calculate_type_size(&TypeNode::Primitive(base_type.clone())),
            },
            TypeNode::Array { element_type, size } => {
                let element_size = self.calculate_type_size(element_type)?;
                let array_size = size.ok_or_else(|| {
                    println!("ERROR: Array type missing size specification");
                    VMError::TypeMismatch
                })?;
                Ok(element_size * (array_size as u32))
            }
            TypeNode::Named(name) => {
                if name == "Pubkey" {
                    Ok(32)
                } else {
                    Err(VMError::TypeMismatch)
                }
            }
            TypeNode::Generic { base, args } => {
                if base == "Option" && args.len() == 1 {
                    // Fixed-width account storage for Option<T>:
                    // reserve a 1-byte tag plus the inner payload width.
                    let inner_size = self.calculate_type_size(&args[0])?;
                    Ok(1 + inner_size)
                } else if base == "Vec" {
                    // Account vectors use fixed-capacity layout for deterministic offsets:
                    // [u32 length][capacity * element_size].
                    if args.is_empty() {
                        return Err(VMError::TypeMismatch);
                    }
                    let element_size = self.calculate_type_size(&args[0])?;
                    let capacity = self.vec_capacity_from_args(args)?;
                    Ok(4 + element_size * capacity)
                } else {
                    Err(VMError::TypeMismatch)
                }
            }
            _ => Err(VMError::TypeMismatch),
        }
    }

    /// Convert TypeNode to string representation
    fn type_node_to_string(&self, type_node: &TypeNode) -> String {
        match type_node {
            TypeNode::Primitive(name) => name.clone(),
            TypeNode::Generic { base, args } => {
                let arg_strings: Vec<String> = args
                    .iter()
                    .map(|arg| self.type_node_to_string(arg))
                    .collect();
                format!("{}<{}>", base, arg_strings.join(", "))
            }
            TypeNode::Array { element_type, size } => {
                let element_string = self.type_node_to_string(element_type);
                match size {
                    Some(size) => format!("[{}; {}]", element_string, size),
                    None => format!("Vec<{}>", element_string),
                }
            }
            TypeNode::Sized { base_type, size } => {
                if base_type == "__const" {
                    size.to_string()
                } else {
                    format!("{}<{}>", base_type, size)
                }
            }
            TypeNode::Account => "Account".to_string(),
            TypeNode::Named(name) => name.clone(),
            _ => "unknown".to_string(),
        }
    }

    /// Get account registry reference
    pub fn get_account_registry(&self) -> &AccountRegistry {
        &self.account_registry
    }

    /// Get mutable account registry reference
    pub fn get_account_registry_mut(&mut self) -> &mut AccountRegistry {
        &mut self.account_registry
    }

    /// Validate account type exists
    pub fn validate_account_type(&self, type_name: &str) -> bool {
        self.is_account_type(type_name)
    }

    /// Get field information for an account type
    pub fn get_field_info(&self, account_type: &str, field_name: &str) -> Option<&FieldInfo> {
        self.account_registry
            .account_types
            .get(account_type)?
            .fields
            .get(field_name)
    }

    /// Generate optimization report
    pub fn generate_optimization_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Account System Report\n");
        report.push_str("====================\n\n");

        report.push_str(&format!("Zerocopy enabled: {}\n", self.zerocopy_enabled));
        report.push_str(&format!(
            "Registered account types: {}\n",
            self.account_registry.account_types.len()
        ));
        report.push_str(&format!(
            "Built-in properties: {}\n",
            self.builtin_properties.len()
        ));

        for (type_name, account_info) in &self.account_registry.account_types {
            report.push_str(&format!("\nAccount Type: {}\n", type_name));
            report.push_str(&format!(
                "  Total size: {} bytes\n",
                account_info.total_size
            ));
            report.push_str(&format!("  Fields: {}\n", account_info.fields.len()));

            for (field_name, field_info) in &account_info.fields {
                report.push_str(&format!(
                    "    - {} ({}): offset {}, mutable: {}\n",
                    field_name, field_info.field_type, field_info.offset, field_info.is_mutable
                ));
            }
        }

        report
    }
}

impl Default for AccountSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension methods for the main DslBytecodeGenerator
impl super::DslBytecodeGenerator {
    /// Initialize account system and process account definitions
    pub fn init_account_system(&mut self, ast: &AstNode) -> Result<AccountSystem, VMError> {
        let mut account_system = AccountSystem::new();
        account_system.process_account_definitions(ast)?;
        Ok(account_system)
    }

    /// Generate account field access using the account system
    pub fn generate_account_field_access(
        &mut self,
        account_param: &str,
        account_type: &str,
        field_name: &str,
        symbol_table: &HashMap<String, FieldInfo>,
    ) -> Result<(), VMError> {
        let mut account_system = AccountSystem::with_registry(self.account_registry.clone());
        account_system.generate_account_field_access(
            self,
            account_param,
            account_type,
            field_name,
            symbol_table,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{StructField, TypeNode};

    #[test]
    fn test_account_system_creation() {
        let account_system = AccountSystem::new();
        assert!(
            account_system.account_registry.account_types.len() >= 1,
            "expected default account registry entries to be preloaded"
        );
        assert!(account_system.zerocopy_enabled);
        assert_eq!(account_system.builtin_properties.len(), 4);
    }

    #[test]
    fn test_builtin_property_detection() {
        let account_system = AccountSystem::new();

        assert!(account_system.is_builtin_account_property("lamports"));
        assert!(account_system.is_builtin_account_property("owner"));
        assert!(account_system.is_builtin_account_property("key"));
        assert!(account_system.is_builtin_account_property("data"));
        assert!(!account_system.is_builtin_account_property("custom_field"));
    }

    #[test]
    fn test_account_type_detection() {
        let account_system = AccountSystem::new();

        assert!(account_system.is_account_type("Account"));
        assert!(account_system.is_account_type("TokenAccount"));
        assert!(account_system.is_account_type("ProgramAccount"));
        assert!(!account_system.is_account_type("u64"));
        assert!(!account_system.is_account_type("string"));
    }

    #[test]
    fn test_type_size_calculation() {
        let account_system = AccountSystem::new();

        assert_eq!(
            account_system
                .calculate_type_size(&TypeNode::Primitive("u8".to_string()))
                .unwrap(),
            1
        );
        assert_eq!(
            account_system
                .calculate_type_size(&TypeNode::Primitive("u64".to_string()))
                .unwrap(),
            8
        );
        assert_eq!(
            account_system
                .calculate_type_size(&TypeNode::Primitive("pubkey".to_string()))
                .unwrap(),
            32
        );

        let sized_string = TypeNode::Sized {
            base_type: "string".to_string(),
            size: 64,
        };
        assert_eq!(
            account_system.calculate_type_size(&sized_string).unwrap(),
            64
        );

        let option_u64 = TypeNode::Generic {
            base: "Option".to_string(),
            args: vec![TypeNode::Primitive("u64".to_string())],
        };
        assert_eq!(account_system.calculate_type_size(&option_u64).unwrap(), 9);

        let vec_u64 = TypeNode::Generic {
            base: "Vec".to_string(),
            args: vec![TypeNode::Primitive("u64".to_string())],
        };
        assert_eq!(
            account_system.calculate_type_size(&vec_u64).unwrap(),
            4 + (8 * DEFAULT_ACCOUNT_VEC_CAPACITY)
        );

        let vec_u64_64 = TypeNode::Generic {
            base: "Vec".to_string(),
            args: vec![
                TypeNode::Primitive("u64".to_string()),
                TypeNode::Sized {
                    base_type: "__const".to_string(),
                    size: 64,
                },
            ],
        };
        assert_eq!(
            account_system.calculate_type_size(&vec_u64_64).unwrap(),
            4 + (8 * 64)
        );
    }

    #[test]
    fn test_account_definition_processing() {
        let mut account_system = AccountSystem::new();
        let initial_count = account_system.account_registry.account_types.len();

        let fields = vec![
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
            .process_account_definition("CustomAccount", &fields, None)
            .unwrap();

        assert!(account_system.validate_account_type("CustomAccount"));
        assert_eq!(
            account_system.account_registry.account_types.len(),
            initial_count + 1
        );

        let account_info = &account_system.account_registry.account_types["CustomAccount"];
        assert_eq!(account_info.fields.len(), 2);
        assert_eq!(account_info.total_size, 40); // 8 + 32 bytes
    }

    #[test]
    fn test_option_field_marked_optional_in_registry() {
        let mut account_system = AccountSystem::new();

        let fields = vec![StructField {
            name: "nickname".to_string(),
            field_type: TypeNode::Generic {
                base: "Option".to_string(),
                args: vec![TypeNode::Sized {
                    base_type: "string".to_string(),
                    size: 16,
                }],
            },
            is_mutable: true,
            is_optional: false,
        }];

        account_system
            .process_account_definition("Profile", &fields, None)
            .unwrap();

        let account_info = &account_system.account_registry.account_types["Profile"];
        let nickname = account_info.fields.get("nickname").unwrap();
        assert!(nickname.is_optional, "Option<T> fields should be optional");
        assert_eq!(account_info.total_size, 17, "tag + payload size");
    }
}
