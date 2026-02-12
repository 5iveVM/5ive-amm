use five_dsl_compiler::compiler::DslCompiler;
use five_protocol::opcodes::CALL_EXTERNAL;

fn assert_external_call_compiles(source: &str) {
    let bytecode = DslCompiler::compile_dsl(source).expect("source should compile");
    assert!(
        bytecode.iter().any(|op| *op == CALL_EXTERNAL),
        "bytecode should contain CALL_EXTERNAL"
    );
    assert!(!source.contains("ext0::"), "source should not use ext0 namespace");
    assert!(
        source.contains("token_bytecode: account"),
        "source should bind token bytecode as explicit account"
    );
}

#[test]
fn amm_style_external_transfer_regression() {
    let source = r#"
        use "11111111111111111111111111111111"::{transfer};

        pub fn swap_a_to_b(
            trader_token_a: account @mut,
            pool_token_a: account @mut,
            pool_token_b: account @mut,
            trader_token_b: account @mut,
            trader: account @signer,
            pool_authority: account @signer,
            amount_in: u64,
            amount_out: u64,
            token_bytecode: account
        ) {
            transfer(trader_token_a, pool_token_a, trader, amount_in);
            transfer(pool_token_b, trader_token_b, pool_authority, amount_out);
        }
    "#;
    assert_external_call_compiles(source);
}

#[test]
fn lending_style_external_transfer_regression() {
    let source = r#"
        use "11111111111111111111111111111111"::{transfer};

        pub fn repay(
            payer_liquidity_token: account @mut,
            reserve_liquidity_token: account @mut,
            payer: account @signer,
            amount: u64,
            token_bytecode: account
        ) {
            transfer(payer_liquidity_token, reserve_liquidity_token, payer, amount);
        }
    "#;
    assert_external_call_compiles(source);
}

#[test]
fn perps_style_external_transfer_regression() {
    let source = r#"
        use "11111111111111111111111111111111"::{transfer};

        pub fn open_position(
            owner_collateral_token: account @mut,
            market_collateral_vault: account @mut,
            owner: account @signer,
            collateral: u64,
            token_bytecode: account
        ) {
            transfer(owner_collateral_token, market_collateral_vault, owner, collateral);
        }
    "#;
    assert_external_call_compiles(source);
}

#[test]
fn vault_style_external_transfer_regression() {
    let source = r#"
        use "11111111111111111111111111111111"::{transfer};

        pub fn deposit(
            owner_asset_token: account @mut,
            vault_asset_token: account @mut,
            owner: account @signer,
            amount: u64,
            token_bytecode: account
        ) {
            transfer(owner_asset_token, vault_asset_token, owner, amount);
        }
    "#;
    assert_external_call_compiles(source);
}

#[test]
fn token_template_import_contract_regression() {
    let source = r#"
        use "11111111111111111111111111111111"::{transfer, mint_to, burn, approve, transfer_from};

        pub fn smoke(
            from_account: account @mut,
            to_account: account @mut,
            owner: account @signer,
            amount: u64,
            token_bytecode: account
        ) {
            transfer(from_account, to_account, owner, amount);
        }
    "#;
    assert_external_call_compiles(source);
}
