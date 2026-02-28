// Merge module ASTs for multi-file compilation.

use crate::ast::AstNode;
use five_vm_mito::error::VMError;
use std::collections::HashMap;

/// Merges multiple module ASTs into a single unified AST
pub struct ModuleMerger {
    /// Main program AST (entry point)
    main_ast: Option<Box<AstNode>>,
    /// Additional module ASTs to merge
    module_asts: HashMap<String, Box<AstNode>>,
    /// Symbol name mapping for conflicts (old_name -> new_name)
    symbol_renames: HashMap<String, String>,
    /// Enable namespace qualification (module::function)
    enable_namespaces: bool,
}

impl ModuleMerger {
    /// Create a new module merger
    pub fn new() -> Self {
        Self {
            main_ast: None,
            module_asts: HashMap::new(),
            symbol_renames: HashMap::new(),
            enable_namespaces: true,
        }
    }

    /// Set the main/entry point AST
    pub fn set_main_ast(&mut self, ast: AstNode) {
        self.main_ast = Some(Box::new(ast));
    }

    /// Add a module AST
    pub fn add_module(&mut self, module_name: String, ast: AstNode) {
        self.module_asts.insert(module_name, Box::new(ast));
    }

    /// Enable or disable module namespace qualification
    pub fn with_namespaces(mut self, enable: bool) -> Self {
        self.enable_namespaces = enable;
        self
    }

    /// Merge all modules into a single AST
    pub fn merge(&mut self) -> Result<AstNode, VMError> {
        if self.main_ast.is_none() {
            return Err(VMError::InvalidScript);
        }

        let main_ast = self.main_ast.take().unwrap();

        // Extract components from main AST
        if let AstNode::Program {
            program_name,
            mut field_definitions,
            mut instruction_definitions,
            mut event_definitions,
            mut account_definitions,
            mut interface_definitions,
            import_statements,
            init_block,
            constraints_block,
        } = *main_ast
        {
            // Collect all module ASTs to avoid borrowing conflicts
            let modules: Vec<_> = self
                .module_asts
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            // Merge all module ASTs into the main AST
            for (module_name, module_ast) in modules.iter() {
                self.merge_module_into(
                    module_name,
                    module_ast.as_ref(),
                    &mut field_definitions,
                    &mut instruction_definitions,
                    &mut event_definitions,
                    &mut account_definitions,
                    &mut interface_definitions,
                )?;
            }

            // Create merged program AST
            Ok(AstNode::Program {
                program_name,
                field_definitions,
                instruction_definitions,
                event_definitions,
                account_definitions,
                interface_definitions,
                import_statements,
                init_block,
                constraints_block,
            })
        } else {
            Err(VMError::InvalidScript)
        }
    }

    /// Merge a single module's definitions into the main lists
    fn merge_module_into(
        &mut self,
        module_name: &str,
        module_ast: &AstNode,
        field_defs: &mut Vec<AstNode>,
        instr_defs: &mut Vec<AstNode>,
        event_defs: &mut Vec<AstNode>,
        account_defs: &mut Vec<AstNode>,
        interface_defs: &mut Vec<AstNode>,
    ) -> Result<(), VMError> {
        if let AstNode::Program {
            field_definitions,
            instruction_definitions,
            event_definitions,
            account_definitions,
            interface_definitions,
            ..
        } = module_ast
        {
            // Add all public and internal definitions from the module
            // (Private definitions are filtered out)
            for field_def in field_definitions {
                if self.is_importable_definition(field_def) {
                    let qualified = self.qualify_with_module(field_def, module_name)?;
                    field_defs.push(qualified);
                }
            }

            for instr_def in instruction_definitions {
                if self.is_importable_definition(instr_def) {
                    let qualified = self.qualify_with_module(instr_def, module_name)?;
                    instr_defs.push(qualified);
                }
            }

            for event_def in event_definitions {
                if self.is_importable_definition(event_def) {
                    let qualified = self.qualify_with_module(event_def, module_name)?;
                    event_defs.push(qualified);
                }
            }

            for account_def in account_definitions {
                if self.is_importable_definition(account_def) {
                    let qualified = self.qualify_with_module(account_def, module_name)?;
                    account_defs.push(qualified);
                }
            }

            for interface_def in interface_definitions {
                if self.is_importable_definition(interface_def) {
                    let qualified = self.qualify_with_module(interface_def, module_name)?;
                    interface_defs.push(qualified);
                }
            }

            Ok(())
        } else {
            Err(VMError::InvalidScript)
        }
    }

    /// Check if a definition is importable based on visibility
    fn is_importable_definition(&self, definition: &AstNode) -> bool {
        match definition {
            AstNode::FieldDefinition { visibility, .. } => visibility.is_importable(),
            AstNode::InstructionDefinition { visibility, .. } => visibility.is_importable(),
            AstNode::EventDefinition { visibility, .. } => visibility.is_importable(),
            AstNode::AccountDefinition { visibility, .. } => visibility.is_importable(),
            // InterfaceDefinitions are always included (no visibility field)
            AstNode::InterfaceDefinition { .. } => true,
            _ => false,
        }
    }

    /// Qualify a definition with its module name prefix
    #[allow(deprecated)]
    fn qualify_with_module(
        &self,
        definition: &AstNode,
        module_name: &str,
    ) -> Result<AstNode, VMError> {
        if !self.enable_namespaces {
            return Ok(definition.clone());
        }

        match definition {
            AstNode::InstructionDefinition {
                name,
                visibility,
                parameters,
                return_type,
                body,
                ..
            } => Ok(AstNode::InstructionDefinition {
                name: format!("{}::{}", module_name, name),
                visibility: *visibility,
                is_public: visibility.is_on_chain_callable(),
                parameters: parameters.clone(),
                return_type: return_type.clone(),
                body: body.clone(),
            }),
            AstNode::FieldDefinition {
                name,
                field_type,
                is_mutable,
                is_optional,
                default_value,
                visibility,
            } => Ok(AstNode::FieldDefinition {
                name: format!("{}::{}", module_name, name),
                field_type: field_type.clone(),
                is_mutable: *is_mutable,
                is_optional: *is_optional,
                default_value: default_value.clone(),
                visibility: *visibility,
            }),
            AstNode::EventDefinition {
                name,
                visibility,
                fields,
            } => Ok(AstNode::EventDefinition {
                name: format!("{}::{}", module_name, name),
                visibility: *visibility,
                fields: fields.clone(),
            }),
            AstNode::AccountDefinition {
                name,
                visibility,
                fields,
            } => Ok(AstNode::AccountDefinition {
                name: format!("{}::{}", module_name, name),
                visibility: *visibility,
                fields: fields.clone(),
            }),
            // InterfaceDefinitions don't get qualified (they're external references)
            _ => Ok(definition.clone()),
        }
    }

    /// Register a symbol rename to handle naming conflicts
    pub fn register_rename(&mut self, old_name: String, new_name: String) {
        self.symbol_renames.insert(old_name, new_name);
    }

    /// Get a renamed symbol name if applicable
    pub fn get_renamed_symbol(&self, original_name: &str) -> String {
        self.symbol_renames
            .get(original_name)
            .cloned()
            .unwrap_or_else(|| original_name.to_string())
    }
}

impl Default for ModuleMerger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BlockKind, Visibility};

    fn create_simple_program(name: &str, instructions: Vec<AstNode>) -> AstNode {
        AstNode::Program {
            program_name: name.to_string(),
            field_definitions: vec![],
            instruction_definitions: instructions,
            event_definitions: vec![],
            account_definitions: vec![],
            interface_definitions: vec![],
            import_statements: vec![],
            init_block: None,
            constraints_block: None,
        }
    }

    #[allow(deprecated)]
    fn create_instruction(name: &str, visibility: Visibility) -> AstNode {
        AstNode::InstructionDefinition {
            name: name.to_string(),
            visibility,
            is_public: visibility == Visibility::Public,
            parameters: vec![],
            return_type: None,
            body: Box::new(AstNode::Block {
                statements: vec![],
                kind: BlockKind::Regular,
            }),
        }
    }

    #[test]
    fn test_merger_creation() {
        let merger = ModuleMerger::new();
        assert!(merger.main_ast.is_none());
        assert!(merger.module_asts.is_empty());
    }

    #[test]
    fn test_set_main_ast() {
        let mut merger = ModuleMerger::new();
        let program = create_simple_program("main", vec![]);

        merger.set_main_ast(program);
        assert!(merger.main_ast.is_some());
    }

    #[test]
    fn test_add_module() {
        let mut merger = ModuleMerger::new();
        let program = create_simple_program("helper", vec![]);

        merger.add_module("helper".to_string(), program);
        assert_eq!(merger.module_asts.len(), 1);
        assert!(merger.module_asts.contains_key("helper"));
    }

    #[test]
    fn test_merge_empty_modules() {
        let mut merger = ModuleMerger::new();
        let main = create_simple_program("main", vec![]);

        merger.set_main_ast(main);
        let result = merger.merge();

        assert!(result.is_ok());
        if let AstNode::Program {
            instruction_definitions,
            ..
        } = result.unwrap()
        {
            assert_eq!(instruction_definitions.len(), 0);
        }
    }

    #[test]
    fn test_merge_with_public_functions() {
        let mut merger = ModuleMerger::new();
        let main = create_simple_program(
            "main",
            vec![create_instruction("main_fn", Visibility::Public)],
        );
        let helper = create_simple_program(
            "helper",
            vec![create_instruction("helper_fn", Visibility::Public)],
        );

        merger.set_main_ast(main);
        merger.add_module("helper".to_string(), helper);

        let result = merger.merge();
        assert!(result.is_ok());

        if let AstNode::Program {
            instruction_definitions,
            ..
        } = result.unwrap()
        {
            // Should have both main_fn and helper::helper_fn
            assert_eq!(instruction_definitions.len(), 2);

            let names: Vec<String> = instruction_definitions
                .iter()
                .filter_map(|node| {
                    if let AstNode::InstructionDefinition { name, .. } = node {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect();

            assert!(names.contains(&"main_fn".to_string()));
            assert!(names.contains(&"helper::helper_fn".to_string()));
        }
    }

    #[test]
    fn test_merge_internal_functions_included() {
        let mut merger = ModuleMerger::new();
        let main = create_simple_program(
            "main",
            vec![create_instruction("main_fn", Visibility::Public)],
        );
        let helper = create_simple_program(
            "helper",
            vec![create_instruction("internal_fn", Visibility::Internal)],
        );

        merger.set_main_ast(main);
        merger.add_module("helper".to_string(), helper);

        let result = merger.merge();
        assert!(result.is_ok());

        if let AstNode::Program {
            instruction_definitions,
            ..
        } = result.unwrap()
        {
            // Should have both main_fn and internal_fn (internal is importable)
            assert_eq!(instruction_definitions.len(), 2);
        }
    }

    #[test]
    fn test_merge_private_functions_excluded() {
        let mut merger = ModuleMerger::new();
        let main = create_simple_program(
            "main",
            vec![create_instruction("main_fn", Visibility::Public)],
        );
        let helper = create_simple_program(
            "helper",
            vec![
                create_instruction("public_fn", Visibility::Public),
                create_instruction("private_fn", Visibility::Private),
            ],
        );

        merger.set_main_ast(main);
        merger.add_module("helper".to_string(), helper);

        let result = merger.merge();
        assert!(result.is_ok());

        if let AstNode::Program {
            instruction_definitions,
            ..
        } = result.unwrap()
        {
            // Should have main_fn and helper::public_fn, but NOT helper::private_fn
            assert_eq!(instruction_definitions.len(), 2);

            let names: Vec<String> = instruction_definitions
                .iter()
                .filter_map(|node| {
                    if let AstNode::InstructionDefinition { name, .. } = node {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect();

            assert!(names.contains(&"main_fn".to_string()));
            assert!(names.contains(&"helper::public_fn".to_string()));
            assert!(!names.contains(&"helper::private_fn".to_string()));
        }
    }

    #[test]
    fn test_register_rename() {
        let mut merger = ModuleMerger::new();
        merger.register_rename("old_name".to_string(), "new_name".to_string());

        assert_eq!(merger.get_renamed_symbol("old_name"), "new_name");
        assert_eq!(merger.get_renamed_symbol("unchanged"), "unchanged");
    }

    #[test]
    fn test_merge_multiple_modules() {
        let mut merger = ModuleMerger::new();
        let main = create_simple_program(
            "main",
            vec![create_instruction("main_fn", Visibility::Public)],
        );
        let helper1 = create_simple_program(
            "helper1",
            vec![create_instruction("helper1_fn", Visibility::Public)],
        );
        let helper2 = create_simple_program(
            "helper2",
            vec![create_instruction("helper2_fn", Visibility::Public)],
        );

        merger.set_main_ast(main);
        merger.add_module("helper1".to_string(), helper1);
        merger.add_module("helper2".to_string(), helper2);

        let result = merger.merge();
        assert!(result.is_ok());

        if let AstNode::Program {
            instruction_definitions,
            ..
        } = result.unwrap()
        {
            assert_eq!(instruction_definitions.len(), 3);

            let names: Vec<String> = instruction_definitions
                .iter()
                .filter_map(|node| {
                    if let AstNode::InstructionDefinition { name, .. } = node {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect();

            assert!(names.contains(&"main_fn".to_string()));
            assert!(names.contains(&"helper1::helper1_fn".to_string()));
            assert!(names.contains(&"helper2::helper2_fn".to_string()));
        }
    }

    #[test]
    fn test_merge_without_main_ast_fails() {
        let mut merger = ModuleMerger::new();
        let helper = create_simple_program("helper", vec![]);
        merger.add_module("helper".to_string(), helper);

        let result = merger.merge();
        assert!(result.is_err());
    }
}
