use crate::ast::AstNode;
use crate::parser::DslParser;
use crate::tokenizer::DslTokenizer;

fn parse_program(source: &str) -> AstNode {
    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("tokenization should succeed");
    let mut parser = DslParser::new(tokens);
    parser.parse().expect("parse should succeed")
}

fn first_instruction(program: &AstNode) -> &AstNode {
    if let AstNode::Program {
        instruction_definitions,
        ..
    } = program
    {
        instruction_definitions
            .first()
            .expect("expected at least one instruction")
    } else {
        panic!("expected AstNode::Program");
    }
}

#[test]
fn session_attribute_parses_keyed_arguments() {
    let source = r#"
script SessionKeyed {
    account Session {
        authority: pubkey;
        delegate: pubkey;
        nonce: u64;
        status: u8;
        expires_at_slot: u64;
    }

    pub play(
        session: Session @session(delegate=delegate, authority=authority, nonce_field=nonce, current_slot=slot),
        delegate: account @signer,
        authority: account,
        nonce: u64,
        slot: u64
    ) { }
}
"#;

    let ast = parse_program(source);
    let instruction = first_instruction(&ast);

    let AstNode::InstructionDefinition { parameters, .. } = instruction else {
        panic!("expected instruction definition");
    };

    let session_param = parameters
        .iter()
        .find(|p| p.name == "session")
        .expect("expected session parameter");
    let session_attr = session_param
        .attributes
        .iter()
        .find(|a| a.name == "session")
        .expect("expected @session attribute");

    assert_eq!(session_attr.args.len(), 4);
    for expected_key in ["delegate", "authority", "nonce_field", "current_slot"] {
        assert!(
            session_attr.args.iter().any(|arg| matches!(
                arg,
                AstNode::Assignment { target, .. } if target == expected_key
            )),
            "missing keyed arg {}",
            expected_key
        );
    }
}

#[test]
fn session_attribute_parses_provenance_arguments() {
    let source = r#"
script SessionProvenance {
    account Session {
        authority: pubkey;
        delegate: pubkey;
        manager_script_account: pubkey;
        manager_code_hash: pubkey;
        manager_version: u8;
    }

    pub play(
        session: Session @session(
            delegate=delegate,
            authority=authority,
            manager_script_account=manager_script,
            manager_code_hash=manager_hash,
            manager_version=1
        ),
        delegate: account @signer,
        authority: account,
        manager_script: account,
        manager_hash: pubkey
    ) { }
}
"#;

    let ast = parse_program(source);
    let instruction = first_instruction(&ast);

    let AstNode::InstructionDefinition { parameters, .. } = instruction else {
        panic!("expected instruction definition");
    };

    let session_param = parameters
        .iter()
        .find(|p| p.name == "session")
        .expect("expected session parameter");
    let session_attr = session_param
        .attributes
        .iter()
        .find(|a| a.name == "session")
        .expect("expected @session attribute");

    for expected_key in [
        "delegate",
        "authority",
        "manager_script_account",
        "manager_code_hash",
        "manager_version",
    ] {
        assert!(
            session_attr.args.iter().any(|arg| matches!(
                arg,
                AstNode::Assignment { target, .. } if target == expected_key
            )),
            "missing keyed arg {}",
            expected_key
        );
    }
}

#[test]
fn session_attribute_still_parses_positional_arguments() {
    let source = r#"
script SessionPositional {
    account Session {
        authority: pubkey;
        delegate: pubkey;
        nonce: u64;
        status: u8;
        expires_at_slot: u64;
    }

    pub play(
        session: Session @session(delegate, authority, nonce, slot),
        delegate: account @signer,
        authority: account,
        nonce: u64,
        slot: u64
    ) { }
}
"#;

    let ast = parse_program(source);
    let instruction = first_instruction(&ast);

    let AstNode::InstructionDefinition { parameters, .. } = instruction else {
        panic!("expected instruction definition");
    };

    let session_param = parameters
        .iter()
        .find(|p| p.name == "session")
        .expect("expected session parameter");
    let session_attr = session_param
        .attributes
        .iter()
        .find(|a| a.name == "session")
        .expect("expected @session attribute");

    assert_eq!(session_attr.args.len(), 4);
    assert!(matches!(&session_attr.args[0], AstNode::Identifier(name) if name == "delegate"));
    assert!(matches!(&session_attr.args[1], AstNode::Identifier(name) if name == "authority"));
}

#[test]
fn session_attribute_parses_authority_attached_form() {
    let source = r#"
script SessionAuthorityAttached {
    pub play(
        player: account @mut,
        authority: account @session(delegate=delegate, nonce_field=session_nonce, bind_account=player),
        delegate: account @signer,
        session_nonce: u64
    ) { }
}
"#;

    let ast = parse_program(source);
    let instruction = first_instruction(&ast);

    let AstNode::InstructionDefinition { parameters, .. } = instruction else {
        panic!("expected instruction definition");
    };

    let authority_param = parameters
        .iter()
        .find(|p| p.name == "authority")
        .expect("expected authority parameter");
    let session_attr = authority_param
        .attributes
        .iter()
        .find(|a| a.name == "session")
        .expect("expected @session attribute on authority");

    for expected_key in ["delegate", "nonce_field", "bind_account"] {
        assert!(
            session_attr.args.iter().any(|arg| matches!(
                arg,
                AstNode::Assignment { target, .. } if target == expected_key
            )),
            "missing keyed arg {}",
            expected_key
        );
    }
}
