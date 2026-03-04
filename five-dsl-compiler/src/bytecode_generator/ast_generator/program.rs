//! Program generation logic.

use super::super::OpcodeEmitter;
use super::types::ASTGenerator;
use crate::ast::AstNode;
use crate::ast::ImportItem;
use five_vm_mito::error::VMError;
use std::collections::HashMap;

fn is_valid_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) if is_valid_identifier_start(first) => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl ASTGenerator {
    pub(super) fn generate_program<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        import_statements: &[AstNode],
        field_definitions: &[AstNode],
        interface_definitions: &[AstNode],
        init_block: &Option<Box<AstNode>>,
        constraints_block: &Option<Box<AstNode>>,
        instruction_definitions: &[AstNode],
    ) -> Result<(), VMError> {
        // Pre-process external imports so qualified calls can emit CALL_EXTERNAL.
        // Contract: each external import is bound to account index by import order.
        // If imported_items is present, only those functions are callable.
        let mut external_import_index: u8 = 0;
        for import_stmt in import_statements {
            let AstNode::ImportStatement {
                module_specifier,
                imported_items,
                ..
            } = import_stmt
            else {
                continue;
            };

            let address = match module_specifier {
                crate::ast::ModuleSpecifier::External(address) => address.clone(),
                crate::ast::ModuleSpecifier::Namespace(ns) => ns.import_key().to_string(),
                _ => continue,
            };

            let mut function_selectors = HashMap::new();
            let mut allow_any_function = imported_items.is_none();
            if let Some(items) = imported_items {
                for item in items {
                    match item {
                        ImportItem::Interface(interface_name) => {
                            // Imported interfaces are external-execution namespaces.
                            self.register_external_import(
                                interface_name.clone(),
                                external_import_index,
                                true,
                                HashMap::new(),
                            );
                        }
                        ImportItem::Method(fn_name) | ImportItem::Unqualified(fn_name) => {
                            function_selectors
                                .insert(fn_name.clone(), Self::external_selector(fn_name));
                        }
                    }
                }
                if function_selectors.is_empty() {
                    allow_any_function = true;
                }
            }

            // Preferred key: full external string if it can be used as an identifier.
            // Fallback key: deterministic synthetic alias.
            let mut keys = Vec::new();
            if is_valid_identifier(&address) {
                keys.push(address.clone());
            }
            keys.push(format!("ext{}", external_import_index));

            for key in keys {
                self.register_external_import(
                    key,
                    external_import_index,
                    allow_any_function,
                    function_selectors.clone(),
                );
            }

            external_import_index = external_import_index.saturating_add(1);
        }

        // Process field definitions first, populating the global symbol table
        for field_def in field_definitions {
            self.process_field_definition(emitter, field_def, true)?;
        }

        // Process interface definitions (populate interface registry)
        self.process_interface_definitions(interface_definitions)?;

        // Cache user-defined function parameter types so call lowering can use
        // expected argument types at call sites.
        self.cache_function_parameter_types(instruction_definitions);

        // Process init block if present
        if let Some(init) = init_block {
            self.generate_ast_node(emitter, init)?;
        }

        // Process constraints block if present
        if let Some(constraints) = constraints_block {
            self.generate_ast_node(emitter, constraints)?;
        }

        // Process instruction definitions with visibility-based ordering
        // Phase 2: Separate public and private functions for proper visibility enforcement
        let mut public_functions = Vec::new();
        let mut private_functions = Vec::new();

        #[cfg(debug_assertions)]
        println!(
            "AST_GENERATOR_DEBUG: Starting function separation, total functions: {}",
            instruction_definitions.len()
        );

        // Separate functions by visibility
        for (i, instruction_def) in instruction_definitions.iter().enumerate() {
            if let AstNode::InstructionDefinition {
                name, visibility, ..
            } = instruction_def
            {
                let is_public = visibility.is_on_chain_callable();
                #[cfg(debug_assertions)]
                println!(
                    "AST_GENERATOR_DEBUG: Function[{}] '{}' is_public: {}",
                    i, name, is_public
                );
                if is_public {
                    public_functions.push(instruction_def);
                } else {
                    private_functions.push(instruction_def);
                }
            }
        }

        #[cfg(debug_assertions)]
        println!(
            "AST_GENERATOR_DEBUG: Separated {} public functions, {} private functions",
            public_functions.len(),
            private_functions.len()
        );

        // Process public functions first (get indices 0, 1, 2... - externally callable)
        for (i, public_function) in public_functions.iter().enumerate() {
            if let AstNode::InstructionDefinition { name, .. } = public_function {
                #[cfg(debug_assertions)]
                println!(
                    "AST_GENERATOR_DEBUG: Processing public function[{}] '{}'",
                    i, name
                );
            }
            self.generate_ast_node(emitter, public_function)?;
        }

        // Process private functions after (get higher indices - internal only)
        for (i, private_function) in private_functions.iter().enumerate() {
            if let AstNode::InstructionDefinition { name, .. } = private_function {
                #[cfg(debug_assertions)]
                println!(
                    "AST_GENERATOR_DEBUG: Processing private function[{}] '{}'",
                    i, name
                );
            }
            self.generate_ast_node(emitter, private_function)?;
        }

        Ok(())
    }
}
