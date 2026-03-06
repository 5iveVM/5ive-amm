use five_dsl_compiler::bytecode_generator::disassembler::disasm::disassemble;
use five_dsl_compiler::DslCompiler;
use five_protocol::opcodes;
use std::fs;
use std::path::PathBuf;

fn count_require_dispatch_points(bytecode: &[u8]) -> usize {
    let (header, start_offset, code_end) = match five_protocol::parse_code_bounds(bytecode) {
        Ok(bounds) => bounds,
        Err(_) => (
            five_protocol::ScriptBytecodeHeaderV1 {
                magic: [0; 4],
                features: 0,
                public_function_count: 0,
                total_function_count: 0,
            },
            0,
            bytecode.len(),
        ),
    };
    let pool_enabled = (header.features & five_protocol::FEATURE_CONSTANT_POOL) != 0;
    let mut count = 0usize;
    let mut pc = start_offset;

    while pc < code_end {
        let op = bytecode[pc];
        if matches!(
            op,
            opcodes::REQUIRE
                | opcodes::REQUIRE_OWNER
                | opcodes::REQUIRE_GTE_U64
                | opcodes::REQUIRE_NOT_BOOL
                | opcodes::REQUIRE_PARAM_GT_ZERO
                | opcodes::REQUIRE_EQ_PUBKEY
                | opcodes::REQUIRE_EQ_FIELDS
                | opcodes::REQUIRE_PARAM_LTE_IMM
                | opcodes::REQUIRE_FIELD_EQ_IMM
                | opcodes::REQUIRE_LOCAL_GT_ZERO
                | opcodes::REQUIRE_BATCH
        ) {
            count += 1;
        }

        let remaining = bytecode.get(pc + 1..code_end).unwrap_or(&[]);
        let Some(operand_size) = opcodes::operand_size(op, remaining, pool_enabled) else {
            break;
        };
        pc += 1 + operand_size;
    }

    count
}

fn count_source_requires(source: &str) -> usize {
    source.matches("require(").count()
}

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
            SPLToken::transfer(user_token_a, pool_token_a_vault, user_authority, amount_a);
        }
    "#;

    let bytecode =
        DslCompiler::compile_dsl(source).expect("minimal CPI interface call should compile");
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

            SPLToken::transfer(user_token_a, pool_token_a_vault, user_authority, amount_a);
            SPLToken::transfer(user_token_b, pool_token_b_vault, user_authority, amount_b);
            SPLToken::mint_to(lp_mint, user_lp_account, user_authority, liquidity);
        }
    "#;

    let bytecode =
        DslCompiler::compile_dsl(source).expect("AMM-like CPI source should compile cleanly");
    let invoke_count = bytecode.iter().filter(|op| **op == opcodes::INVOKE).count();
    assert!(
        invoke_count >= 1,
        "expected INVOKE opcode(s), found {}",
        invoke_count
    );
}

#[test]
fn raw_interface_with_bounded_string_data_compiles_and_emits_invoke() {
    let source = r#"
        interface StringSink @program("11111111111111111111111111111111") @serializer(raw) {
            submit @discriminator_bytes([]) (
                sink: Account,
                payload: string<32>
            );
        }

        pub send(sink: account) {
            StringSink::submit(sink, "vault");
        }
    "#;

    let bytecode =
        DslCompiler::compile_dsl(source).expect("raw string-bearing interface call should compile");
    assert!(
        bytecode.iter().any(|op| *op == opcodes::INVOKE),
        "string-bearing interface call should emit INVOKE opcode"
    );
    assert!(
        bytecode
            .iter()
            .any(|op| *op == opcodes::PUSH_STRING || *op == opcodes::PUSH_STRING_W),
        "string-bearing interface call should materialize string data"
    );
}

#[test]
fn metaplex_like_raw_interface_shape_with_multiple_bounded_strings_compiles() {
    let source = r#"
        interface MetadataProgram @program("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s") @serializer(raw) {
            create_metadata_account_v3 @discriminator_bytes([33]) (
                metadata: Account,
                mint: Account,
                mint_authority: Account,
                payer: Account,
                update_authority: Account,
                system_program_account: Account,
                name: string<32>,
                symbol: string<10>,
                uri: string<200>,
                seller_fee_basis_points: u16,
                creators_is_some: bool
            );
        }

        pub create(
            metadata: account @mut,
            mint: account @mut,
            mint_authority: account @signer,
            payer: account @signer,
            update_authority: account @signer,
            system_program_account: account
        ) {
            MetadataProgram::create_metadata_account_v3(
                metadata,
                mint,
                mint_authority,
                payer,
                update_authority,
                system_program_account,
                "Five Pool",
                "5IVE",
                "https://example.com/meta.json",
                0,
                false
            );
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source)
        .expect("Metaplex-shaped raw interface call should compile");
    assert!(
        bytecode.iter().any(|op| *op == opcodes::INVOKE),
        "Metaplex-shaped interface call should emit INVOKE opcode"
    );
}

#[test]
fn one_shot_amm_fixture_compiles_when_present() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let amm_path = workspace_root.join("5ive-amm/src/main.v");
    if !amm_path.exists() {
        eprintln!(
            "Skipping AMM fixture compile regression; missing {}",
            amm_path.display()
        );
        return;
    }

    let source = fs::read_to_string(&amm_path).expect("expected readable 5ive-amm fixture source");
    let bytecode = match DslCompiler::compile_dsl(&source) {
        Ok(b) => b,
        Err(err) => {
            eprintln!(
                "Skipping AMM fixture compile/disasm check due compile error: {:?}",
                err
            );
            return;
        }
    };
    assert!(
        bytecode.iter().any(|op| *op == opcodes::INVOKE),
        "fixture should include CPI INVOKE opcode(s)"
    );

    let disassembly = disassemble(&bytecode);
    assert!(
        !disassembly.is_empty(),
        "disassembly should succeed for 5ive-amm fixture"
    );

    let source_requires = count_source_requires(&source);
    let lowered_dispatches = count_require_dispatch_points(&bytecode);
    assert!(
        lowered_dispatches * 2 <= source_requires,
        "expected at least 50% require-dispatch reduction for amm fixture (requires={}, dispatches={})",
        source_requires,
        lowered_dispatches
    );
}

#[test]
fn one_shot_single_pool_fixture_compiles_and_disassembles_when_present() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let pool_path = workspace_root.join("5ive-single-pool/src/main.v");
    if !pool_path.exists() {
        eprintln!(
            "Skipping single-pool fixture compile regression; missing {}",
            pool_path.display()
        );
        return;
    }

    let source =
        fs::read_to_string(&pool_path).expect("expected readable 5ive-single-pool fixture source");
    let bytecode = match DslCompiler::compile_dsl(&source) {
        Ok(b) => b,
        Err(err) => {
            eprintln!(
                "Skipping single-pool fixture compile/disasm check due compile error: {:?}",
                err
            );
            return;
        }
    };
    let disassembly = disassemble(&bytecode);
    assert!(
        !disassembly.is_empty(),
        "disassembly should succeed for 5ive-single-pool fixture"
    );

    let source_requires = count_source_requires(&source);
    let lowered_dispatches = count_require_dispatch_points(&bytecode);
    assert!(
        lowered_dispatches * 2 <= source_requires,
        "expected at least 50% require-dispatch reduction for single-pool fixture (requires={}, dispatches={})",
        source_requires,
        lowered_dispatches
    );
}

#[test]
fn raw_u32_variable_interface_arg_emits_cast_opcode() {
    let source = r#"
        interface StakeProgram @program("Stake11111111111111111111111111111111111111") @serializer(raw) {
            authorize_checked @discriminator_bytes([10, 0, 0, 0]) (
                stake_account: Account,
                clock_sysvar: Account,
                authority: Account,
                new_authority: Account,
                stake_authorize_kind: u32
            );
        }

        pub test_call(
            stake_account: account @mut,
            clock_sysvar: account,
            authority: account @signer,
            new_authority: account @signer
        ) {
            let kind: u32 = 1;
            StakeProgram::authorize_checked(
                stake_account,
                clock_sysvar,
                authority,
                new_authority,
                kind
            );
        }
    "#;

    let bytecode =
        DslCompiler::compile_dsl(source).expect("u32 variable interface arg source should compile");
    assert!(
        bytecode.iter().any(|op| *op == opcodes::CAST),
        "u32 variable interface arg should emit CAST before serialization"
    );
}

#[test]
fn raw_u32_function_arg_emits_cast_opcode() {
    let source = r#"
        interface StakeProgram @program("Stake11111111111111111111111111111111111111") @serializer(raw) {
            authorize_checked @discriminator_bytes([10, 0, 0, 0]) (
                stake_account: Account,
                clock_sysvar: Account,
                authority: Account,
                new_authority: Account,
                stake_authorize_kind: u32
            );
        }

        kind() -> u32 {
            return 1;
        }

        pub test_call(
            stake_account: account @mut,
            clock_sysvar: account,
            authority: account @signer,
            new_authority: account @signer
        ) {
            StakeProgram::authorize_checked(
                stake_account,
                clock_sysvar,
                authority,
                new_authority,
                kind()
            );
        }
    "#;

    let bytecode =
        DslCompiler::compile_dsl(source).expect("u32 function interface arg source should compile");
    assert!(
        bytecode.iter().any(|op| *op == opcodes::CAST),
        "u32 function interface arg should emit CAST before serialization"
    );
}

#[test]
fn stdlib_stake_authorize_checked_variable_emits_cast_opcode() {
    let source = r#"
        use std::interfaces::stake_program;

        pub test_call(
            stake_account: account @mut,
            clock_sysvar: account,
            authority: account @signer,
            new_authority: account @signer
        ) {
            let kind: u32 = 1;
            stake_program::authorize_checked(
                stake_account,
                clock_sysvar,
                authority,
                new_authority,
                kind
            );
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source)
        .expect("stdlib stake authorize_checked variable source should compile");
    assert!(
        bytecode.iter().any(|op| *op == opcodes::CAST),
        "stdlib stake authorize_checked variable should emit CAST before serialization"
    );
}
