use five_dsl_compiler::DslCompiler;
use five_protocol::opcodes::{CALL_EXTERNAL, INVOKE};

#[test]
fn docs_quick_start_snippet_compiles() {
    let source = r#"
        account Counter {
            value: u64;
            authority: pubkey;
        }

        pub init_counter(counter: Counter @mut @init, authority: account @signer) {
            counter.value = 0;
            counter.authority = authority.key;
        }

        pub increment(counter: Counter @mut, authority: account @signer) {
            require(counter.authority == authority.key);
            counter.value = counter.value + 1;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("quick start snippet should compile");
    assert!(!bytecode.is_empty());
}

#[test]
fn docs_external_import_snippet_compiles_to_call_external() {
    let source = r#"
        use "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"::{transfer};

        pub settle(
            source_account: account @mut,
            destination_account: account @mut,
            owner: account @signer,
            token_bytecode: account
        ) {
            transfer(source_account, destination_account, owner, 50);
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("external import snippet should compile");
    assert!(
        bytecode.iter().any(|op| *op == CALL_EXTERNAL),
        "external import snippet should emit CALL_EXTERNAL"
    );
}

#[test]
fn docs_ambiguous_import_snippet_fails_compile() {
    let source = r#"
        use "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"::{transfer};
        use "11111111111111111111111111111111"::{transfer};

        pub execute(token_a: account, token_b: account) {
            transfer();
        }
    "#;

    let result = DslCompiler::compile_dsl(source);
    assert!(result.is_err(), "ambiguous imported symbol should fail");
}

#[test]
fn docs_interface_cpi_snippet_compiles() {
    let source = r#"
        interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
            transfer @discriminator(3) (
                from: account,
                to: account,
                authority: account,
                amount: u64
            );
        }

        pub cpi_transfer(from: account @mut, to: account @mut, authority: account @signer) {
            SPLToken.transfer(from, to, authority, 100);
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("interface CPI snippet should compile");
    assert!(!bytecode.is_empty());
}

#[test]
fn docs_imported_interface_snippet_compiles_to_call_external() {
    let source = r#"
        use "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"::{interface TokenOps};

        pub route_call(
            TokenOps: account,
            from: account @mut,
            to: account @mut,
            authority: account @signer
        ) {
            TokenOps.transfer(from, to, authority, 100);
        }
    "#;

    let bytecode =
        DslCompiler::compile_dsl(source).expect("imported interface snippet should compile");
    assert!(
        bytecode.iter().any(|op| *op == CALL_EXTERNAL),
        "imported interface method call should emit CALL_EXTERNAL"
    );
    assert!(
        !bytecode.iter().any(|op| *op == INVOKE),
        "imported interface method call should not emit INVOKE"
    );
}
