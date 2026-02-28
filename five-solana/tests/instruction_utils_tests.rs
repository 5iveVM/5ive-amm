#[cfg(test)]
mod instruction_utils_tests {
    use five::instructions::fees::transfer_fee;
    use five::instructions::{require_min_accounts, require_signer};
    use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

    // Helper to create a fake AccountInfo
    fn create_account_info<'a>(
        key: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
    ) -> AccountInfo {
        AccountInfo::new(key, is_signer, is_writable, lamports, data, owner, false, 0)
    }

    #[test]
    fn test_require_min_accounts() {
        let key = Pubkey::default();
        let mut lamports = 0;
        let mut data = [];
        let owner = Pubkey::default();

        let account = create_account_info(&key, false, false, &mut lamports, &mut data, &owner);
        let accounts = [account.clone(), account.clone()];

        // Success
        assert_eq!(require_min_accounts(&accounts, 1), Ok(()));
        assert_eq!(require_min_accounts(&accounts, 2), Ok(()));

        // Failure
        assert_eq!(
            require_min_accounts(&accounts, 3),
            Err(ProgramError::NotEnoughAccountKeys)
        );
    }

    #[test]
    fn test_require_signer() {
        let key = Pubkey::default();
        let mut lamports = 0;
        let mut data = [];
        let owner = Pubkey::default();

        // Success
        let signer = create_account_info(&key, true, false, &mut lamports, &mut data, &owner);
        assert_eq!(require_signer(&signer), Ok(()));

        // Failure
        let non_signer = create_account_info(&key, false, false, &mut lamports, &mut data, &owner);
        assert_eq!(
            require_signer(&non_signer),
            Err(ProgramError::MissingRequiredSignature)
        );
    }

    #[test]
    fn test_transfer_fee_direct() {
        let payer_key = Pubkey::from([1u8; 32]);
        let recipient_key = Pubkey::from([2u8; 32]);
        let program_id = Pubkey::from([3u8; 32]); // Not system program
        let mut payer_lamports = 1000;
        let mut recipient_lamports = 100;
        let mut data = [];

        let payer = create_account_info(
            &payer_key,
            true,
            true,
            &mut payer_lamports,
            &mut data,
            &program_id,
        );
        let recipient = create_account_info(
            &recipient_key,
            false,
            true,
            &mut recipient_lamports,
            &mut data,
            &program_id,
        );

        // Success transfer
        assert_eq!(
            transfer_fee(&program_id, &payer, &recipient, 100, None),
            Ok(())
        );
        assert_eq!(payer.lamports(), 900);
        assert_eq!(recipient.lamports(), 200);

        // Fail: insufficient funds
        assert_eq!(
            transfer_fee(&program_id, &payer, &recipient, 2000, None),
            Err(ProgramError::InsufficientFunds)
        );

        // Zero amount (no-op)
        assert_eq!(
            transfer_fee(&program_id, &payer, &recipient, 0, None),
            Ok(())
        );
        assert_eq!(payer.lamports(), 900);
        assert_eq!(recipient.lamports(), 200);

        // Payer == Recipient (no-op)
        assert_eq!(transfer_fee(&program_id, &payer, &payer, 100, None), Ok(()));
        assert_eq!(payer.lamports(), 900);
    }

    // Note: Testing system program transfer via CPI is hard in unit tests because
    // `pinocchio::program::invoke` attempts to call into the runtime which is not present.
    // However, we can test the logic branch selection if we mock the owner properly.
    // But since invoke will panic or fail, we should probably stick to testing the direct branch
    // or test that it ATTEMPTS to invoke if we can catch it, but we can't easily catch CPI calls here.
    // So we'll skip the system program path test for now or accept that it's hard to test in isolation without a simulator.

    // We can at least test that it FAILS with "MissingRequiredSignature" (used as error code in the implementation)
    // if system_program is None when payer is system owned.
    #[test]
    fn test_transfer_fee_system_missing_program() {
        let payer_key = Pubkey::from([1u8; 32]);
        let recipient_key = Pubkey::from([2u8; 32]);
        let system_program_id = [0u8; 32];
        let mut payer_lamports = 1000;
        let mut recipient_lamports = 100;
        let mut data = [];

        let payer = create_account_info(
            &payer_key,
            true,
            true,
            &mut payer_lamports,
            &mut data,
            &Pubkey::from(system_program_id),
        );
        let recipient = create_account_info(
            &recipient_key,
            false,
            true,
            &mut recipient_lamports,
            &mut data,
            &Pubkey::default(),
        );
        let program_id = Pubkey::from([9u8; 32]);

        // This should hit the check `let system_program = system_program.ok_or(...)`
        assert_eq!(
            transfer_fee(&program_id, &payer, &recipient, 100, None),
            Err(ProgramError::MissingRequiredSignature)
        );
    }
}
