//! Tests for importing bytecode from account data

#[cfg(test)]
mod import_bytecode_tests {
    use crate::BytecodeUtils;
    use crate::test_framework::{TestUtils, AccountUtils};

    #[test]
    fn imports_bytecode_when_account_has_data() {
        let owner = TestUtils::five_vm_program_id();
        let key = TestUtils::create_test_pubkey(1);
        let account = AccountUtils::account_with_data(1_000_000, vec![1, 2, 3], owner);
        let account_info = TestUtils::account_info_from_account(&key, &account, false, false);

        let result = BytecodeUtils::import_account_bytecode(&account_info);
        assert!(result.is_ok(), "expected Ok for account with data");
        assert_eq!(result.unwrap(), &[1, 2, 3]);
    }

    #[test]
    fn errors_when_account_has_no_data() {
        let key = TestUtils::create_test_pubkey(2);
        let account = AccountUtils::uninitialized_account(1_000_000);
        let account_info = TestUtils::account_info_from_account(&key, &account, false, false);

        let result = BytecodeUtils::import_account_bytecode(&account_info);
        assert!(result.is_err(), "expected Err for empty account");
    }
}

