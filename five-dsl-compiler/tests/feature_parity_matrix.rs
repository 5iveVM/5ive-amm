use five_dsl_compiler::compiler::DslCompiler;

struct MatrixRow {
    category: &'static str,
    positive: &'static [&'static str],
    negative: &'static [&'static str],
}

#[test]
fn matrix_rows_have_positive_and_negative_assertions() {
    let rows = vec![
        MatrixRow {
            category: "01-language-basics",
            positive: &["test_compiler_features::test_simple_state_update"],
            negative: &["test_dsl_compiler_invalid_syntax"],
        },
        MatrixRow {
            category: "03-control-flow",
            positive: &["test_compiler_features::test_if_statement_compilation"],
            negative: &["test_dsl_compiler_type_error"],
        },
        MatrixRow {
            category: "04-account-system",
            positive: &["test_constraints::test_constraints_signer_valid"],
            negative: &["test_constraints::test_constraints_signer_invalid_type"],
        },
        MatrixRow {
            category: "05-blockchain-integration",
            positive: &["cpi_compile_regression_tests::test_invoke_standard"],
            negative: &["lib::test_cpi_parameter_count_validation"],
        },
        MatrixRow {
            category: "08-match-expressions",
            positive: &["lib::test_match_expression_parsing"],
            negative: &["lib::test_infer_type_invalid_enum_variant"],
        },
    ];

    for row in rows {
        assert!(
            !row.positive.is_empty(),
            "category {} must include at least one positive assertion",
            row.category
        );
        assert!(
            !row.negative.is_empty(),
            "category {} must include at least one negative assertion",
            row.category
        );
    }
}

#[test]
fn cast_preserves_target_type_for_field_access() {
    let dsl = r#"
account MyAccount {
    balance: u64,
}

pub test(acc: Account @mut) {
    let mut x = acc as MyAccount;
    let amount = x.balance;
    x.balance = amount;
}

pub main() {}
"#;

    let result = DslCompiler::compile_dsl(dsl);
    assert!(
        result.is_ok(),
        "cast target type should be preserved in field access"
    );
}

#[test]
fn cast_does_not_bypass_mutability_constraints() {
    let dsl = r#"
account MyAccount {
    balance: u64,
}

pub test(acc: Account) {
    let x = acc as MyAccount;
    x.balance = 100;
}

pub main() {}
"#;

    let result = DslCompiler::compile_dsl(dsl);
    assert!(result.is_err(), "cast must not bypass @mut constraints");
}

#[test]
fn reserved_keyword_function_name_has_targeted_diagnostic() {
    let dsl = r#"
pub init() {}
pub main() {}
"#;
    let err = DslCompiler::compile_dsl(dsl).expect_err("reserved keyword should fail");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("reserved keyword"),
        "expected targeted reserved-keyword diagnostic, got: {msg}"
    );
    assert!(
        msg.contains("'init'"),
        "expected offending keyword in diagnostic: {msg}"
    );
}
