//! Import discovery for ecosystem composability
//!
//! This module enables Five DSL programs to discover and import functions
//! from deployed contracts using `use ContractAddress::*` syntax.
//! It leverages embedded function name metadata in bytecode.

use crate::bytecode_parser::BytecodeParseError;
use std::collections::HashMap;

/// Discovered function interface from deployed bytecode
#[derive(Debug, Clone)]
pub struct DiscoveredFunction {
    pub name: String,
    pub address: u16,
    pub param_count: u8,
    pub account_index: Option<u8>, // For external contract calls
}

/// Import discovery result containing all available functions
#[derive(Debug, Clone)]
pub struct DiscoveredInterface {
    pub contract_address: String,
    pub functions: HashMap<String, DiscoveredFunction>,
    pub total_bytecode_size: usize,
}

/// Import discovery engine for ecosystem composability
pub struct ImportDiscovery;

impl ImportDiscovery {
    /// Discover available functions from deployed contract bytecode
    /// This enables `use ContractAddress::*` import syntax
    pub fn discover_functions_from_bytecode(
        contract_address: &str,
        bytecode: &[u8],
    ) -> Result<DiscoveredInterface, BytecodeParseError> {
        use five_protocol::parser::parse_optimized_bytecode;

        // Map any parse error from the optimized bytecode parser into a
        // BytecodeParseError variant recognizable by the discovery flow.
        // The parser returns a String on error; treat any parse failure here as
        // an InvalidUtf8FunctionName condition for tooling integration purposes.
        let parsed = parse_optimized_bytecode(bytecode)
            .map_err(|_e| BytecodeParseError::InvalidUtf8FunctionName)?;

        let mut functions = HashMap::new();

        if let Some(metadata) = parsed.function_names {
            for name_entry in metadata.names {
                functions.insert(
                    name_entry.name.clone(),
                    DiscoveredFunction {
                        name: name_entry.name,
                        address: name_entry.function_index as u16, // Use index as address for now
                        param_count: 0,      // Metadata doesn't include param count
                        account_index: None, // Will be resolved during compilation
                    },
                );
            }
        }

        Ok(DiscoveredInterface {
            contract_address: contract_address.to_string(),
            functions,
            total_bytecode_size: bytecode.len(),
        })
    }

    /// Resolve import at compile time by finding function index from name
    pub fn resolve_import_at_compile_time(
        imported_name: &str,
        available_functions: &[five_protocol::FunctionNameEntry],
    ) -> Result<u8, String> {
        for entry in available_functions {
            if entry.name == imported_name {
                return Ok(entry.function_index);
            }
        }
        Err(format!(
            "Function '{}' not found in available functions",
            imported_name
        ))
    }

    /// Generate Five DSL interface declaration from discovered functions
    /// This can be used for IDE autocomplete and type checking
    pub fn generate_interface_declaration(discovered: &DiscoveredInterface) -> String {
        let mut declaration = format!(
            "// Auto-generated interface for {}\n",
            discovered.contract_address
        );
        declaration.push_str("interface ExternalContract {\n");

        for (name, func) in &discovered.functions {
            // Generate function signature (simplified - real implementation would need type analysis)
            declaration.push_str(&format!(
                "    fn {}({}) -> Value; // {} parameters\n",
                name,
                (0..func.param_count)
                    .map(|i| format!("param{}: Value", i))
                    .collect::<Vec<_>>()
                    .join(", "),
                func.param_count
            ));
        }

        declaration.push_str("}\n");
        declaration
    }

    /// Check if a function is available for import
    pub fn is_function_available(discovered: &DiscoveredInterface, function_name: &str) -> bool {
        discovered.functions.contains_key(function_name)
    }

    /// Get function information for compiler integration
    pub fn get_function_info<'a>(
        discovered: &'a DiscoveredInterface,
        function_name: &str,
    ) -> Option<&'a DiscoveredFunction> {
        discovered.functions.get(function_name)
    }

    /// Create import mapping for compiler's function dispatcher
    /// Maps function names to external call information
    pub fn create_import_mapping(
        discovered: &DiscoveredInterface,
        account_index: u8, // Index of this contract in the transaction
    ) -> HashMap<String, (String, u8, u16)> {
        // (contract_address, account_index, func_offset)
        let mut mapping = HashMap::new();

        for (name, func) in &discovered.functions {
            mapping.insert(
                name.clone(),
                (
                    discovered.contract_address.clone(),
                    account_index,
                    func.address,
                ),
            );
        }

        mapping
    }
}

/// Mock implementation for testing - in practice this would fetch from blockchain
pub struct MockContractStorage {
    contracts: HashMap<String, Vec<u8>>,
}

impl Default for MockContractStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MockContractStorage {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
        }
    }

    pub fn add_contract(&mut self, address: &str, bytecode: Vec<u8>) {
        self.contracts.insert(address.to_string(), bytecode);
    }

    pub fn get_bytecode(&self, address: &str) -> Option<&[u8]> {
        self.contracts.get(address).map(|v| v.as_slice())
    }
}

/// Integration helper for compiler to resolve imports
pub fn resolve_import_statement(
    contract_address: &str,
    storage: &MockContractStorage,
) -> Result<DiscoveredInterface, String> {
    let bytecode = storage
        .get_bytecode(contract_address)
        .ok_or_else(|| format!("Contract not found: {}", contract_address))?;

    ImportDiscovery::discover_functions_from_bytecode(contract_address, bytecode)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use five_protocol::opcodes;

    #[test]
    fn test_discover_functions() {
        // Create bytecode with optimized header and function name metadata
        use five_protocol::{BytecodeBuilder, VLE};

        let mut builder = BytecodeBuilder::new();
        builder.emit_header(2, 2); // 2 public functions

        // Set FEATURE_FUNCTION_NAMES flag
        builder.patch_u32(4, 0x0100).expect("patch features");

        // Emit function name metadata
        let section_size = 1 + (1 + 7) + (1 + 8); // name_count + (len + "swap_AB") + (len + "get_rate")
        let (size_bytes, bytes) = VLE::encode_u16(section_size as u16);
        builder.emit_bytes(&bytes[..size_bytes]);
        builder.emit_u8(2); // 2 function names

        // swap_AB
        builder.emit_u8(7); // length
        builder.emit_bytes(b"swap_AB");

        // get_rate
        builder.emit_u8(8); // length
        builder.emit_bytes(b"get_rate");

        // Add actual instructions
        builder.emit_u8(opcodes::HALT);

        let bytecode = builder.build();

        let result =
            ImportDiscovery::discover_functions_from_bytecode("test_contract", &bytecode).unwrap();

        assert_eq!(result.functions.len(), 2);
        assert!(result.functions.contains_key("swap_AB"));
        assert!(result.functions.contains_key("get_rate"));
    }

    #[test]
    fn test_generate_interface_declaration() {
        let mut functions = HashMap::new();
        functions.insert(
            "test_func".to_string(),
            DiscoveredFunction {
                name: "test_func".to_string(),
                address: 0x100,
                param_count: 2,
                account_index: None,
            },
        );

        let discovered = DiscoveredInterface {
            contract_address: "test_contract".to_string(),
            functions,
            total_bytecode_size: 100,
        };

        let declaration = ImportDiscovery::generate_interface_declaration(&discovered);
        assert!(declaration.contains("interface ExternalContract"));
        assert!(declaration.contains("fn test_func(param0: Value, param1: Value)"));
    }
}
