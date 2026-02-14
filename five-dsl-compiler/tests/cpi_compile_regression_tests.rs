use five_dsl_compiler::DslCompiler;
use five_protocol::opcodes;
use std::fs;
use std::path::PathBuf;

#[test]
fn cpi_minimal_interface_call_compiles_without_jump_verification_failure() {
    let source = r#"
        interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
            transfer @discriminator(3) (
                source: Account,
                destination: Account,
                authority: Account,
                amount: u64
            );
        }

        pub cpi_only(
            user_token_a: account @mut,
            pool_token_a_vault: account @mut,
            user_authority: account @signer,
            amount_a: u64
        ) {
            SPLToken.transfer(user_token_a, pool_token_a_vault, user_authority, amount_a);
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source)
        .expect("minimal CPI interface call should compile");
    assert!(
        bytecode.iter().any(|op| *op == opcodes::INVOKE),
        "interface CPI call should emit INVOKE opcode"
    );
}

#[test]
fn amm_like_cpi_program_compiles_and_emits_invoke() {
    let source = r#"
        interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
            transfer @discriminator(3) (
                source: Account,
                destination: Account,
                authority: Account,
                amount: u64
            );

            mint_to @discriminator(7) (
                mint: Account,
                destination: Account,
                authority: Account,
                amount: u64
            );
        }

        account Pool {
            reserve_a: u64;
            reserve_b: u64;
            lp_supply: u64;
            fee_numerator: u64;
            fee_denominator: u64;
            authority: pubkey;
        }

        pub add_liquidity(
            pool: Pool @mut,
            user_token_a: account @mut,
            user_token_b: account @mut,
            pool_token_a_vault: account @mut,
            pool_token_b_vault: account @mut,
            lp_mint: account @mut,
            user_lp_account: account @mut,
            user_authority: account @signer,
            amount_a: u64,
            amount_b: u64
        ) {
            require(amount_a > 0);
            require(amount_b > 0);

            let mut liquidity: u64 = 0;
            if (pool.lp_supply == 0) {
                liquidity = amount_a + amount_b;
            } else {
                require(amount_a * pool.reserve_b == amount_b * pool.reserve_a);
                liquidity = (amount_a * pool.lp_supply) / pool.reserve_a;
            }

            SPLToken.transfer(user_token_a, pool_token_a_vault, user_authority, amount_a);
            SPLToken.transfer(user_token_b, pool_token_b_vault, user_authority, amount_b);
            SPLToken.mint_to(lp_mint, user_lp_account, user_authority, liquidity);
        }
    "#;

    let bytecode =
        DslCompiler::compile_dsl(source).expect("AMM-like CPI source should compile cleanly");
    let invoke_count = bytecode.iter().filter(|op| **op == opcodes::INVOKE).count();
    assert!(invoke_count >= 1, "expected INVOKE opcode(s), found {}", invoke_count);
}

#[test]
fn one_shot_amm_fixture_compiles_when_present() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let amm_path = workspace_root.join("5ive-amm/src/main.v");
    if !amm_path.exists() {
        eprintln!("Skipping AMM fixture compile regression; missing {}", amm_path.display());
        return;
    }

    let source = fs::read_to_string(&amm_path)
        .expect("expected readable 5ive-amm fixture source");
    let bytecode = DslCompiler::compile_dsl(&source)
        .expect("5ive-amm fixture should compile without InvalidInstructionPointer");
    assert!(
        bytecode.iter().any(|op| *op == opcodes::INVOKE),
        "fixture should include CPI INVOKE opcode(s)"
    );
}
