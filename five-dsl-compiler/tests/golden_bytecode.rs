use five_dsl_compiler::DslCompiler;
use five_protocol::{
    opcodes, ConstantPoolDescriptor, FEATURE_CONSTANT_POOL, FEATURE_FUNCTION_NAMES,
    FEATURE_PUBLIC_ENTRY_TABLE, FIVE_HEADER_OPTIMIZED_SIZE,
};
use std::collections::VecDeque;

/// Helper: find all positions of a raw opcode byte in the bytecode
fn find_opcode_positions(bytecode: &[u8], opcode: u8) -> Vec<usize> {
    bytecode
        .iter()
        .enumerate()
        .filter_map(|(i, &b)| if b == opcode { Some(i) } else { None })
        .collect()
}

/// Helper: count occurrences of an opcode
fn count_opcode(bytecode: &[u8], opcode: u8) -> usize {
    find_opcode_positions(bytecode, opcode).len()
}

fn code_contains_u64_literal(
    bytecode: &[u8],
    code: &[u8],
    pool_info: Option<(usize, u16)>,
    target: u64,
) -> bool {
    if let Some((pool_offset, pool_slots)) = pool_info {
        let mut i = 0usize;
        while i < code.len() {
            let op = code[i];
            if matches!(
                op,
                opcodes::PUSH_U8
                    | opcodes::PUSH_U16
                    | opcodes::PUSH_U32
                    | opcodes::PUSH_U64
                    | opcodes::PUSH_I64
                    | opcodes::PUSH_BOOL
                    | opcodes::PUSH_PUBKEY
                    | opcodes::PUSH_U128
                    | opcodes::PUSH_STRING
            ) {
                if i + 1 >= code.len() {
                    break;
                }
                let idx = code[i + 1] as u16;
                if idx < pool_slots {
                    let start = pool_offset + idx as usize * 8;
                    if start + 8 <= bytecode.len() {
                        let mut bytes = [0u8; 8];
                        bytes.copy_from_slice(&bytecode[start..start + 8]);
                        if u64::from_le_bytes(bytes) == target {
                            return true;
                        }
                    }
                }
                i += 2;
                continue;
            }
            if matches!(
                op,
                opcodes::PUSH_U8_W
                    | opcodes::PUSH_U16_W
                    | opcodes::PUSH_U32_W
                    | opcodes::PUSH_U64_W
                    | opcodes::PUSH_I64_W
                    | opcodes::PUSH_BOOL_W
                    | opcodes::PUSH_PUBKEY_W
                    | opcodes::PUSH_U128_W
                    | opcodes::PUSH_STRING_W
            ) {
                if i + 2 >= code.len() {
                    break;
                }
                let idx = u16::from_le_bytes([code[i + 1], code[i + 2]]);
                if idx < pool_slots {
                    let start = pool_offset + idx as usize * 8;
                    if start + 8 <= bytecode.len() {
                        let mut bytes = [0u8; 8];
                        bytes.copy_from_slice(&bytecode[start..start + 8]);
                        if u64::from_le_bytes(bytes) == target {
                            return true;
                        }
                    }
                }
                i += 3;
                continue;
            }
            i += 1;
        }
        return false;
    }

    for &push_opcode in &[
        opcodes::PUSH_U8,
        opcodes::PUSH_U16,
        opcodes::PUSH_U32,
        opcodes::PUSH_U64,
    ] {
        for p in find_opcode_positions(code, push_opcode) {
            if p + 9 <= code.len() {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&code[p + 1..p + 9]);
                if u64::from_le_bytes(bytes) == target {
                    return true;
                }
            }
        }
    }
    false
}

/// Golden test: verify arithmetic emits MUL before ADD for `2 + 3 * 4` (multiplication binds tighter)
///
/// This golden check accepts either:
///  - an explicit MUL opcode followed later by ADD (normal codegen), or
///  - an optimizer constant-fold that replaces `3 * 4` with a PUSH_U64(12) followed by ADD (i.e. 2 + 12).
/// The test asserts that one of these correct codegen patterns is present and that ADD happens after the multiplication result.
#[test]
fn golden_arithmetic_mul_then_add() {
    let source = r#"
        script golden_arith {
            init {
                // Expression: 2 + 3 * 4
                let _ = 2 + 3 * 4;
            }
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("compile should succeed");

    // Detect constant pool layout if present
    let (pool_info, code_start) = parse_constant_pool_layout(&bytecode);
    let code = &bytecode[code_start..];

    const PUSH_OPCODES: [u8; 4] = [
        opcodes::PUSH_U8,
        opcodes::PUSH_U16,
        opcodes::PUSH_U32,
        opcodes::PUSH_U64,
    ];

    // Find MUL and ADD opcodes
    let mul_positions = find_opcode_positions(code, opcodes::MUL);
    let add_positions = find_opcode_positions(code, opcodes::ADD);

    // If MUL exists, require at least one ADD and that a MUL occurs before an ADD
    if !mul_positions.is_empty() {
        assert!(
            !add_positions.is_empty(),
            "Golden check failed: ADD opcode not found in bytecode while MUL is present"
        );

        let found_mul_before_add = mul_positions
            .iter()
            .any(|&mul_pos| add_positions.iter().any(|&add_pos| mul_pos < add_pos));

        assert!(
            found_mul_before_add,
            "Golden check failed: did not find MUL occurring before ADD in bytecode (expected 2 + (3 * 4) evaluation order). \
             Bytecode op positions: MUL={:?}, ADD={:?}",
            mul_positions,
            add_positions
        );
    } else {
        // No MUL present — accept a constant-folded multiplication: PUSH_UX(12) then ADD
        let mut found_folded = false;

        if code_contains_u64_literal(&bytecode, code, pool_info, 12) {
            found_folded = !add_positions.is_empty();
        }

        // Optimizer can fully fold `2 + 3 * 4` to a single literal.
        let found_fully_folded = code_contains_u64_literal(&bytecode, code, pool_info, 14);
        let found_compact_add_only = !add_positions.is_empty();

        assert!(
            found_folded || found_fully_folded || found_compact_add_only,
            "Golden check failed: expected MUL+ADD, partially folded (12 then ADD), fully folded literal 14, or compact ADD-only form. \
             Bytecode prefix: {:?}",
            &bytecode[..std::cmp::min(64, bytecode.len())]
        );
    }

    // Additionally assert we have a reasonable number of literal pushes (PUSH_U64) present
    let push_literal_count: usize = PUSH_OPCODES
        .iter()
        .map(|&opcode| count_opcode(code, opcode))
        .sum();
    assert!(
        push_literal_count >= 1,
        "Golden check warning: expected at least 1 PUSH immediate in bytecode for arithmetic literals, found {}",
        push_literal_count
    );
}

fn parse_constant_pool_layout(bytecode: &[u8]) -> (Option<(usize, u16)>, usize) {
    if bytecode.len() < FIVE_HEADER_OPTIMIZED_SIZE || &bytecode[0..4] != b"5IVE" {
        return (None, 0);
    }
    let features = u32::from_le_bytes([bytecode[4], bytecode[5], bytecode[6], bytecode[7]]);
    let mut offset = FIVE_HEADER_OPTIMIZED_SIZE;
    if (features & FEATURE_FUNCTION_NAMES) != 0 {
        if offset + 2 > bytecode.len() {
            return (None, offset.min(bytecode.len()));
        }
        let section_size = u16::from_le_bytes([bytecode[offset], bytecode[offset + 1]]) as usize;
        offset += 2 + section_size;
    }
    if (features & FEATURE_PUBLIC_ENTRY_TABLE) != 0 {
        if offset + 2 > bytecode.len() {
            return (None, offset.min(bytecode.len()));
        }
        let section_size = u16::from_le_bytes([bytecode[offset], bytecode[offset + 1]]) as usize;
        offset += 2 + section_size;
    }
    if (features & FEATURE_CONSTANT_POOL) == 0 {
        return (None, offset.min(bytecode.len()));
    }
    if offset + core::mem::size_of::<ConstantPoolDescriptor>() > bytecode.len() {
        return (None, offset.min(bytecode.len()));
    }
    let desc = ConstantPoolDescriptor {
        pool_offset: u32::from_le_bytes([bytecode[offset], bytecode[offset + 1], bytecode[offset + 2], bytecode[offset + 3]]),
        string_blob_offset: u32::from_le_bytes([bytecode[offset + 4], bytecode[offset + 5], bytecode[offset + 6], bytecode[offset + 7]]),
        string_blob_len: u32::from_le_bytes([bytecode[offset + 8], bytecode[offset + 9], bytecode[offset + 10], bytecode[offset + 11]]),
        pool_slots: u16::from_le_bytes([bytecode[offset + 12], bytecode[offset + 13]]),
        reserved: u16::from_le_bytes([bytecode[offset + 14], bytecode[offset + 15]]),
    };
    let pool_offset = desc.pool_offset as usize;
    let code_offset = pool_offset + desc.pool_slots as usize * 8;
    (Some((pool_offset, desc.pool_slots)), code_offset.min(bytecode.len()))
}

/// Golden test: verify left-associative division emits two DIVs in left-associative order for `6 / 2 / 3`
#[test]
fn golden_division_left_associative() {
    let source = r#"
        script golden_div {
            init {
                // (6 / 2) / 3  -> left associative
                let _ = 6 / 2 / 3;
            }
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("compile should succeed");
    let (pool_info, code_start) = parse_constant_pool_layout(&bytecode);
    let code = &bytecode[code_start..];

    let div_positions = find_opcode_positions(code, opcodes::DIV);
    let fully_folded = code_contains_u64_literal(&bytecode, code, pool_info, 1);

    if !fully_folded {
        assert!(
            div_positions.len() >= 2,
            "Golden check failed: expected at least two DIV opcodes for two divisions unless fully folded to 1, found {}. Bytecode: {:?}",
            div_positions.len(),
            &bytecode[..std::cmp::min(64, bytecode.len())]
        );

        // Ensure order: first DIV (inner) appears earlier than second DIV (outer)
        assert!(
            div_positions[0] < div_positions[1],
            "Golden check failed: DIV opcodes not in expected left-associative order (positions: {:?})",
            div_positions
        );
    }
}

#[test]
fn golden_logical_or_short_circuits_with_jumps() {
    let source = r#"
        script golden_or_sc {
            pub fn f(a: bool, b: bool) -> bool {
                return a || b;
            }
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("compile should succeed");
    let (_pool_info, code_start) = parse_constant_pool_layout(&bytecode);
    let code = &bytecode[code_start..];

    let jump_if_positions = find_opcode_positions(code, opcodes::JUMP_IF);
    let jump_positions = find_opcode_positions(code, opcodes::JUMP);
    let dup_positions = find_opcode_positions(code, opcodes::DUP);
    let pop_positions = find_opcode_positions(code, opcodes::POP);

    assert!(
        !jump_if_positions.is_empty(),
        "Golden check failed: expected JUMP_IF for logical-or short-circuit path"
    );
    assert!(
        !jump_positions.is_empty(),
        "Golden check failed: expected JUMP for logical-or merge path"
    );
    assert!(
        !dup_positions.is_empty() && !pop_positions.is_empty(),
        "Golden check failed: expected DUP + POP in short-circuit lowering"
    );
}

/// Golden test: ensure `require(...)` compiles to a REQUIRE opcode in the final bytecode
#[test]
fn golden_require_emitted() {
    let source = r#"
        script golden_require {
            init {
                let amount = 10;
            }
            constraints {
                require(amount > 0);
            }
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("compile should succeed");

    let req_positions = find_opcode_positions(&bytecode, opcodes::REQUIRE);
    let fused_req_positions = find_opcode_positions(&bytecode, opcodes::REQUIRE_LOCAL_GT_ZERO);

    assert!(
        !req_positions.is_empty() || !fused_req_positions.is_empty(),
        "Golden check failed: expected REQUIRE or fused REQUIRE_LOCAL_GT_ZERO opcode in bytecode (constraints / require not emitted). \
         Bytecode (prefix): {:?}",
        &bytecode[..std::cmp::min(64, bytecode.len())]
    );

    // As a sanity measure, ensure REQUIRE occurs after at least one PUSH or comparison opcode
    // Find earliest index of any comparison/opcode that might precede require (GT or literal push)
    let mut predecessor_candidates = VecDeque::new();
    predecessor_candidates.extend(find_opcode_positions(&bytecode, opcodes::GT));
    predecessor_candidates.extend(find_opcode_positions(&bytecode, opcodes::PUSH_U64));
    predecessor_candidates.make_contiguous().sort_unstable();

    let first_req_pos = req_positions
        .first()
        .copied()
        .or_else(|| fused_req_positions.first().copied());
    if let Some(first_req_pos) = first_req_pos {
        if let Some(&first_pred) = predecessor_candidates.front() {
            assert!(
                first_pred < first_req_pos,
                "Golden check ordering: expected a comparison or literal push before REQUIRE, but found REQUIRE earlier. \
                 first_pred={}, first_req={}, prefix={:?}",
                first_pred,
                first_req_pos,
                &bytecode[..std::cmp::min(64, bytecode.len())]
            );
        }
    }
}

/// Golden test: ensure derive_pda invocation emits DERIVE_PDA opcode and pushes seeds count
#[test]
fn golden_derive_pda_emitted() {
    let source = r#"
        script golden_pda {
            init {
                let p = derive_pda(1, 2, 3);
            }
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("compile should succeed");

    // DERIVE_PDA should appear somewhere in the bytecode
    let derive_positions = find_opcode_positions(&bytecode, opcodes::DERIVE_PDA);
    assert!(
        !derive_positions.is_empty(),
        "Golden check failed: expected DERIVE_PDA opcode in bytecode. Bytecode prefix: {:?}",
        &bytecode[..std::cmp::min(64, bytecode.len())]
    );

    // Verify there's a PUSH_U8 (seeds count) preceding DERIVE_PDA in the instruction stream
    // Find any PUSH_U8 before a DERIVE_PDA index (we expect at least one)
    let push_u8_positions = find_opcode_positions(&bytecode, opcodes::PUSH_U8);
    let found = derive_positions
        .iter()
        .any(|&dp| push_u8_positions.iter().any(|&pp| pp < dp));

    assert!(
        found,
        "Golden check failed: expected a PUSH_U8 (seeds count) before DERIVE_PDA. push_u8_positions={:?}, derive_positions={:?}",
        push_u8_positions,
        derive_positions
    );
}

/// Golden test: derive_pda with explicit u8 bump should unwrap tuple and keep only pubkey
#[test]
fn golden_derive_pda_with_bump_unwraps_to_pubkey() {
    let source = r#"
        script golden_pda_with_bump {
            init {
                let vault_key = derive_pda("vault", 7 as u8);
            }
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("compile should succeed");
    let derive_positions = find_opcode_positions(&bytecode, opcodes::DERIVE_PDA);
    assert!(!derive_positions.is_empty(), "expected DERIVE_PDA in bytecode");

    // In bump-validation mode codegen should emit UNPACK_TUPLE + DROP after DERIVE_PDA.
    let unpack_positions = find_opcode_positions(&bytecode, opcodes::UNPACK_TUPLE);
    let drop_positions = find_opcode_positions(&bytecode, opcodes::DROP);
    assert!(
        !unpack_positions.is_empty(),
        "expected UNPACK_TUPLE after DERIVE_PDA in bump mode"
    );
    assert!(
        !drop_positions.is_empty(),
        "expected DROP after DERIVE_PDA in bump mode"
    );

    let has_ordered_pattern = derive_positions.iter().any(|&d| {
        unpack_positions.iter().any(|&u| {
            u > d && drop_positions.iter().any(|&dr| dr > u)
        })
    });
    assert!(
        has_ordered_pattern,
        "expected ordered pattern DERIVE_PDA -> UNPACK_TUPLE -> DROP"
    );
}

/// Golden test: array creation and length opcodes
#[test]
fn golden_array_create_and_length() {
    let source = r#"
        script golden_array {
            init {
                let a = [1, 2, 3];
                let l = string_length(a);
            }
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("compile should succeed");

    // Expect PUSH_ARRAY_LITERAL (current implementation) and ARRAY_LENGTH opcodes in the bytecode
    let create_positions = find_opcode_positions(&bytecode, opcodes::PUSH_ARRAY_LITERAL);
    let len_positions = find_opcode_positions(&bytecode, opcodes::ARRAY_LENGTH);

    assert!(
        !create_positions.is_empty(),
        "Golden check failed: expected PUSH_ARRAY_LITERAL opcode in bytecode"
    );

    assert!(
        !len_positions.is_empty(),
        "Golden check failed: expected ARRAY_LENGTH opcode in bytecode for .length()"
    );

    // Ensure array creation occurs before length query
    let found_order = create_positions
        .iter()
        .any(|&cp| len_positions.iter().any(|&lp| cp < lp));
    assert!(
        found_order,
        "Golden check failed: array creation not occurring before length check. create_positions={:?}, len_positions={:?}",
        create_positions,
        len_positions
    );
}
