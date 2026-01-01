use five_dsl_compiler::ast::AstNode;
use five_dsl_compiler::*;
use five_protocol::opcodes::REQUIRE;
use five_protocol::FIVE_MAGIC;

#[test]
fn test_attribute_macros_requires_desugaring() {
    let source = r#"
        script requires_test {
            pub deposit(@requires(amount > 0) amount: u64) {
                // Body
                let x = amount;
            }
        }
    "#;

    // 1. Tokenize
    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");

    // 2. Parse
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");
    
    // Check AST structure for attribute
    if let AstNode::Program { instruction_definitions, .. } = &ast {
        if let AstNode::InstructionDefinition { parameters, .. } = &instruction_definitions[0] {
            assert_eq!(parameters.len(), 1);
            let param = &parameters[0];
            assert_eq!(param.name, "amount");
            assert!(!param.attributes.is_empty());
            assert_eq!(param.attributes[0].name, "requires");
            assert_eq!(param.attributes[0].args.len(), 1);
        } else {
            panic!("Expected InstructionDefinition");
        }
    } else {
        panic!("Expected Program");
    }

    // 3. Type check
    let mut type_checker = DslTypeChecker::new();
    type_checker.check_types(&ast).expect("Should type check");

    // 4. Generate bytecode
    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    // Verify bytecode is valid
    assert!(bytecode.starts_with(&FIVE_MAGIC));
    
    // Verify bytecode contains REQUIRE opcode
    // We expect: ... [Generate Condition] REQUIRE [Generate Body] ...
    assert!(bytecode.contains(&REQUIRE), "Bytecode should contain REQUIRE opcode generated from @requires");
}

#[test]
fn test_attribute_macros_multiple_attributes() {
    let source = r#"
        script multiple_attributes {
            // Test mixing @mut, @signer, and @requires
            // Also test trailing attribute syntax
            pub withdraw(
                @signer owner: Account, 
                amount: u64 @requires(amount <= 1000)
            ) {
                // Body
                let x = amount;
            }
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");
    
    if let AstNode::Program { instruction_definitions, .. } = &ast {
        if let AstNode::InstructionDefinition { parameters, .. } = &instruction_definitions[0] {
            assert_eq!(parameters.len(), 2);
            
            // Check first parameter: @signer owner: Account
            let p1 = &parameters[0];
            assert_eq!(p1.name, "owner");
            assert!(p1.attributes.iter().any(|a| a.name == "signer"));
            
            // Check second parameter: amount: u64 @requires(...)
            let p2 = &parameters[1];
            assert_eq!(p2.name, "amount");
            assert!(p2.attributes.iter().any(|a| a.name == "requires"));
        }
    }

    let mut type_checker = DslTypeChecker::new();
    type_checker.check_types(&ast).expect("Should type check");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");
    
    assert!(bytecode.contains(&REQUIRE), "Bytecode should contain REQUIRE opcode");
}
