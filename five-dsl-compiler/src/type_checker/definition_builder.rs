//! Definition builder for type-safe definition construction
//!
//! Provides helper methods for building type-safe Definition nodes during type checking.
//! These definitions automatically convert to AstNode via the From trait.

use crate::ast::{
    generated::*, AstNode, ErrorVariant, ImportItem, InstructionParameter, ModuleSpecifier,
    StructField, TestAttribute, TypeNode, Visibility,
};

/// Helper methods for building definitions
#[allow(dead_code)]
impl super::types::TypeCheckerContext {
    /// Create a field definition
    pub(crate) fn build_field_definition(
        name: String,
        field_type: TypeNode,
        is_mutable: bool,
        is_optional: bool,
        default_value: Option<AstNode>,
        visibility: Visibility,
    ) -> Definition {
        Definition::FieldDefinition(FieldDefinitionNode {
            name,
            field_type: Box::new(field_type),
            is_mutable,
            is_optional,
            default_value: default_value.map(Box::new),
            visibility,
        })
    }

    /// Create an instruction definition
    pub(crate) fn build_instruction_definition(
        name: String,
        parameters: Vec<InstructionParameter>,
        return_type: Option<TypeNode>,
        body: AstNode,
        visibility: Visibility,
    ) -> Definition {
        Definition::InstructionDefinition(InstructionDefinitionNode {
            name,
            parameters,
            return_type: return_type.map(Box::new),
            body: Box::new(body),
            visibility,
        })
    }

    /// Create an event definition
    pub(crate) fn build_event_definition(
        name: String,
        fields: Vec<StructField>,
        visibility: Visibility,
    ) -> Definition {
        Definition::EventDefinition(EventDefinitionNode {
            name,
            fields,
            visibility,
        })
    }

    /// Create an error type definition
    pub(crate) fn build_error_type_definition(
        name: String,
        variants: Vec<ErrorVariant>,
    ) -> Definition {
        Definition::ErrorTypeDefinition(ErrorTypeDefinitionNode { name, variants })
    }

    /// Create an account definition
    pub(crate) fn build_account_definition(
        name: String,
        fields: Vec<StructField>,
        visibility: Visibility,
    ) -> Definition {
        Definition::AccountDefinition(AccountDefinitionNode {
            name,
            fields,
            serializer: None,
            visibility,
        })
    }

    /// Create an interface definition
    pub(crate) fn build_interface_definition(
        name: String,
        program_id: Option<String>,
        functions: Vec<AstNode>,
    ) -> Definition {
        Definition::InterfaceDefinition(InterfaceDefinitionNode {
            name,
            program_id,
            serializer: None,
            is_anchor: false,
            functions,
        })
    }

    /// Create an interface function
    pub(crate) fn build_interface_function(
        name: String,
        parameters: Vec<InstructionParameter>,
        return_type: Option<TypeNode>,
        discriminator: Option<u8>,
    ) -> Definition {
        Definition::InterfaceFunction(InterfaceFunctionNode {
            name,
            parameters,
            return_type: return_type.map(Box::new),
            discriminator,
            discriminator_bytes: None,
            is_anchor: false,
        })
    }

    /// Create an import statement
    pub(crate) fn build_import_statement(
        module_specifier: ModuleSpecifier,
        imported_items: Option<Vec<ImportItem>>,
    ) -> Definition {
        Definition::ImportStatement(ImportStatementNode {
            module_specifier,
            imported_items,
            is_reexport: false,
        })
    }

    /// Create an arrow function
    pub(crate) fn build_arrow_function(
        parameters: Vec<InstructionParameter>,
        return_type: Option<TypeNode>,
        body: AstNode,
        is_async: bool,
    ) -> Definition {
        Definition::ArrowFunction(ArrowFunctionNode {
            parameters,
            return_type: return_type.map(Box::new),
            body: Box::new(body),
            is_async,
        })
    }

    /// Create a test function
    pub(crate) fn build_test_function(
        name: String,
        attributes: Vec<TestAttribute>,
        body: AstNode,
    ) -> Definition {
        Definition::TestFunction(TestFunctionNode {
            name,
            attributes,
            body: Box::new(body),
        })
    }

    /// Create a test module
    pub(crate) fn build_test_module(
        name: String,
        attributes: Vec<TestAttribute>,
        body: AstNode,
    ) -> Definition {
        Definition::TestModule(TestModuleNode {
            name,
            attributes,
            body: Box::new(body),
        })
    }
}
