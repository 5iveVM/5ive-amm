use five_dsl_compiler::DslCompiler;
use five_protocol::opcodes;
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

    const PUSH_OPCODES: [u8; 4] = [
        opcodes::PUSH_U8,
        opcodes::PUSH_U16,
        opcodes::PUSH_U32,
        opcodes::PUSH_U64,
    ];

    // Find MUL and ADD opcodes
    let mul_positions = find_opcode_positions(&bytecode, opcodes::MUL);
    let add_positions = find_opcode_positions(&bytecode, opcodes::ADD);

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
        const PUSH_OPCODES: [u8; 4] = [
            opcodes::PUSH_U8,
            opcodes::PUSH_U16,
            opcodes::PUSH_U32,
            opcodes::PUSH_U64,
        ];
        let mut found_folded = false;

        for &push_opcode in &PUSH_OPCODES {
            let push_positions = find_opcode_positions(&bytecode, push_opcode);
            for &p in &push_positions {
                // Decode the fixed-size immediate after PUSH_U64 opcode
                if p + 9 <= bytecode.len() {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&bytecode[p + 1..p + 9]);
                    let val = u64::from_le_bytes(bytes);
                    if val == 12 {
                        // Check there is an ADD after this push
                        let add_after = add_positions.iter().any(|&ap| ap > p);
                        if add_after {
                            found_folded = true;
                            break;
                        }
                    }
                }
            }
            if found_folded {
                break;
            }
        }

        assert!(
            found_folded,
            "Golden check failed: neither explicit MUL opcode present nor folded PUSH_U64(12) followed by ADD found. \
             Bytecode prefix: {:?}",
            &bytecode[..std::cmp::min(64, bytecode.len())]
        );
    }

    // Additionally assert we have a reasonable number of literal pushes (PUSH_U64) present
    let push_literal_count: usize = PUSH_OPCODES
        .iter()
        .map(|&opcode| count_opcode(&bytecode, opcode))
        .sum();
    assert!(
        push_literal_count >= 1,
        "Golden check warning: expected at least 1 PUSH immediate in bytecode for arithmetic literals, found {}",
        push_literal_count
    );
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

    let div_positions = find_opcode_positions(&bytecode, opcodes::DIV);

    assert!(
        div_positions.len() >= 2,
        "Golden check failed: expected at least two DIV opcodes for two divisions, found {}. Bytecode: {:?}",
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

    assert!(
        !req_positions.is_empty(),
        "Golden check failed: expected REQUIRE opcode to be present in bytecode (constraints / require not emitted). \
         Bytecode (prefix): {:?}",
        &bytecode[..std::cmp::min(64, bytecode.len())]
    );

    // As a sanity measure, ensure REQUIRE occurs after at least one PUSH or comparison opcode
    // Find earliest index of any comparison/opcode that might precede require (GT or literal push)
    let mut predecessor_candidates = VecDeque::new();
    predecessor_candidates.extend(find_opcode_positions(&bytecode, opcodes::GT));
    predecessor_candidates.extend(find_opcode_positions(&bytecode, opcodes::PUSH_U64));
    predecessor_candidates.make_contiguous().sort_unstable();

    if let Some(first_req_pos) = req_positions.first().copied() {
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
