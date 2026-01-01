
    #[test]
    fn test_parse_instruction_def_attributes_order() {
        let source = "
            instruction test_func(
                account1: Account @mut @init,
                account2: Account @init @mut
            ) {}
        ";
        let mut tokenizer = crate::tokenizer::DslTokenizer::new(source);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = DslParser::new(tokens);
        
        // Skip to instruction definition
        while parser.current_token.kind() != crate::tokenizer::TokenKind::Instruction {
            parser.advance();
        }
        
        let node = parser.parse_instruction_definition().unwrap();
        
        if let AstNode::InstructionDefinition { parameters, .. } = node {
            assert_eq!(parameters.len(), 2);
            
            // Check account1 (@mut @init)
            let p1 = &parameters[0];
            assert!(p1.is_init, "account1 should be init");
            assert!(p1.init_config.is_some(), "account1 init_config should be Some");
            
            // Check account2 (@init @mut)
            let p2 = &parameters[1];
            assert!(p2.is_init, "account2 should be init");
            assert!(p2.init_config.is_some(), "account2 init_config should be Some");
        } else {
            panic!("Expected InstructionDefinition");
        }
    }
