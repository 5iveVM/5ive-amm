//! In-process instruction tests are limited to parser/unit semantics.
//! Runtime invocation behavior belongs in ProgramTest BPF suites.

#[cfg(test)]
mod tests {
    use five::instructions::{set_fees, FIVEInstruction, DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION};
    use five::state::FIVEVMState;
    use pinocchio::account_info::AccountInfo;
    use pinocchio::program_error::ProgramError;
    use pinocchio::pubkey::Pubkey;

    fn create_account(
        key: &Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &mut u64,
        data: &mut [u8],
        owner: &Pubkey,
    ) -> AccountInfo {
        AccountInfo::new(key, is_signer, is_writable, lamports, data, owner, false, 0)
    }

    fn key(seed: u8) -> Pubkey {
        [seed; 32]
    }

    fn canonical_program_id() -> Pubkey {
        five::hardcoded_program_id()
    }

    fn canonical_vm_key(program_id: &Pubkey) -> Pubkey {
        let mut pid = [0u8; 32];
        pid.copy_from_slice(program_id);
        let (pda, _bump) = five_vm_mito::utils::find_program_address_offchain(&[b"vm_state"], &pid)
            .expect("canonical vm_state pda");
        pda
    }

    #[test]
    fn instruction_parsing() {
        let init_ix = FIVEInstruction::try_from([0, 0].as_slice()).unwrap();
        assert!(matches!(init_ix, FIVEInstruction::Initialize { .. }));

        let bytecode = vec![0x35, 0x49, 0x56, 0x45, 0x00];
        let mut deploy_data = vec![DEPLOY_INSTRUCTION];
        deploy_data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
        deploy_data.push(0);
        deploy_data.extend_from_slice(&0u32.to_le_bytes());
        deploy_data.extend_from_slice(&bytecode);
        let deploy_ix = FIVEInstruction::try_from(deploy_data.as_slice()).unwrap();
        assert!(matches!(deploy_ix, FIVEInstruction::Deploy { .. }));

        let mut exec_data = vec![EXECUTE_INSTRUCTION];
        exec_data.extend_from_slice(&0u32.to_le_bytes());
        exec_data.extend_from_slice(&0u32.to_le_bytes());
        let exec_ix = FIVEInstruction::try_from(exec_data.as_slice()).unwrap();
        assert!(matches!(exec_ix, FIVEInstruction::Execute { .. }));

        let migrate_ix = FIVEInstruction::try_from([15].as_slice()).unwrap();
        assert!(matches!(migrate_ix, FIVEInstruction::MigrateVmState));
    }

    #[test]
    fn invalid_instruction_shapes() {
        assert!(matches!(
            FIVEInstruction::try_from(&[][..]),
            Err(ProgramError::InvalidInstructionData)
        ));
        assert!(matches!(
            FIVEInstruction::try_from(&[99][..]),
            Err(ProgramError::InvalidInstructionData)
        ));
        assert!(matches!(
            FIVEInstruction::try_from(&[DEPLOY_INSTRUCTION, 10, 0, 0][..]),
            Err(ProgramError::InvalidInstructionData)
        ));
        assert!(matches!(
            FIVEInstruction::try_from(&[15, 0][..]),
            Err(ProgramError::InvalidInstructionData)
        ));
    }

    #[test]
    fn set_fees_updates_state() {
        let program_id = canonical_program_id();
        let authority_key = key(2);
        let mut vm_lamports = 0u64;
        let mut vm_data = vec![0u8; FIVEVMState::LEN];
        let mut authority_lamports = 0u64;
        let mut authority_data = [];

        {
            let vm_state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
            vm_state.initialize(authority_key, 0);
        }

        let vm_key = canonical_vm_key(&program_id);
        let vm_account = create_account(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );
        let authority_account = create_account(
            &authority_key,
            true,
            false,
            &mut authority_lamports,
            &mut authority_data,
            &program_id,
        );

        let accounts = [vm_account, authority_account];
        set_fees(&program_id, &accounts, 250, 150).unwrap();

        let updated_data = accounts[0].try_borrow_data().unwrap();
        let updated = FIVEVMState::from_account_data(&updated_data).unwrap();
        assert_eq!(updated.deploy_fee_lamports, 250);
        assert_eq!(updated.execute_fee_lamports, 150);
    }
}
