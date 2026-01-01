//! Grammar generator CLI tool
//!
//! Generates tree-sitter grammar.js from node_metadata.toml single source of truth.
//! Usage: cargo run --bin generate-grammar [output_path]

use five_dsl_compiler::ast::NODE_REGISTRY;
use std::fs;
use std::path::Path;

fn main() {
    // Get output path from arguments or use default
    let output_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../five-tree-sitter/grammar.js".to_string());

    println!("🔨 Generating grammar.js from node_metadata.toml...");
    println!("   Registry loaded with {} AST nodes", NODE_REGISTRY.nodes.len());

    match generate_grammar(&output_path) {
        Ok(size) => {
            println!("✅ Generated grammar.js ({} bytes)", size);
            println!("   Output: {}", output_path);
        }
        Err(e) => {
            eprintln!("❌ Failed to generate grammar: {}", e);
            std::process::exit(1);
        }
    }
}

/// Generate grammar.js from registry
fn generate_grammar(output_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    // Generate the grammar content
    let grammar = generate_grammar_content(&NODE_REGISTRY)?;

    // Create parent directories if needed
    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    // Write to file
    fs::write(output_path, &grammar)?;
    Ok(grammar.len())
}

/// Generate tree-sitter grammar.js content
fn generate_grammar_content(
    registry: &five_dsl_compiler::ast::registry::NodeRegistry,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut output = String::new();

    // Module header
    output.push_str("module.exports = grammar({\n");
    output.push_str("  // Keep legacy name to avoid breaking bindings; grammar targets Five DSL\n");
    output.push_str("  name: \"stacks\",\n");
    output.push('\n');
    output.push_str("  extras: ($) => [/\\s/, $.comment],\n");
    output.push('\n');
    output.push_str("  conflicts: ($) => [[$.field_access, $.method_call_expression]],\n");
    output.push('\n');
    output.push_str("  rules: {\n");

    // Generate rules for each node with grammar metadata
    let mut rules_generated = 0;
    let mut nodes_with_grammar = Vec::new();

    for (_name, node) in registry.nodes.iter() {
        if let Some(grammar) = &node.grammar {
            nodes_with_grammar.push((node.clone(), grammar.clone()));
        }
    }

    // Sort by rule name for consistent output
    nodes_with_grammar.sort_by(|a, b| a.1.rule_name.cmp(&b.1.rule_name));

    for (_node, grammar) in nodes_with_grammar {
        output.push_str(&format!(
            "    {}: ($) => {},\n",
            grammar.rule_name, grammar.rule
        ));
        rules_generated += 1;
    }

    // Add placeholder rules that aren't in metadata yet
    // (would normally extract these from five-tree-sitter/grammar.js)
    output.push('\n');
    output.push_str("    // Helper rules\n");
    output.push_str("    comment: ($) => token(seq('//', /[^\\n]*/)),\n");
    output.push_str("    identifier: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,\n");
    output.push_str("    number_literal: ($) => /[0-9]+/,\n");
    output.push_str("    escape_sequence: ($) => token(seq('\\\\', /./)),\n");
    output.push_str("    array_element_list: ($) => seq($._expression, repeat(seq(',', $._expression)), optional(',')),\n");
    output.push_str("    argument_list: ($) => seq($._expression, repeat(seq(',', $._expression)), optional(',')),\n");
    output.push_str("    pattern_list: ($) => seq($.identifier, repeat(seq(',', $.identifier)), optional(',')),\n");
    output.push_str("    type_argument_list: ($) => seq($._type_expression, repeat(seq(',', $._type_expression)), optional(',')),\n");

    // Additional required rules
    output.push_str("    parameter_list: ($) => seq($.identifier, repeat(seq(',', $.identifier))),\n");
    output.push_str("    tuple_pattern: ($) => seq('(', $.pattern_list, ')'),\n");
    output.push_str("    generic_type: ($) => seq($.identifier, '<', $.type_argument_list, '>'),\n");
    output.push_str("    array_type: ($) => seq('[', $._type_expression, ']'),\n");
    output.push_str("    tuple_type: ($) => seq('(', $.type_argument_list, ')'),\n");
    output.push_str("    _top_level_item: ($) => choice($._statement, $.instruction_definition),\n");

    // Placeholder expressions and statements (would be generated from category data)
    output.push('\n');
    output.push_str("    // Placeholder rules - generated from category metadata\n");
    output.push_str("    _expression: ($) => choice(\n");
    output.push_str("      $.identifier,\n");
    output.push_str("      $.number_literal,\n");
    output.push_str("      $.string_literal,\n");
    output.push_str("      $.field_access,\n");
    output.push_str("      $.array_access,\n");
    output.push_str("      $.call_expression,\n");
    output.push_str("      $.binary_expression,\n");
    output.push_str("      $.unary_expression,\n");
    output.push_str("      $.array_literal,\n");
    output.push_str("      $.tuple_literal\n");
    output.push_str("    ),\n");

    output.push('\n');
    output.push_str("    _statement: ($) => choice(\n");
    output.push_str("      $.assignment_statement,\n");
    output.push_str("      $.if_statement,\n");
    output.push_str("      $.while_statement,\n");
    output.push_str("      $.for_loop,\n");
    output.push_str("      $.return_statement,\n");
    output.push_str("      $.emit_statement,\n");
    output.push_str("      $.let_statement,\n");
    output.push_str("      $.require_statement,\n");
    output.push_str("      $.match_expression\n");
    output.push_str("    ),\n");

    output.push('\n');
    output.push_str("    _type_expression: ($) => choice(\n");
    output.push_str("      $.identifier,\n");
    output.push_str("      $.generic_type,\n");
    output.push_str("      $.array_type,\n");
    output.push_str("      $.tuple_type\n");
    output.push_str("    ),\n");

    output.push('\n');
    output.push_str("    _assignable_expression: ($) => choice(\n");
    output.push_str("      $.identifier,\n");
    output.push_str("      $.field_access,\n");
    output.push_str("      $.array_access\n");
    output.push_str("    ),\n");

    // Close module
    output.push_str("  },\n");
    output.push_str("});\n");

    // Print summary
    eprintln!("   Generated {} grammar rules from metadata", rules_generated);

    Ok(output)
}
