use five_dsl_compiler::{AstNode, DslCompiler, DslParser, DslTokenizer, TypeNode};

#[test]
fn parses_type_record_and_alias() {
    let source = r#"
        script types_demo {
            pub type Clock {
                slot: u64,
                unix_timestamp: i64,
            }

            pub type ProgramAddress = (pubkey, u8);
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("parse");

    let AstNode::Program { type_definitions, .. } = ast else {
        panic!("expected program");
    };

    assert_eq!(type_definitions.len(), 2);

    let AstNode::TypeDefinition { definition, .. } = &type_definitions[0] else {
        panic!("expected type definition");
    };
    assert!(matches!(definition.as_ref(), TypeNode::Struct { .. }));

    let AstNode::TypeDefinition { definition, .. } = &type_definitions[1] else {
        panic!("expected alias type definition");
    };
    assert!(matches!(definition.as_ref(), TypeNode::Tuple { .. }));
}

#[test]
fn parses_pub_use_reexport() {
    let source = r#"
        script prelude_like {
            pub use std::types::{Clock};
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("parse");

    let AstNode::Program {
        import_statements, ..
    } = ast
    else {
        panic!("expected program");
    };

    assert_eq!(import_statements.len(), 1);
    let AstNode::ImportStatement { is_reexport, .. } = &import_statements[0] else {
        panic!("expected import statement");
    };
    assert!(*is_reexport);
}

#[test]
fn clock_field_access_typechecks() {
    let source = r#"
        script clock_field {
            pub fn run() -> u64 {
                return get_clock().slot;
            }
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("clock slot access should compile");
    assert!(bytecode.starts_with(&five_protocol::FIVE_MAGIC));
}
