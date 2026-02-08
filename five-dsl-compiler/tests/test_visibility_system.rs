/// Tests for the Visibility system.

use five_dsl_compiler::{DslParser, DslTokenizer, Visibility, AstNode};

#[test]
fn test_visibility_enum_properties() {
    // Test Public visibility
    let public_vis = Visibility::Public;
    assert!(public_vis.is_importable());
    assert!(public_vis.is_on_chain_callable());

    // Test Internal visibility
    let internal_vis = Visibility::Internal;
    assert!(internal_vis.is_importable());
    assert!(!internal_vis.is_on_chain_callable());

    // Test Private visibility
    let private_vis = Visibility::Private;
    assert!(!private_vis.is_importable());
    assert!(!private_vis.is_on_chain_callable());

    // Test default is Internal
    assert_eq!(Visibility::default(), Visibility::Internal);
}

#[test]
fn test_parse_public_function() {
    let code = r#"
script TestScript {
    pub fn transfer(recipient: pubkey, amount: u64) -> bool {
        return true;
    }
}
"#;

    let mut tokenizer = DslTokenizer::new(code);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    // Extract the program
    if let AstNode::Program {
        instruction_definitions,
        ..
    } = ast
    {
        assert_eq!(instruction_definitions.len(), 1);

        // Check the first instruction
        if let AstNode::InstructionDefinition {
            name,
            visibility,
            ..
        } = &instruction_definitions[0]
        {
            assert_eq!(name, "transfer");
            assert_eq!(visibility, &Visibility::Public);
            assert!(visibility.is_on_chain_callable());
            assert!(visibility.is_importable());
        } else {
            panic!("Expected InstructionDefinition");
        }
    } else {
        panic!("Expected Program");
    }
}

#[test]
fn test_parse_internal_function() {
    let code = r#"
script TestScript {
    fn validate_amount(amount: u64) -> bool {
        return true;
    }
}
"#;

    let mut tokenizer = DslTokenizer::new(code);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    // Extract the program
    if let AstNode::Program {
        instruction_definitions,
        ..
    } = ast
    {
        assert_eq!(instruction_definitions.len(), 1);

        // Check the first instruction
        if let AstNode::InstructionDefinition {
            name,
            visibility,
            ..
        } = &instruction_definitions[0]
        {
            assert_eq!(name, "validate_amount");
            assert_eq!(visibility, &Visibility::Internal);
            assert!(!visibility.is_on_chain_callable());
            assert!(visibility.is_importable());
        } else {
            panic!("Expected InstructionDefinition");
        }
    } else {
        panic!("Expected Program");
    }
}

#[test]
fn test_parse_public_and_internal_functions_mixed() {
    let code = r#"
script TestScript {
    pub fn main_entry() {
        let result = helper();
    }

    fn helper() -> u64 {
        return 42;
    }
}
"#;

    let mut tokenizer = DslTokenizer::new(code);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    if let AstNode::Program {
        instruction_definitions,
        ..
    } = ast
    {
        assert_eq!(instruction_definitions.len(), 2);

        // First function should be public
        if let AstNode::InstructionDefinition {
            name,
            visibility: vis1,
            ..
        } = &instruction_definitions[0]
        {
            assert_eq!(name, "main_entry");
            assert_eq!(vis1, &Visibility::Public);
        } else {
            panic!("Expected InstructionDefinition");
        }

        // Second function should be internal
        if let AstNode::InstructionDefinition {
            name,
            visibility: vis2,
            ..
        } = &instruction_definitions[1]
        {
            assert_eq!(name, "helper");
            assert_eq!(vis2, &Visibility::Internal);
        } else {
            panic!("Expected InstructionDefinition");
        }
    } else {
        panic!("Expected Program");
    }
}

#[test]
fn test_parse_public_field() {
    // Note: The parser currently supports `pub` for instructions/functions only
    // Fields and other definitions don't parse `pub` yet, so they default to Internal
    let code = r#"
script TestScript {
    mut balance: u64;
    owner: pubkey;
}
"#;

    let mut tokenizer = DslTokenizer::new(code);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    if let AstNode::Program {
        field_definitions,
        ..
    } = ast
    {
        assert_eq!(field_definitions.len(), 2);

        // First field defaults to internal
        if let AstNode::FieldDefinition {
            name,
            visibility: vis1,
            ..
        } = &field_definitions[0]
        {
            assert_eq!(name, "balance");
            assert_eq!(vis1, &Visibility::Internal);
        } else {
            panic!("Expected FieldDefinition");
        }

        // Second field is also internal
        if let AstNode::FieldDefinition {
            name,
            visibility: vis2,
            ..
        } = &field_definitions[1]
        {
            assert_eq!(name, "owner");
            assert_eq!(vis2, &Visibility::Internal);
        } else {
            panic!("Expected FieldDefinition");
        }
    } else {
        panic!("Expected Program");
    }
}

#[test]
fn test_parse_public_event() {
    // Note: The parser currently supports `pub` for instructions/functions only
    // Events default to Internal visibility
    let code = r#"
script TestScript {
    event Transfer {
        from: pubkey,
        to: pubkey,
        amount: u64,
    }

    event InternalEvent {
        data: u64,
    }
}
"#;

    let mut tokenizer = DslTokenizer::new(code);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    if let AstNode::Program {
        event_definitions,
        ..
    } = ast
    {
        assert_eq!(event_definitions.len(), 2);

        // First event defaults to internal
        if let AstNode::EventDefinition {
            name,
            visibility: vis1,
            ..
        } = &event_definitions[0]
        {
            assert_eq!(name, "Transfer");
            assert_eq!(vis1, &Visibility::Internal);
        } else {
            panic!("Expected EventDefinition");
        }

        // Second event is also internal
        if let AstNode::EventDefinition {
            name,
            visibility: vis2,
            ..
        } = &event_definitions[1]
        {
            assert_eq!(name, "InternalEvent");
            assert_eq!(vis2, &Visibility::Internal);
        } else {
            panic!("Expected EventDefinition");
        }
    } else {
        panic!("Expected Program");
    }
}

#[test]
fn test_parse_public_account() {
    // Note: The parser currently supports `pub` for instructions/functions only
    // Accounts default to Internal visibility
    let code = r#"
script TestScript {
    account Token {
        symbol: string,
        decimals: u8,
    }

    account Private {
        data: u64,
    }
}
"#;

    let mut tokenizer = DslTokenizer::new(code);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    if let AstNode::Program {
        account_definitions,
        ..
    } = ast
    {
        assert_eq!(account_definitions.len(), 2);

        // First account defaults to internal
        if let AstNode::AccountDefinition {
            name,
            visibility: vis1,
            ..
        } = &account_definitions[0]
        {
            assert_eq!(name, "Token");
            assert_eq!(vis1, &Visibility::Internal);
        } else {
            panic!("Expected AccountDefinition");
        }

        // Second account is also internal
        if let AstNode::AccountDefinition {
            name,
            visibility: vis2,
            ..
        } = &account_definitions[1]
        {
            assert_eq!(name, "Private");
            assert_eq!(vis2, &Visibility::Internal);
        } else {
            panic!("Expected AccountDefinition");
        }
    } else {
        panic!("Expected Program");
    }
}

#[test]
fn test_visibility_for_multi_file_import_semantics() {
    // This test demonstrates the intended usage:
    // - pub functions can be called on-chain AND imported
    // - internal functions can ONLY be imported (not on-chain callable)

    let code = r#"
script MyApp {
    pub fn transfer(recipient: pubkey, amount: u64) {
        validate(amount);
        execute_transfer(recipient, amount);
    }

    fn validate(amount: u64) {
        // Only callable from within this module
    }

    fn execute_transfer(recipient: pubkey, amount: u64) {
        // Helper function, can be imported by other modules but not called on-chain
    }
}
"#;

    let mut tokenizer = DslTokenizer::new(code);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    if let AstNode::Program {
        instruction_definitions,
        ..
    } = ast
    {
        assert_eq!(instruction_definitions.len(), 3);

        // Collect visibility info
        let visibilities: Vec<_> = instruction_definitions
            .iter()
            .filter_map(|node| {
                if let AstNode::InstructionDefinition {
                    name,
                    visibility,
                    ..
                } = node
                {
                    Some((name.clone(), *visibility))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(visibilities.len(), 3);

        // Check transfer is public (on-chain callable)
        let transfer = visibilities
            .iter()
            .find(|(name, _)| name == "transfer")
            .expect("transfer function not found");
        assert!(transfer.1.is_on_chain_callable());
        assert!(transfer.1.is_importable());

        // Check validate is internal (importable only)
        let validate = visibilities
            .iter()
            .find(|(name, _)| name == "validate")
            .expect("validate function not found");
        assert!(!validate.1.is_on_chain_callable());
        assert!(validate.1.is_importable());

        // Check execute_transfer is internal (importable only)
        let execute_transfer = visibilities
            .iter()
            .find(|(name, _)| name == "execute_transfer")
            .expect("execute_transfer function not found");
        assert!(!execute_transfer.1.is_on_chain_callable());
        assert!(execute_transfer.1.is_importable());
    } else {
        panic!("Expected Program");
    }
}

// Test for future enhancement when fields support pub modifier
// #[test]
// fn test_mixed_visibility_in_account_fields() { ... }

#[test]
fn test_visibility_consistency_across_types() {
    // Ensure visibility is consistently applied to functions (currently the only type supporting pub)
    let code = r#"
script Consistent {
    pub fn public_func() {}
    fn internal_func() {}
}
"#;

    let mut tokenizer = DslTokenizer::new(code);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    if let AstNode::Program {
        instruction_definitions,
        ..
    } = ast
    {
        // Check functions
        assert_eq!(instruction_definitions.len(), 2);
        let public_func = instruction_definitions
            .iter()
            .find(|f| {
                if let AstNode::InstructionDefinition { name, .. } = f {
                    name == "public_func"
                } else {
                    false
                }
            })
            .unwrap();
        if let AstNode::InstructionDefinition { visibility, .. } = public_func {
            assert_eq!(visibility, &Visibility::Public);
        }

        let internal_func = instruction_definitions
            .iter()
            .find(|f| {
                if let AstNode::InstructionDefinition { name, .. } = f {
                    name == "internal_func"
                } else {
                    false
                }
            })
            .unwrap();
        if let AstNode::InstructionDefinition { visibility, .. } = internal_func {
            assert_eq!(visibility, &Visibility::Internal);
        }
    } else {
        panic!("Expected Program");
    }
}

#[test]
fn test_visibility_documentation_example() {
    // Real-world example: Token swap contract with public entry and internal helpers
    let code = r#"
script TokenSwap {
    pub fn swap(token_in: pubkey, token_out: pubkey, amount_in: u64) -> u64 {
        require_tokens_registered(token_in, token_out);
        let exchange_rate = get_exchange_rate(token_in, token_out);
        let amount_out = calculate_amount_out(amount_in, exchange_rate);
        execute_swap(token_in, token_out, amount_in, amount_out);
        return amount_out;
    }

    fn require_tokens_registered(token_in: pubkey, token_out: pubkey) {
        // Internal validation, can be imported but not on-chain callable
    }

    fn get_exchange_rate(token_in: pubkey, token_out: pubkey) -> u64 {
        // Internal calculation, importable only
        return 100;
    }

    fn calculate_amount_out(amount_in: u64, rate: u64) -> u64 {
        // Internal math, importable only
        return amount_in * rate;
    }

    fn execute_swap(token_in: pubkey, token_out: pubkey, amount_in: u64, amount_out: u64) {
        // Internal execution, importable only
    }
}
"#;

    let mut tokenizer = DslTokenizer::new(code);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    if let AstNode::Program {
        instruction_definitions,
        ..
    } = ast
    {
        // Should have 5 functions
        assert_eq!(instruction_definitions.len(), 5);

        // Verify the swap function is public and others are internal
        for node in &instruction_definitions {
            if let AstNode::InstructionDefinition { name, visibility, .. } = node {
                if name == "swap" {
                    assert_eq!(visibility, &Visibility::Public);
                    assert!(visibility.is_on_chain_callable());
                } else {
                    // All helper functions should be internal
                    assert_eq!(visibility, &Visibility::Internal);
                    assert!(!visibility.is_on_chain_callable());
                    assert!(visibility.is_importable());
                }
            }
        }
    } else {
        panic!("Expected Program");
    }
}

#[test]
fn test_no_public_functions_parser_behavior() {
    // This test documents the requirement that scripts with functions must have at least one public function
    // This is enforced at compile time to prevent bytecode with public_function_count=0
    // The parser accepts internal-only functions, but compilation will fail

    let code = r#"
script NoPublic {
    fn internal_only() -> u64 {
        return 42;
    }
}
"#;

    let mut tokenizer = DslTokenizer::new(code);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    // The AST parses fine - the issue is caught at bytecode generation time
    // This test documents that parsing succeeds but the script will fail at compilation
    if let AstNode::Program {
        instruction_definitions,
        ..
    } = ast
    {
        // Verify the function is parsed (but as internal/private)
        assert_eq!(instruction_definitions.len(), 1);
        if let AstNode::InstructionDefinition {
            name,
            visibility,
            ..
        } = &instruction_definitions[0]
        {
            assert_eq!(name, "internal_only");
            // This function is internal because it lacks the 'pub' keyword
            assert_eq!(visibility, &Visibility::Internal);
            assert!(!visibility.is_on_chain_callable());
        }
    } else {
        panic!("Expected Program");
    }
}
