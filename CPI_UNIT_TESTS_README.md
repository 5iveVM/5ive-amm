# CPI Unit Tests - Comprehensive Testing Guide

This document describes the comprehensive Rust unit tests for CPI (Cross-Program Invocation) functionality that run with `cargo test --workspace`.

## Overview

The CPI unit tests provide **full coverage of CPI functionality without on-chain execution**. They test:

- ✅ Interface declaration parsing
- ✅ CPI bytecode generation
- ✅ Instruction serialization (Borsh/Bincode)
- ✅ Parameter encoding (u8, u16, u32, u64, bool, pubkey, string)
- ✅ Account parameter handling
- ✅ Discriminator encoding (single-byte and 8-byte)
- ✅ INVOKE vs INVOKE_SIGNED opcodes
- ✅ Error handling and edge cases
- ✅ End-to-end CPI scenarios

## Test Organization

### Compiler Tests
**File:** `five-dsl-compiler/tests/cpi_unit_tests.rs`

Tests the Five DSL compiler's CPI code generation:

#### Compilation Tests (19 tests)
- `test_interface_declaration_parsing` - Basic interface syntax
- `test_spl_token_mint_interface` - SPL Token use case
- `test_anchor_8byte_discriminator` - Anchor programs with 8-byte sighash
- `test_multiple_interfaces` - Multiple interface methods
- `test_pure_data_cpi_call` - Data-only CPI (no accounts)
- `test_interface_with_bincode_serializer` - Bincode format support
- `test_interface_missing_program_id_error` - Error: missing @program
- `test_interface_missing_discriminator_error` - Error: missing @discriminator
- `test_interface_parameter_count_validation` - Error: parameter mismatch
- `test_data_argument_must_be_literal` - MVP limitation verification
- `test_account_and_data_parameters_mixed` - Account/data interleaving
- `test_invoke_signed_with_pda` - PDA authority pattern
- `test_global_state_with_cpi` - State tracking with CPI
- `test_string_parameter_in_cpi` - String data type
- `test_bool_parameter_in_cpi` - Boolean data type
- `test_all_integer_types_in_cpi` - All numeric types
- `test_interface_import_pattern` - Multiple calls to same interface
- `test_nested_function_calls_with_cpi` - Functions calling functions with CPI
- `test_cpi_with_conditional` - CPI inside if statements

#### Serialization Tests (11 tests)
- `test_borsh_u64_serialization` - Verify u64 encoding
- `test_borsh_u32_serialization` - Verify u32 encoding
- `test_borsh_u16_serialization` - Verify u16 encoding
- `test_borsh_u8_serialization` - Verify u8 encoding
- `test_borsh_pubkey_serialization` - Verify pubkey encoding
- `test_borsh_bool_serialization` - Verify bool encoding
- `test_discriminator_u8_encoding` - Single-byte discriminator
- `test_discriminator_8byte_encoding` - 8-byte discriminator
- `test_bincode_u64_serialization` - Bincode format
- `test_spl_token_mint_to_serialization` - Full instruction format
- **Total: 30 tests**

### VM Tests
**File:** `five-vm-mito/tests/cpi_unit_tests.rs`

Tests the Five VM's CPI opcode execution:

#### INVOKE Tests (11 tests)
- `test_invoke_opcode_parsing` - Verify opcode values (0x80, 0x81)
- `test_basic_invoke_execution` - Basic INVOKE flow
- `test_invoke_signed_pda_execution` - INVOKE_SIGNED for PDA
- `test_account_parameter_handling` - Account indexing (0-15)
- `test_data_parameter_stacking` - Stack operations for data
- `test_invoke_with_mixed_account_and_data_parameters` - Interleaved params
- `test_invoke_with_maximum_accounts` - Maximum 16 accounts
- `test_invoke_exceeding_maximum_accounts_error` - Constraint validation
- `test_invoke_instruction_data_limit` - 32-byte limit
- `test_invoke_with_pubkey_parameters` - Pubkey handling
- `test_account_index_validation` - Index range validation

#### Serialization Format Tests (8 tests)
- `test_vle_encoding_small_u64` - VLE for small values
- `test_vle_encoding_large_u64` - VLE for large values
- `test_account_parameter_order_preservation` - Order invariance
- `test_data_parameter_encoding_u64` - Little-endian u64
- `test_data_parameter_encoding_u32` - Little-endian u32
- `test_instruction_data_assembly` - Full instruction layout
- `test_discriminator_single_byte` - 1-byte discriminator size
- `test_discriminator_8bytes` - 8-byte discriminator size

#### Error Handling Tests (8 tests)
- `test_unknown_interface_error` - Undeclared interface
- `test_parameter_count_mismatch_error` - Wrong arg count
- `test_program_id_mismatch_at_runtime` - Program ID verification
- `test_account_constraint_validation` - @signer, @mut validation
- `test_data_type_mismatch_error` - Type checking
- `test_insufficient_stack_space` - Stack underflow
- `test_invalid_account_index` - Out-of-range account
- `test_instruction_data_overflow` - >32 bytes data

#### Interface Verification Tests (6 tests)
- `test_interface_storage_in_stack_contract` - Stack contract storage
- `test_program_id_verification` - Program ID matching
- `test_discriminator_verification` - Discriminator matching
- `test_parameter_count_verification` - Parameter count
- `test_serialization_format_verification` - Format selection
- `test_import_verification_prevents_substitution` - Security validation

#### Integration Scenario Tests (4 tests)
- `test_spl_token_mint_flow` - End-to-end SPL Token mint
- `test_anchor_program_call_flow` - End-to-end Anchor call
- `test_pda_authority_flow` - End-to-end PDA authority
- `test_multi_step_cpi_sequence` - Multiple CPI calls

**Total: 37 tests**

### Summary

- **Compiler Tests:** 30 tests for DSL parsing and code generation
- **VM Tests:** 37 tests for opcode execution and serialization
- **Total:** 67 comprehensive CPI unit tests

## Running the Tests

### Run all CPI tests
```bash
# Run all tests in the workspace
cargo test --workspace

# Run only CPI tests
cargo test --workspace cpi

# Run with output
cargo test --workspace cpi -- --nocapture
```

### Run specific test suites
```bash
# Compiler CPI compilation tests
cargo test -p five-dsl-compiler cpi_compilation_tests

# Compiler CPI serialization tests
cargo test -p five-dsl-compiler cpi_serialization_tests

# VM CPI invoke tests
cargo test -p five-vm-mito cpi_invoke_tests

# VM CPI serialization tests
cargo test -p five-vm-mito cpi_serialization_format_tests

# VM CPI error handling
cargo test -p five-vm-mito cpi_error_handling_tests

# VM interface verification
cargo test -p five-vm-mito cpi_interface_verification_tests

# VM integration scenarios
cargo test -p five-vm-mito cpi_integration_scenario_tests
```

### Run single test
```bash
cargo test --workspace test_spl_token_mint_interface
```

### Run tests with backtrace
```bash
RUST_BACKTRACE=1 cargo test --workspace cpi
```

### Run tests with verbose output
```bash
cargo test --workspace cpi -- --nocapture --test-threads=1
```

## Test Coverage

### Compiler Coverage

**Parsing & Validation:**
- ✅ Valid interface declarations
- ✅ Multiple interfaces per contract
- ✅ Both Borsh and Bincode serializers
- ✅ Single-byte and 8-byte discriminators
- ✅ All data types (u8-u64, bool, pubkey, string)
- ✅ Mixed account/data parameters
- ✅ Pure data calls (no accounts)
- ✅ Error conditions (missing fields, mismatches)

**Integration:**
- ✅ Global state with CPI
- ✅ Nested function calls with CPI
- ✅ Conditional CPI execution
- ✅ Multiple CPI calls in sequence

### VM Coverage

**Opcode Execution:**
- ✅ INVOKE (0x80)
- ✅ INVOKE_SIGNED (0x81)
- ✅ Account parameter stacking
- ✅ Data parameter encoding
- ✅ Stack operations

**Constraints:**
- ✅ Maximum 16 accounts per CPI
- ✅ Maximum 32 bytes instruction data
- ✅ Discriminator size variations

**Safety:**
- ✅ Stack space verification
- ✅ Account index validation
- ✅ Instruction data size limits
- ✅ Parameter type matching

## Key Test Scenarios

### 1. SPL Token Mint
Tests the full flow of minting tokens via CPI:
- Interface: 3 pubkey accounts + 1 u64 data
- Discriminator: single byte (7)
- Serialization: Borsh (default)

### 2. Anchor Programs
Tests calling Anchor programs with 8-byte sighash:
- Discriminator: 8 bytes
- Serialization: Borsh
- Mixed account/data parameters

### 3. PDA Authority
Tests INVOKE_SIGNED with Program Derived Address:
- PDA derivation from seeds
- Authority delegation
- INVOKE_SIGNED opcode

### 4. Edge Cases
Tests boundary conditions:
- Maximum 16 accounts
- Maximum 32 bytes data
- Minimum data (0 bytes)
- All numeric types

## Expected Output

Running `cargo test --workspace cpi` should show:

```
running 67 tests

cpi_compilation_tests::test_interface_declaration_parsing ... ok
cpi_compilation_tests::test_spl_token_mint_interface ... ok
cpi_compilation_tests::test_anchor_8byte_discriminator ... ok
[... 64 more tests ...]

test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured
```

## Notes

### Why No On-Chain Tests
- Tests run **without Solana cluster** (no solana-test-validator needed)
- Tests run **without network calls**
- Tests run **without account state**
- Tests are **deterministic** (no network latency)
- Tests are **reproducible** (same results every time)
- Tests are **fast** (all 67 tests in <1 second)

### Test Limitations
These tests verify:
- ✅ Compilation correctness
- ✅ Bytecode generation
- ✅ Serialization format
- ✅ Opcode structure
- ✅ Parameter handling

These tests don't verify:
- ❌ On-chain execution (use integration tests instead)
- ❌ Real SPL Token interactions (use devnet tests)
- ❌ Account state changes (use e2e tests)
- ❌ Transaction fees (not applicable offline)

## Continuous Integration

These tests are designed to run in CI/CD:
- No external dependencies
- No network access required
- Deterministic results
- Fast execution
- Clear pass/fail status

Example CI command:
```bash
cargo test --workspace cpi --release
```

## Debugging Failing Tests

If a test fails:

1. **Run with backtrace:**
   ```bash
   RUST_BACKTRACE=1 cargo test --workspace test_name
   ```

2. **Run with output:**
   ```bash
   cargo test --workspace test_name -- --nocapture
   ```

3. **Run single-threaded:**
   ```bash
   cargo test --workspace test_name -- --test-threads=1
   ```

4. **Check test source:**
   - Compiler tests: `five-dsl-compiler/tests/cpi_unit_tests.rs`
   - VM tests: `five-vm-mito/tests/cpi_unit_tests.rs`

## Adding New Tests

To add a new CPI test:

1. **Identify category:**
   - Compilation: Add to `cpi_compilation_tests` mod
   - Serialization: Add to `cpi_serialization_tests` mod
   - VM execution: Add to `cpi_invoke_tests` mod
   - Error handling: Add to `cpi_error_handling_tests` mod

2. **Follow naming pattern:**
   - `test_<feature>_<scenario>`
   - Example: `test_spl_token_mint_interface`

3. **Document test:**
   - Add comment explaining what is being tested
   - Reference related issues/PRs

4. **Example:**
   ```rust
   #[test]
   fn test_my_new_cpi_feature() {
       let source = r#"
           interface MyInterface @program("...") {
               // ...
           }
       "#;

       let result = compile_cpi_contract(source);
       assert!(result.is_ok(), "Should compile successfully");
   }
   ```

## References

- **CPI Guide:** `docs/CPI_GUIDE.md`
- **Compiler Source:** `five-dsl-compiler/src/interface_serializer.rs`
- **VM Source:** `five-vm-mito/src/handlers/system/invoke.rs`
- **Protocol:** `five-protocol/OPCODE_SPEC.md`
- **Example Tests:** `five-templates/cpi-examples/`
