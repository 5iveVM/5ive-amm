/// Grammar attribute macro for Five DSL AST nodes
///
/// This attribute is purely informational and does not generate code.
/// It attaches grammar metadata to AST enum variants for later code generation
/// by the `generate-grammar` CLI tool.
///
/// # Example
///
/// ```ignore
/// #[grammar(
///     name = "account_definition",
///     rule = r#"seq("account", field("name", $.identifier), "{", repeat($.field_definition), "}")"#
/// )]
/// AccountDefinition {
///     name: String,
///     fields: Vec<StructField>,
///     visibility: Visibility,
/// }
/// ```
///
/// Supported attributes:
/// - `name`: The rule name in grammar.js (usually snake_case)
/// - `rule`: The tree-sitter rule definition
/// - `precedence`: Optional precedence level for expressions (integer)
/// - `associativity`: Optional associativity ("left" or "right")
use proc_macro::TokenStream;

/// Attribute macro for attaching tree-sitter grammar rules to AST nodes
///
/// This macro does minimal validation and returns the item unchanged.
/// The `generate-grammar` CLI tool will parse this attribute using `syn` to
/// extract rule definitions and generate grammar.js.
#[proc_macro_attribute]
pub fn grammar(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // The attribute is purely metadata - just pass through the item unchanged
    // The generator tool will read these attributes at compile time via syn
    item
}
