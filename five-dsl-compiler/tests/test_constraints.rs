use five_dsl_compiler::ast::AstNode;
use five_dsl_compiler::*;
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;

#[test]
fn test_constraints_signer_valid() {
    let source = r#"
        script constraints_test {
            pub deposit(user: Account @signer) {
                // Body
            }
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");

    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    // Check AST structure
    if let AstNode::Program {
        instruction_definitions,
        ..
    } = &ast
    {
        if let AstNode::InstructionDefinition { parameters, .. } = &instruction_definitions[0] {
            let param = &parameters[0];
            assert!(param.attributes.iter().any(|a| a.name == "signer"));
        } else {
            panic!("Expected InstructionDefinition");
        }
    } else {
        panic!("Expected Program node");
    }

    // Type check
    let mut type_checker = DslTypeChecker::new();
    type_checker.check_types(&ast).expect("Should type check");

    // Generate bytecode
    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    // Verify CHECK_SIGNER opcode presence (0x70)
    assert!(
        bytecode.contains(&CHECK_SIGNER),
        "Bytecode should contain CHECK_SIGNER"
    );
}

#[test]
fn test_constraints_signer_invalid_type() {
    let source = r#"
        script constraints_test {
            pub deposit(amount: u64 @signer) {
                // Body
            }
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    // Type check should fail
    let mut type_checker = DslTypeChecker::new();
    let result = type_checker.check_types(&ast);

    match result {
        Err(VMError::TypeMismatch) => {} // Expected
        _ => panic!(
            "Expected TypeMismatch error for @signer on non-account, got {:?}",
            result
        ),
    }
}

#[test]
fn test_constraints_has_valid_single() {
    let source = r#"
        script constraints_test {
            account Vault {
                owner: Pubkey,
            }

            pub deposit(
                vault: Vault @has(owner),
                owner: Account
            ) {
                // Body
            }
        }
    "#;

    let bytecode = compile_source(source).expect("Should compile");

    // Should have LOAD_FIELD (0x43), GET_KEY (0x57), EQ (0x27), REQUIRE (0x04)
    assert!(bytecode.contains(&LOAD_FIELD));
    assert!(bytecode.contains(&GET_KEY));
    assert!(bytecode.contains(&EQ));
    assert!(bytecode.contains(&REQUIRE));
}

#[test]
fn test_constraints_has_valid_multiple() {
    let source = r#"
        script constraints_test {
            account Vault {
                owner: Pubkey,
                token_account: Pubkey,
            }

            pub deposit(
                vault: Vault @has(owner, token_account),
                owner: Account,
                token_account: Account
            ) {
                // Body
            }
        }
    "#;

    let bytecode = compile_source(source).expect("Should compile");

    // Should iterate twice, so multiple EQ/REQUIRE
    let require_count = bytecode.iter().filter(|&&b| b == REQUIRE).count();
    assert!(require_count >= 2, "Should have at least 2 REQUIREs");
}

#[test]
fn test_constraints_has_invalid_target_missing() {
    let source = r#"
        script constraints_test {
            pub deposit(
                vault: Account @has(missing_param)
            ) {
                // Body
            }
        }
    "#;

    let result = compile_source(source);
    match result {
        Err(VMError::InvalidScript) => {} // Target param not found
        _ => panic!(
            "Expected InvalidScript error for missing target in @has, got {:?}",
            result
        ),
    }
}

#[test]
fn test_constraints_has_invalid_non_account() {
    let source = r#"
        script constraints_test {
            pub deposit(
                amount: u64 @has(owner)
            ) {
                // Body
            }
        }
    "#;

    let result = compile_source(source);
    match result {
        Err(VMError::TypeMismatch) => {}
        _ => panic!(
            "Expected TypeMismatch for @has on non-account, got {:?}",
            result
        ),
    }
}

// Helper for quick compilation
fn compile_source(source: &str) -> std::result::Result<Vec<u8>, VMError> {
    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().map_err(|_| VMError::UnexpectedToken)?;

    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().map_err(|_| VMError::InvalidScript)?;

    let mut type_checker = DslTypeChecker::new();
    type_checker.check_types(&ast)?;

    let mut generator = DslBytecodeGenerator::new();
    generator.generate(&ast)
}
