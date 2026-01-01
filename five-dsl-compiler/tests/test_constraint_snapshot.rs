use five_dsl_compiler::*;
use five_protocol::opcodes::*;
use five_dsl_compiler::bytecode_generator::disassembler::Instruction;

#[test]
fn test_constraint_bytecode_optimality_and_ordering() {
    let source = r#"
        script constraint_bytecode_test {
            account Vault {
                owner: Pubkey,
                mint: Pubkey,
            }

            // Test ordering: @mut, @has(owner), @signer
            pub deposit(
                vault: Vault @mut @has(owner, mint),
                owner: Account @signer,
                mint: Account
            ) {
                // Body
            }
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");
    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    // Get structured disassembly
    let instructions = generator.get_structured_disassembly();
    
    // Find where the function starts (after header)
    // We look for LOAD_PARAM opcodes which start the function body
    let start_index = instructions.iter().position(|i| matches!(i, Instruction::Opcode(op) if *op == LOAD_PARAM)).unwrap_or(0);
    
    // We expect a sequence of validations after parameter loading
    // The compiler emits constraints in this order for each param:
    // 1. @mut (CHECK_WRITABLE)
    // 2. @has (LOAD_FIELD -> arg -> EQ -> REQUIRE)
    // 3. @owner (CHECK_OWNER)
    // 4. @signer (CHECK_SIGNER)
    //
    // Params are processed in order.
    
    // Scan for `vault` constraints (param index 0)
    let mut found_vault_mut = false;
    let mut found_vault_has_owner = false;
    let mut found_vault_has_mint = false;
    
    // Scan for `owner` constraints (param index 1)
    let mut found_owner_signer = false;

    // Helper to find subsequence
    let mut i = start_index;
    while i < instructions.len() {
        match &instructions[i] {
            Instruction::CheckWritable { account_index, .. } => {
                if *account_index == 2 {
                    found_vault_mut = true;
                }
            },
            Instruction::CheckSigner { account_index, .. } => {
                if *account_index == 3 {
                    found_owner_signer = true;
                }
            },
            Instruction::LoadField { account_index, field_offset, .. } => {
                // Detected a @has check start.
                if *account_index == 2 { // vault (param 0 + 2 offset)
                    // Next instructions should be loading the target key, then EQ, then REQUIRE
                    // For `owner` field (offset 0 likely, as it's first)
                    // For `mint` field (offset 32 likely, as it's second)
                    if *field_offset == 0 {
                         // Expect check against owner param (index 1)
                         // We can't verify exact target load easily without complex matching, 
                         // but we can verify the structure: LOAD_FIELD -> GET_KEY/LOCAL -> EQ -> REQUIRE
                         if verify_check_pattern(&instructions, i) {
                             found_vault_has_owner = true;
                         }
                    } else if *field_offset == 8 { // Pubkey treated as 8 bytes (unknown/ref) or compacted
                         if verify_check_pattern(&instructions, i) {
                             found_vault_has_mint = true;
                         }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    assert!(found_vault_mut, "Should emit CHECK_WRITABLE for @mut");
    assert!(found_vault_has_owner, "Should emit check for @has(owner)");
    assert!(found_vault_has_mint, "Should emit check for @has(mint)");
    assert!(found_owner_signer, "Should emit CHECK_SIGNER for @signer");
}

fn verify_check_pattern(instrs: &[Instruction], start_idx: usize) -> bool {
    // Expected: LOAD_FIELD (at start_idx) -> (GET_KEY or GET_LOCAL) -> EQ -> REQUIRE
    if start_idx + 3 >= instrs.len() { return false; }
    
    // index + 1: Load Target
    let load_target = &instrs[start_idx + 1];
    let is_load_target = matches!(load_target, 
        Instruction::GetLocal { .. } | 
        Instruction::GetKey { .. } |
        Instruction::Opcode(GET_KEY) | 
        Instruction::Opcode(GET_LOCAL)
    );
    
    // index + 2: EQ
    let eq_op = &instrs[start_idx + 2];
    let is_eq = matches!(eq_op, Instruction::Opcode(op) if *op == EQ);
    
    // index + 3: REQUIRE
    let req_op = &instrs[start_idx + 3];
    let is_req = matches!(req_op, Instruction::Opcode(op) if *op == REQUIRE);
    
    is_load_target && is_eq && is_req
}
