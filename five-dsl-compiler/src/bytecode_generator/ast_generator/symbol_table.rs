//! Symbol table operations.

use super::super::types::FieldInfo;
use super::types::ASTGenerator;
use std::collections::HashMap;

impl ASTGenerator {
    /// Get current symbol table
    pub fn get_symbol_table(&self) -> &HashMap<String, FieldInfo> {
        &self.local_symbol_table
    }

    /// Get field counter value
    pub fn get_field_counter(&self) -> u32 {
        self.field_counter
    }

    /// Get a clone of the current symbol table
    pub fn clone_symbol_table(&self) -> HashMap<String, FieldInfo> {
        self.local_symbol_table.clone()
    }

    /// Set the symbol table (for state transfer between generators)
    pub fn set_symbol_table(&mut self, symbol_table: HashMap<String, FieldInfo>) {
        self.local_symbol_table = symbol_table;
    }

    /// Add a function parameter to the symbol table (for function parameter access)
    pub fn add_parameter_to_symbol_table(&mut self, name: String, field_info: FieldInfo) {
        self.local_symbol_table.insert(name, field_info);
    }
}
