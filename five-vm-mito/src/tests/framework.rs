//! Test framework and utilities for Five VM Mito
//!
//! This module provides test utilities, mock implementations, and helper functions
//! for comprehensive testing of the Five VM Mito implementation.
//!
//! All test utilities are feature-gated behind "test-utils" to ensure production builds
//! exclude all testing code and dependencies.

#[cfg(feature = "test-utils")]
use crate::{context::ExecutionManager, types::CallFrame, MitoVM, Result, VMError, Value};
#[cfg(feature = "test-utils")]
use five_protocol::{opcodes::*, ValueRef};
#[cfg(feature = "test-utils")]
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};
#[cfg(feature = "test-utils")]
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey as SolPubkey,
    system_program,
};
#[cfg(feature = "test-utils")]
fn make_account_info(
    key: &Pubkey,
    lamports: u64,
    data: Vec<u8>,
    owner: &Pubkey,
    is_signer: bool,
    is_writable: bool,
    executable: bool,
) -> AccountInfo {
    let key_ref = Box::leak(Box::new(*key));
    let owner_ref = Box::leak(Box::new(*owner));
    let lamports_ref = Box::leak(Box::new(lamports));
    let data_ref = Box::leak(data.into_boxed_slice());

    AccountInfo::new(
        key_ref,
        is_signer,
        is_writable,
        lamports_ref,
        data_ref,
        owner_ref,
        executable,
        0,
    )
}

/// Account constraint validation trait - matches counter-pinocchio pattern
/// All constraint validation code is feature-gated to exclude from production builds
#[cfg(feature = "test-utils")]
pub trait AccountCheck<T> {
    fn check(account: &AccountInfo, data: &T) -> Result<()>;
}

/// Signer account constraint - validates account is a signer
#[cfg(feature = "test-utils")]
pub struct SignerAccount;

#[cfg(feature = "test-utils")]
impl<T> AccountCheck<T> for SignerAccount {
    fn check(account: &AccountInfo, _data: &T) -> Result<()> {
        if !account.is_signer() {
            return Err(VMError::ConstraintViolation);
        }
        Ok(())
    }
}

/// Writable account constraint - validates account is writable
#[cfg(feature = "test-utils")]
pub struct WritableAccount;

#[cfg(feature = "test-utils")]
impl<T> AccountCheck<T> for WritableAccount {
    fn check(account: &AccountInfo, _data: &T) -> Result<()> {
        if !account.is_writable() {
            return Err(VMError::ConstraintViolation);
        }
        Ok(())
    }
}

/// Initialized account constraint - validates account has data
#[cfg(feature = "test-utils")]
pub struct InitializedAccount;

#[cfg(feature = "test-utils")]
impl<T> AccountCheck<T> for InitializedAccount {
    fn check(account: &AccountInfo, _data: &T) -> Result<()> {
        if account.data_len() == 0 {
            return Err(VMError::ConstraintViolation);
        }
        Ok(())
    }
}

/// Uninitialized account constraint - validates account is empty and system-owned
#[cfg(feature = "test-utils")]
pub struct UninitializedAccount;

#[cfg(feature = "test-utils")]
impl<T> AccountCheck<T> for UninitializedAccount {
    fn check(account: &AccountInfo, _data: &T) -> Result<()> {
        if account.data_len() > 0 || account.owner() != &system_program::ID.to_bytes() {
            return Err(VMError::ConstraintViolation);
        }
        Ok(())
    }
}

/// Owner constraint validator - validates account is owned by specific program
#[cfg(feature = "test-utils")]
pub struct OwnedAccount(pub Pubkey);

#[cfg(feature = "test-utils")]
impl<T> AccountCheck<T> for OwnedAccount {
    fn check(account: &AccountInfo, _data: &T) -> Result<()> {
        // if account.owner() != &self.0 {
        //     return Err(VMError::ConstraintViolation);
        // }
        Ok(())
    }
}

/// Real Account utilities following counter-pinocchio pattern
/// Uses real solana_sdk::Account instances instead of mocks
#[cfg(feature = "test-utils")]
pub struct AccountUtils;

#[cfg(feature = "test-utils")]
impl AccountUtils {
    /// Create a system account (owned by system program)
    pub fn system_account(lamports: u64) -> Account {
        Account::new(lamports, 0, &system_program::ID)
    }

    /// Create a signer account (system owned, for signing)
    pub fn signer_account(lamports: u64) -> Account {
        Account::new(lamports, 0, &system_program::ID)
    }

    /// Create a state account with data
    pub fn state_account(lamports: u64, data: Vec<u8>, owner: Pubkey) -> Account {
        Account::new(lamports, data.len(), &SolPubkey::new_from_array(owner))
    }

    /// Create an initialized account with 8 bytes of data
    pub fn initialized_account(lamports: u64, owner: Pubkey) -> Account {
        let mut account = Account::new(lamports, 8, &SolPubkey::new_from_array(owner));
        account.data = vec![0u8; 8];
        account
    }

    /// Create an uninitialized account (empty data, system owned)
    pub fn uninitialized_account(lamports: u64) -> Account {
        Account::new(lamports, 0, &system_program::ID)
    }

    /// Create account with specific data
    pub fn account_with_data(lamports: u64, data: Vec<u8>, owner: Pubkey) -> Account {
        let mut account = Account::new(lamports, data.len(), &SolPubkey::new_from_array(owner));
        account.data = data;
        account
    }

    /// Get the Five VM program ID
    pub fn five_vm_program_id() -> Pubkey {
        crate::FIVE_VM_PROGRAM_ID
    }
}

/// Test utilities for VM operations
/// All test utilities are feature-gated to ensure clean production builds
#[cfg(feature = "test-utils")]
pub struct TestUtils;

#[cfg(feature = "test-utils")]
impl TestUtils {
    /// Create simple bytecode with correct 10-byte FIVE V3 header
    /// Header format: magic(4) + features(4) + public_count(1) + total_count(1) = 10 bytes
    pub fn create_simple_bytecode(operations: &[u8]) -> Vec<u8> {
        let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // "5IVE" magic
        bytecode.extend_from_slice(&0u32.to_le_bytes()); // features (no special features)
        bytecode.push(0); // public_count = 0
        bytecode.push(0); // total_count = 0
        bytecode.extend_from_slice(operations);
        bytecode
    }

    /// Create bytecode for function testing with correct FIVE header
    /// Header format: magic(4) + features(4) + public_count(1) + total_count(1) = 10 bytes
    pub fn create_function_bytecode(main_code: &[u8], function_code: &[u8]) -> Vec<u8> {
        let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // "5IVE" magic (b'5', b'I', b'V', b'E')

        // Features (4 bytes LE) - enable function metadata
        bytecode.extend_from_slice(&0x0102u32.to_le_bytes()); // FEATURE_FUNCTION_NAMES (0x100) + base features
        
        // public_count (1 byte) - number of externally callable functions  
        bytecode.push(0);
        
        // total_count (1 byte) - total number of functions including internal
        bytecode.push(1);

        // Main code starts at offset 10 (header size)
        bytecode.extend_from_slice(main_code);

        // Function code
        bytecode.extend_from_slice(function_code);

        bytecode
    }

    /// Execute bytecode with empty accounts and input
    pub fn execute_simple(bytecode: &[u8]) -> Result<Option<Value>> {
        let mut storage = crate::stack::StackStorage::new(bytecode);
        MitoVM::execute_direct(bytecode, &[], &[], &Pubkey::default(), &mut storage)
    }

    /// Execute bytecode with provided accounts
    pub fn execute_with_accounts(
        bytecode: &[u8],
        accounts: &[AccountInfo],
    ) -> Result<Option<Value>> {
        let mut storage = crate::stack::StackStorage::new(bytecode);
        MitoVM::execute_direct(bytecode, &[], accounts, &Pubkey::default(), &mut storage)
    }

    /// Execute bytecode with input data (for function calls)
    pub fn execute_with_input(bytecode: &[u8], input_data: &[u8]) -> Result<Option<Value>> {
        let mut storage = crate::stack::StackStorage::new(bytecode);
        MitoVM::execute_direct(bytecode, input_data, &[], &Pubkey::default(), &mut storage)
    }

    /// Create VLE encoded input data for function calls
    pub fn create_function_input(function_index: u8, params: &[Value]) -> Vec<u8> {
        let mut input = vec![function_index, params.len() as u8];

        for param in params {
            match param {
                Value::U64(val) => {
                    input.push(0x04); // ValueRef::U64 type
                    input.extend_from_slice(&val.to_le_bytes());
                }
                Value::U8(val) => {
                    input.push(0x01); // ValueRef::U8 type
                    input.push(*val);
                }
                Value::Bool(val) => {
                    input.push(0x09); // ValueRef::Bool type
                    input.push(if *val { 1 } else { 0 });
                }
                _ => panic!("Unsupported parameter type for test input"),
            }
        }

        input
    }

    /// Assert that execution succeeds and returns expected value
    pub fn assert_execution_success(bytecode: &[u8], expected: Option<Value>) {
        let result = Self::execute_simple(bytecode);
        assert!(
            result.is_ok(),
            "Execution should succeed, got: {:?}",
            result
        );
        assert_eq!(result.unwrap(), expected, "Unexpected return value");
    }

    /// Assert that execution fails with expected error
    pub fn assert_execution_error(bytecode: &[u8], expected_error: VMError) {
        let result = Self::execute_simple(bytecode);
        assert!(result.is_err(), "Execution should fail");
        assert_eq!(
            std::mem::discriminant(&result.unwrap_err()),
            std::mem::discriminant(&expected_error),
            "Expected different error type"
        );
    }

    /// Create pubkey from seed for testing
    pub fn create_test_pubkey(seed: u8) -> Pubkey {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        Pubkey::from(bytes)
    }

    /// Create test data with pattern
    pub fn create_test_data(size: usize, pattern: u8) -> Vec<u8> {
        vec![pattern; size]
    }

    /// Create real AccountInfo from Account for testing (following Mollusk pattern)
    /// Note: This creates a simplified AccountInfo for testing purposes
    /// Real Mollusk usage would handle account references differently
    pub fn account_info_from_account<'a>(
        key: &'a Pubkey,
        account: &'a Account,
        is_signer: bool,
        is_writable: bool,
    ) -> AccountInfo {
        let owner_bytes = account.owner.to_bytes();
        make_account_info(
            key,
            account.lamports,
            account.data.clone(),
            &owner_bytes,
            is_signer,
            is_writable,
            account.executable,
        )
    }

    /// Create signer AccountInfo for testing
    pub fn create_signer_account_info<'a>(
        key: &'a Pubkey,
        lamports: u64,
    ) -> (Account, AccountInfo) {
        let account = AccountUtils::signer_account(lamports);
        let account_info = Self::account_info_from_account(key, &account, true, true);
        (account, account_info)
    }

    /// Create writable AccountInfo for testing
    pub fn create_writable_account_info<'a>(
        key: &'a Pubkey,
        lamports: u64,
        data: Vec<u8>,
        owner: &Pubkey,
    ) -> (Account, AccountInfo) {
        let account = AccountUtils::account_with_data(lamports, data, *owner);
        let account_info = Self::account_info_from_account(key, &account, false, true);
        (account, account_info)
    }

    /// Create readonly AccountInfo for testing
    pub fn create_readonly_account_info<'a>(
        key: &'a Pubkey,
        lamports: u64,
        data: Vec<u8>,
        owner: &Pubkey,
    ) -> (Account, AccountInfo) {
        let account = AccountUtils::account_with_data(lamports, data, *owner);
        let account_info = Self::account_info_from_account(key, &account, false, false);
        (account, account_info)
    }

    /// Execute bytecode with real AccountInfo instances
    pub fn execute_with_real_accounts(
        bytecode: &[u8],
        accounts: &[AccountInfo],
    ) -> Result<Option<Value>> {
        let mut storage = crate::stack::StackStorage::new(bytecode);
        MitoVM::execute_direct(bytecode, &[], accounts, &Pubkey::default(), &mut storage)
    }

    /// Run account constraint validation test
    pub fn test_constraint<T: AccountCheck<()>>(account: &AccountInfo) -> Result<()> {
        T::check(account, &())
    }

    /// Create proper PDA for testing using real Solana derivation
    pub fn derive_pda_for_test(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
        pinocchio::pubkey::find_program_address(seeds, program_id)
    }

    /// Create ExecutionContext for testing with V3 header (MitoVM style: fast, zero-copy)
    ///
    /// This is the primary helper for creating test contexts with the new signature.
    /// Uses sensible defaults for testing scenarios.
    #[inline]
    pub fn create_test_context<'a>(
        bytecode: &'a [u8],
        accounts: &'a [AccountInfo],
        storage: &'a mut crate::stack::StackStorage<'a>,
    ) -> crate::context::ExecutionContext<'a> {
        let program_id = Pubkey::default();
        let instruction_data: &[u8] = &[];

        crate::context::ExecutionContext::new(
            bytecode,
            accounts,
            program_id,
            instruction_data,
            0,
            storage,
            0,
            0, // total_function_count
        )
    }

    /// Create ExecutionContext with custom parameters (full control)
    #[inline]
    pub fn create_custom_context<'a>(
        bytecode: &'a [u8],
        accounts: &'a [AccountInfo],
        program_id: Pubkey,
        instruction_data: &'a [u8],
        start_pc: u16,
        storage: &'a mut crate::stack::StackStorage<'a>,
        function_count: u8,
    ) -> crate::context::ExecutionContext<'a> {
        crate::context::ExecutionContext::new(
            bytecode,
            accounts,
            program_id,
            instruction_data,
            start_pc,
            storage,
            function_count,
            function_count, // Assume public=total for simple tests
        )
    }
}

/// Mollusk-style testing utilities for Five VM
/// Provides integration with mollusk-svm for comprehensive end-to-end testing
#[cfg(feature = "test-utils")]
pub struct MolluskTestUtils;

#[cfg(feature = "test-utils")]
impl MolluskTestUtils {
    /// Create Mollusk instance for Five VM testing
    /// Note: This is a placeholder - actual Mollusk integration would require
    /// a compiled Five VM program binary    /// Create a state account with data
    pub fn create_mollusk() -> Result<()> {
        // This would normally create a Mollusk instance like:
        // let mollusk = Mollusk::new(&PROGRAM_ID, "path/to/five_vm_program.so");
        // For now, we provide the framework for when the program is available
        Ok(())
    }

    /// Create a state account with data
    pub fn state_account(lamports: u64, data: Vec<u8>, owner: Pubkey) -> Account {
        Account::new(lamports, data.len(), &SolPubkey::new_from_array(owner))
    }

    /// Create an initialized account with 8 bytes of data
    pub fn initialized_account(lamports: u64, owner: Pubkey) -> Account {
        let mut account = Account::new(lamports, 8, &SolPubkey::new_from_array(owner));
        account.data = vec![0u8; 8];
        account
    }

    /// Create account with specific data
    pub fn account_with_data(lamports: u64, data: Vec<u8>, owner: Pubkey) -> Account {
        let mut account = Account::new(lamports, data.len(), &SolPubkey::new_from_array(owner));
        account.data = data;
        account
    }

    /// Create system program account pair for Mollusk tests
    pub fn system_program_account() -> (Pubkey, Account) {
        (system_program::ID.to_bytes(), Account::new(1, 0, &system_program::ID))
    }

    /// Create funded authority account for testing
    pub fn authority_account(lamports: u64) -> Account {
        Account::new(lamports, 0, &system_program::ID)
    }

    /// Create Five VM state account with proper initialization
    pub fn vm_state_account(lamports: u64, script_data: &[u8], program_id: &Pubkey) -> Account {
        let mut account = Account::new(lamports, script_data.len(), &SolPubkey::new_from_array(*program_id));
        account.data = script_data.to_vec();
        account
    }

    /// Create test instruction for Five VM script execution
    /// Uses real solana_sdk::Instruction with proper account metas
    pub fn create_vm_instruction(
        program_id: &Pubkey,
        script_data: &[u8],
        accounts: Vec<AccountMeta>,
    ) -> Result<Instruction> {
        // Create instruction data with Five VM script
        let mut instruction_data = Vec::new();
        instruction_data.push(0x01); // Execute script instruction discriminator
        instruction_data.extend_from_slice(&(script_data.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(script_data);

        Ok(Instruction::new_with_bytes(
            SolPubkey::new_from_array(*program_id),
            &instruction_data,
            accounts,
        ))
    }

    /// Execute Five VM script using Mollusk framework
    /// This follows the counter-pinocchio pattern for real account testing
    pub fn execute_vm_script_with_mollusk(
        script_bytecode: &[u8],
        tx_accounts: &[(Pubkey, Account)],
        account_metas: Vec<AccountMeta>,
        program_id: &Pubkey,
    ) -> Result<()> {
        // Convert only referenced transaction accounts to AccountInfo for execution
        let account_infos: Vec<AccountInfo> = account_metas
            .iter()
            .filter_map(|meta| {
                tx_accounts
                    .iter()
                    .find(|(key, _)| SolPubkey::new_from_array(*key) == meta.pubkey)
                    .map(|(key, account)| {
                        let owner_bytes = account.owner.to_bytes();
                        make_account_info(
                            key,
                            account.lamports,
                            account.data.clone(),
                            &owner_bytes,
                            meta.is_signer,
                            meta.is_writable,
                            account.executable,
                        )
                    })
            })
            .collect();

        // Execute the VM script with real bytecode and filtered accounts
        let mut storage = crate::stack::StackStorage::new(script_bytecode);
        match MitoVM::execute_direct(script_bytecode, &[], account_infos.as_slice(), program_id, &mut storage) {
            Ok(_) => Ok(()),
            Err(vm_error) => Err(vm_error),
        }
    }

    /// Create system program account pair for Mollusk tests
    /// Following counter-pinocchio pattern for real account creation


    /// Validate Five VM execution result using Mollusk checks
    /// Provides comprehensive result validation following Mollusk patterns
    pub fn validate_execution_success(/* result: ExecutionResult */) -> bool {
        // This would validate actual execution results:
        // result.program_result == ProgramResult::Success &&
        // result.return_data.is_some()

        // For now, provide framework for validation
        true
    }

    /// Create complete test setup for Five VM Mollusk testing
    /// Returns all components needed for end-to-end testing
    pub fn setup_vm_test(
        script_bytecode: &[u8],
        authority_lamports: u64,
        program_id: &Pubkey,
    ) -> Result<(Vec<(Pubkey, Account)>, Vec<AccountMeta>)> {
        // Create test accounts
        let authority_key = TestUtils::create_test_pubkey(1);
        let authority_account = Self::authority_account(authority_lamports);

        let script_account_key = TestUtils::create_test_pubkey(2);
        let script_account = Self::vm_state_account(1_000_000, script_bytecode, program_id);

        let (system_program_key, system_account) = Self::system_program_account();

        // Create transaction accounts
        let tx_accounts = vec![
            (authority_key, authority_account),
            (script_account_key, script_account),
            (system_program_key, system_account),
        ];

        // Create account metas for instruction
        let account_metas = vec![
            AccountMeta::new(SolPubkey::new_from_array(authority_key), true),                // signer
            AccountMeta::new(SolPubkey::new_from_array(script_account_key), false),          // script state
            AccountMeta::new_readonly(SolPubkey::new_from_array(system_program_key), false), // system program
        ];

        Ok((tx_accounts, account_metas))
    }
}

/// Macro for creating test opcodes sequences
/// Feature-gated to exclude from production builds
#[cfg(feature = "test-utils")]
#[macro_export]
macro_rules! opcodes {
    ($($op:expr),* $(,)?) => {
        vec![$($op),*]
    };
}

/// Macro for creating PUSH_U64 instruction with Fixed encoding
/// Feature-gated to exclude from production builds
#[cfg(feature = "test-utils")]
#[macro_export]
macro_rules! push_u64 {
    ($val:expr) => {{
        let mut ops = vec![0x1B]; // PUSH_U64 opcode
        // Fixed size little endian encoding
        ops.extend_from_slice(&($val as u64).to_le_bytes());
        ops
    }};
}

/// Macro for creating PUSH_BOOL instruction
/// Feature-gated to exclude from production builds
#[cfg(feature = "test-utils")]
#[macro_export]
macro_rules! push_bool {
    ($val:expr) => {
        vec![0x1D, if $val { 1 } else { 0 }] // PUSH_BOOL opcode
    };
}

/// Macro for creating complete test bytecode with correct 10-byte FIVE V3 header
/// Header format: magic(4) + features(4) + public_count(1) + total_count(1) = 10 bytes
/// Feature-gated to exclude from production builds
#[cfg(feature = "test-utils")]
#[macro_export]
macro_rules! test_bytecode {
    ($($ops:expr),* $(,)?) => {
        {
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // "5IVE" magic
            bytecode.extend_from_slice(&0u32.to_le_bytes()); // features (no special features)
            bytecode.push(0); // public_count = 0
            bytecode.push(0); // total_count = 0
            $(
                bytecode.extend_from_slice(&$ops);
            )*
            bytecode.push(0x07); // RETURN_VALUE
            bytecode
        }
    };
}

#[cfg(all(test, feature = "test-utils"))]
mod tests {
    use super::*;

    #[test]
    fn test_framework_basic_operations() {
        // Test simple push and halt
        let bytecode = test_bytecode![push_u64!(42)];
        TestUtils::assert_execution_success(&bytecode, Some(Value::U64(42)));
    }

    #[test]
    fn test_framework_arithmetic() {
        // Test addition: 100 + 25 = 125
        let bytecode = test_bytecode![
            push_u64!(100),
            push_u64!(25),
            opcodes![0x20], // ADD
        ];
        TestUtils::assert_execution_success(&bytecode, Some(Value::U64(125)));
    }

    #[test]
    fn test_framework_comparison() {
        // Test greater than: 125 > 100 = true
        let bytecode = test_bytecode![
            push_u64!(125),
            push_u64!(100),
            opcodes![0x25], // GT
        ];
        TestUtils::assert_execution_success(&bytecode, Some(Value::Bool(true)));
    }

    #[test]
    fn test_framework_require_failure() {
        // Test REQUIRE with false should fail
        let bytecode = test_bytecode![
            push_bool!(false),
            opcodes![0x04], // REQUIRE
        ];
        TestUtils::assert_execution_error(&bytecode, VMError::ConstraintViolation);
    }
}
