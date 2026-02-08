//! Program generation logic.

use super::types::ASTGenerator;
use super::super::OpcodeEmitter;
use crate::ast::AstNode;
use five_vm_mito::error::VMError;

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
        // Pre-process imports to populate external_imports for CALL_EXTERNAL generation
        // TEMP FIX: Hardcode offsets for math_lib test case since we don't have a linker yet
        #[cfg(debug_assertions)]
        println!("AST Generator: Processing {} import statements", import_statements.len());
        for import_stmt in import_statements {
            #[cfg(debug_assertions)]
            println!("AST Generator: Inspecting import: {:?}", import_stmt);
            let module_name = match &import_stmt {
                AstNode::ImportStatement { module_specifier, .. } => match module_specifier {
                    crate::ast::ModuleSpecifier::Local(name) => Some(name.clone()),
                    crate::ast::ModuleSpecifier::Nested(path) => path.last().cloned(),
                    crate::ast::ModuleSpecifier::External(path) => Some(path.clone()), // Or parse path if needed
                },
                _ => None,
            };

            if let Some(_name) = module_name {
                // Future: Implement generic external import registration here
                // This would involve looking up the module in a registry or checking for interface definitions
                // to populate self.external_imports
            }
        }

        // Process field definitions first, populating the global symbol table
        for field_def in field_definitions {
            self.process_field_definition(emitter, field_def, true)?;
        }

        // Process interface definitions (populate interface registry)
        self.process_interface_definitions(interface_definitions)?;

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
