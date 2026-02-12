# 5IVE Frontend Claims Matrix

This matrix maps homepage/docs claims to source-of-truth references in the codebase.

## Legend

- `Verified`: directly supported by compiler/runtime tests or implementation.
- `Needs qualifier`: directionally true but should not be presented as an absolute number/guarantee.
- `Removed`: removed from copy due weak or stale evidence.

## Claims

1. `External bytecode calls are non-CPI and use CALL_EXTERNAL.`
- Status: `Verified`
- Evidence:
  - `five-dsl-compiler/src/bytecode_generator/ast_generator/functions.rs`
  - `five-dsl-compiler/tests/lib.rs` (`test_external_imported_items_allow_unqualified_call`)
  - `five-solana/tests/runtime_script_fixture_tests.rs`

2. `Interfaces are used for CPI-style integration with non-Five programs.`
- Status: `Verified`
- Evidence:
  - `five-dsl-compiler/tests/lib.rs` (`test_cpi_interface_spl_token_mint_to`)
  - `five-dsl-compiler/src/interface_registry.rs`

3. `Import verification prevents bytecode account substitution.`
- Status: `Verified`
- Evidence:
  - `five-dsl-compiler/tests/lib.rs` (import verification tests)
  - `five-vm-mito/tests/call_external_verification_tests.rs`

4. `External calls are always zero-cost.`
- Status: `Removed`
- Reason:
  - External calls are cheaper than CPI in many harness cases, but still incur runtime cost.

5. `Fixed economic promise ($1 vs $1000).`
- Status: `Removed`
- Reason:
  - Environment-dependent and not a stable protocol guarantee.

6. `Five can reduce deploy/logic bloat through composable bytecode accounts.`
- Status: `Needs qualifier`
- Evidence:
  - Template/runtime patterns and external call support.
- Qualification:
  - Keep this as a directional capability statement, not a fixed numeric multiplier.
