//! AST code generator
//!
//! Generates Rust AST structures from node_metadata.toml.
//! Creates individual structs for each node and nested category enums.
//! Usage: cargo run --bin generate_ast [output_path]

use five_dsl_compiler::ast::NODE_REGISTRY;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn main() {
    let output_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "src/ast/generated.rs".to_string());

    println!("🔨 Generating new AST structures from node_metadata.toml...");
    println!("   Registry loaded with {} AST nodes", NODE_REGISTRY.nodes.len());

    match generate_ast(&output_path) {
        Ok(size) => {
            println!("✅ Generated new AST structures ({} bytes)", size);
            println!("   Output: {}", output_path);
        }
        Err(e) => {
            eprintln!("❌ Failed to generate AST: {}", e);
            std::process::exit(1);
        }
    }
}

/// Generate generated.rs with new AST structures
fn generate_ast(output_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let code = generate_ast_code(&NODE_REGISTRY)?;

    // Create parent directories if needed
    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(output_path, &code)?;
    Ok(code.len())
}

/// Generate Rust code for new AST structures
fn generate_ast_code(
    registry: &five_dsl_compiler::ast::registry::NodeRegistry,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut output = String::new();

    // Header
    output.push_str("// AUTO-GENERATED FILE: Do not edit manually\n");
    output.push_str("// Generated from node_metadata.toml by generate_ast tool\n");
    output.push_str("// Run: cargo run --bin generate_ast\n\n");

    output.push_str("use crate::ast::{AstNode, BlockKind, TypeNode, StructField, InstructionParameter, EventFieldAssignment, MatchArm, ErrorVariant, StructLiteralField, SwitchCase, TestAttribute, AssertionType, ModuleSpecifier, Visibility};\n");
    output.push_str("use five_protocol::Value;\n\n");

    // Group nodes by category
    let mut categories: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, node) in registry.nodes.iter() {
        categories
            .entry(node.category.clone())
            .or_default()
            .push(name.clone());
    }

    // Generate individual node structs
    output.push_str("// ============================================================================\n");
    output.push_str("// INDIVIDUAL NODE STRUCTS\n");
    output.push_str("// ============================================================================\n\n");

    let mut struct_names = Vec::new();

    for (name, node) in registry.nodes.iter() {
        if name == "Identifier" || name == "Literal" {
            continue; // Skip special cases
        }

        let struct_name = format!("{}Node", name);
        struct_names.push((node.category.clone(), struct_name.clone()));

        output.push_str(&format!("/// {}\n", node.doc));
        output.push_str("#[derive(Debug, Clone, PartialEq)]\n");
        output.push_str(&format!("pub struct {} {{\n", struct_name));

        // Generate fields
        for (field_name, field) in &node.fields {
            output.push_str(&format!(
                "    pub {}: {},\n",
                field_name, field.field_type
            ));
        }

        output.push_str("}\n\n");
    }

    // Generate category enums
    output.push_str("// ============================================================================\n");
    output.push_str("// CATEGORY ENUMS (Type-safe AST organization)\n");
    output.push_str("// ============================================================================\n\n");

    for (category, _nodes) in categories.iter() {
        if category == "program" {
            continue; // Skip program category
        }

        // let enum_name = format!("{}[0]", category); // Placeholder removed
        let enum_name = match category.as_str() {
            "expression" => "Expression",
            "statement" => "Statement",
            "definition" => "Definition",
            _ => category.as_str(),
        };

        output.push_str(&format!(
            "/// {} nodes grouped for type safety\n",
            category
        ));
        output.push_str("#[derive(Debug, Clone, PartialEq)]\n");
        output.push_str(&format!("pub enum {} {{\n", enum_name));

        // Filter nodes that have this category
        for node_name in registry.get_by_category(category) {
            if node_name.name != "Identifier" && node_name.name != "Literal" {
                let struct_name = format!("{}Node", node_name.name);
                output.push_str(&format!("    {}({}),\n", node_name.name, struct_name));
            }
        }

        // Add special cases for Identifier and Literal if they're expressions
        if category == "expression" {
            output.push_str("    Identifier(String),\n");
            output.push_str("    Literal(Value),\n");
        }

        output.push_str("}\n\n");
    }

    // Generate From/Into conversions from new types to old AstNode
    output.push_str("// ============================================================================\n");
    output.push_str("// BACKWARD COMPATIBILITY CONVERSIONS\n");
    output.push_str("// ============================================================================\n\n");

    output.push_str("impl From<Expression> for AstNode {\n");
    output.push_str("    fn from(expr: Expression) -> Self {\n");
    output.push_str("        match expr {\n");

    // Generate match arms
    for node in registry.get_by_category("expression") {
        if node.name == "Identifier" {
            output.push_str("            Expression::Identifier(name) => AstNode::Identifier(name),\n");
        } else if node.name == "Literal" {
            output.push_str("            Expression::Literal(value) => AstNode::Literal(value),\n");
        } else {
            output.push_str(&format!(
                "            Expression::{}(node) => AstNode::{}{{ ",
                node.name, node.name
            ));

            // Generate field assignments
            let mut first = true;
            for field_name in node.fields.keys() {
                if !first {
                    output.push_str(", ");
                }
                output.push_str(&format!("{}: node.{}", field_name, field_name));
                first = false;
            }

            output.push_str(" },\n");
        }
    }

    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate From/Into for Statement
    output.push_str("\n// ============================================================================\n");
    output.push_str("// STATEMENT CATEGORY CONVERSIONS\n");
    output.push_str("// ============================================================================\n\n");
    output.push_str("impl From<Statement> for AstNode {\n");
    output.push_str("    fn from(stmt: Statement) -> Self {\n");
    output.push_str("        match stmt {\n");

    for node in registry.get_by_category("statement") {
        output.push_str(&format!(
            "            Statement::{}(node) => AstNode::{}{{ ",
            node.name, node.name
        ));

        // Generate field assignments
        let mut first = true;
        for field_name in node.fields.keys() {
            if !first {
                output.push_str(", ");
            }
            output.push_str(&format!("{}: node.{}", field_name, field_name));
            first = false;
        }

        output.push_str(" },\n");
    }

    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate From/Into for Definition
    output.push_str("// ============================================================================\n");
    output.push_str("// DEFINITION CATEGORY CONVERSIONS\n");
    output.push_str("// ============================================================================\n\n");
    output.push_str("impl From<Definition> for AstNode {\n");
    output.push_str("    fn from(def: Definition) -> Self {\n");
    output.push_str("        match def {\n");

    for node in registry.get_by_category("definition") {
        output.push_str(&format!(
            "            Definition::{}(node) => AstNode::{}{{ ",
            node.name, node.name
        ));

        // Generate field assignments - handle special cases
        let mut first = true;
        for field_name in node.fields.keys() {
            if !first {
                output.push_str(", ");
            }

            // Special handling for InstructionDefinition's is_public field
            if node.name == "InstructionDefinition" && field_name == "visibility" {
                output.push_str("visibility: node.visibility, is_public: node.visibility.is_on_chain_callable()");
                first = false;
            } else {
                output.push_str(&format!("{}: node.{}", field_name, field_name));
                first = false;
            }
        }

        output.push_str(" },\n");
    }

    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate From/Into for Block structure
    output.push_str("// ============================================================================\n");
    output.push_str("// STRUCTURE CONVERSIONS\n");
    output.push_str("// ============================================================================\n\n");
    output.push_str("impl From<BlockNode> for AstNode {\n");
    output.push_str("    fn from(node: BlockNode) -> Self {\n");
    output.push_str("        AstNode::Block {\n");
    output.push_str("            statements: node.statements,\n");
    output.push_str("            kind: node.kind,\n");
    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");
    output.push_str("impl From<ProgramNode> for AstNode {\n");
    output.push_str("    fn from(node: ProgramNode) -> Self {\n");
    output.push_str("        AstNode::Program {\n");
    output.push_str("            program_name: node.program_name,\n");
    output.push_str("            field_definitions: node.field_definitions,\n");
    output.push_str("            instruction_definitions: node.instruction_definitions,\n");
    output.push_str("            event_definitions: node.event_definitions,\n");
    output.push_str("            account_definitions: node.account_definitions,\n");
    output.push_str("            interface_definitions: node.interface_definitions,\n");
    output.push_str("            import_statements: node.import_statements,\n");
    output.push_str("            init_block: node.init_block,\n");
    output.push_str("            constraints_block: node.constraints_block,\n");
    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    eprintln!("   Generated {} individual node structs", registry.nodes.len() - 2);
    eprintln!("   Generated {} category enums with From conversions", categories.len() - 1);
    eprintln!("   Generated complete bidirectional conversion system");

    Ok(output)
}
