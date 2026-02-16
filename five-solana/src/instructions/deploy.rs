use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
    instruction::{Seed, Signer, AccountMeta, Instruction},
    program::invoke_signed, sysvars::Sysvar,
};

use crate::{
    common::{
        validate_vm_and_script_accounts, verify_program_owned, validate_permissions,
        verify_hardcoded_vm_state_account_with_bump,
        verify_admin_signer,
    },
    error::program_already_initialized_error,
    state::{FIVEVMState, ScriptAccountHeader},
};

use super::{
    fees::{collect_deploy_fee, collect_deploy_fee_with_state},
    verify::{verify_bytecode_content},
    require_min_accounts, require_signer, safe_realloc,
};

/// Minimum deployment instruction length:
/// discriminator + u32 bytecode length + permissions byte + u32 metadata length.
pub const MIN_DEPLOY_LEN: usize = 10;

/// Initialize the VM state account
pub fn initialize(program_id: &Pubkey, accounts: &[AccountInfo], bump: u8) -> ProgramResult {
    require_min_accounts(accounts, 2)?;

    let vm_state_account = &accounts[0];
    let authority = &accounts[1];
    verify_hardcoded_vm_state_account_with_bump(vm_state_account, program_id, bump)?;

    // Check if the account is already owned by the program or needs to be created
    if vm_state_account.owner() == &Pubkey::default() {
        require_min_accounts(accounts, 4)?;
        let payer = &accounts[2];
        let system_program = &accounts[3];
        
        require_signer(payer)?;
        
        // Verify System Program ID
        if system_program.key() != &Pubkey::default() {
             return Err(ProgramError::InvalidAccountData);
        }

        // Calculate rent for VM state account
        let rent = pinocchio::sysvars::rent::Rent::get()
            .map_err(|_| ProgramError::AccountNotRentExempt)?;
        let rent_lamports = rent.minimum_balance(FIVEVMState::LEN);

        // Prepare CreateAccount instruction data
        let mut create_account_data = [0u8; 52];
        create_account_data[0..4].copy_from_slice(&0u32.to_le_bytes()); // CreateAccount discriminator
        create_account_data[4..12].copy_from_slice(&rent_lamports.to_le_bytes());
        create_account_data[12..20].copy_from_slice(&(FIVEVMState::LEN as u64).to_le_bytes());
        create_account_data[20..52].copy_from_slice(program_id.as_ref());

        let bump_seed = [bump];
        let seeds: &[Seed] = &[
            Seed::from(b"vm_state"),
            Seed::from(&bump_seed),
        ];
        let signer = Signer::from(seeds);

        let metas = [
            AccountMeta {
                pubkey: payer.key(),
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: vm_state_account.key(),
                is_signer: true, // PDA is signer
                is_writable: true,
            },
        ];

        let instruction = Instruction {
            program_id: system_program.key(),
            accounts: &metas,
            data: &create_account_data,
        };

        invoke_signed::<3>(&instruction, &[payer, vm_state_account, system_program], &[signer])?;
    } else {
        // Ownership is enough here because canonical key+bump was already checked above.
        // This allows legacy initialized state (stale stored bump) to pass into migration.
        verify_program_owned(vm_state_account, program_id)?;
    }

    require_signer(authority)?;

    // Initialize VM state exactly once.
    // SAFETY: Account verified owned by program (either by check or creation), mutable borrow is safe.
    let vm_state_data = unsafe { vm_state_account.borrow_mut_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data_mut(vm_state_data)?;
    if vm_state.is_initialized() {
        // Legacy migration path: older devnet/localnet vm_state accounts were initialized
        // before the canonical bump field was enforced. Allow the authority signer to
        // backfill the canonical bump without resetting counters/fees/authority.
        if vm_state.authority == *authority.key() && vm_state.vm_state_bump != bump {
            vm_state.version = FIVEVMState::VERSION;
            vm_state.vm_state_bump = bump;
            if vm_state.fee_vault_shard_count == 0 {
                vm_state.fee_vault_shard_count = FIVEVMState::DEFAULT_FEE_VAULT_SHARD_COUNT;
            }
            return Ok(());
        }
        return Err(program_already_initialized_error());
    }
    vm_state.initialize(*authority.key(), bump);

    Ok(())
}

/// Deploy a script using the optimized script header format with permissions
///
/// **Permissions**: The deployer specifies what this bytecode can do:
/// - PERMISSION_PRE_BYTECODE (0x01): Can run before another script
/// - PERMISSION_POST_BYTECODE (0x02): Can run after another script
/// - PERMISSION_PDA_SPECIAL_CHARS (0x04): Can use !, @, #, $, % in PDA seeds
///
/// **Admin Requirement**: Only the admin key can deploy bytecode with any special permissions.
/// If permissions != 0, the admin must sign the transaction.
#[allow(unused_variables)]
pub fn deploy(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    bytecode: &[u8],
    metadata: &[u8],
    permissions: u8,
    fee_shard_index: u8,
) -> ProgramResult {

    // Validate permissions bitmask
    validate_permissions(permissions)?;

    require_min_accounts(accounts, 5)?;

    let script_account = &accounts[0];
    let vm_state_account = &accounts[1];
    let owner = &accounts[2];

    validate_vm_and_script_accounts(program_id, script_account, vm_state_account)?;
    let vm_state_data = unsafe { vm_state_account.borrow_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data(&vm_state_data)?;
    require_signer(owner)?;

    // If any permissions are set, require admin key (VM authority) signature
    let (fee_vault_idx, system_program_idx) = if permissions != 0 {
        let admin_key = vm_state.authority;

        // Admin account must be present and be the signer when special permissions are used
        require_min_accounts(accounts, 6)?;
        let admin_account = &accounts[3];
        verify_admin_signer(admin_account, &admin_key)?;
        (4usize, 5usize)
    } else {
        (3usize, 4usize)
    };
    let fee_vault_account = &accounts[fee_vault_idx];
    let system_program = &accounts[system_program_idx];

    // Validate bytecode size
    if bytecode.len() < 4 || bytecode.len() > five_protocol::MAX_SCRIPT_SIZE {
        return Err(ProgramError::Custom(8001));
    }

    // Check if valid Five Protocol bytecode header format (10 bytes minimum)
    if bytecode.len() < five_protocol::FIVE_HEADER_OPTIMIZED_SIZE {
        return Err(ProgramError::Custom(8002));
    }
    if &bytecode[..4] != five_protocol::FIVE_MAGIC {
        return Err(ProgramError::Custom(8003));
    }

    // Verify bytecode content
    verify_bytecode_content(bytecode)?;

    #[cfg(not(feature = "debug-logs"))]
    let _ = program_id; // Suppress unused variable warning

    // Calculate required account size: header + metadata + bytecode
    let required_size = ScriptAccountHeader::LEN + metadata.len() + bytecode.len();

    if script_account.data_len() < required_size {
        return Err(ProgramError::Custom(7005));
    }

    // Prevent overwriting an existing deployed script account.
    // Upgrades must use explicit upload/append/finalize flow with owner checks.
    {
        let script_data = unsafe { script_account.borrow_data_unchecked() };
        if ScriptAccountHeader::is_valid(script_data) {
            return Err(ProgramError::Custom(7007));
        }
    }

    // Charge deploy fee only after all non-mutating deployment validations pass.
    collect_deploy_fee_with_state(
        program_id,
        owner,
        fee_vault_account,
        system_program,
        fee_shard_index,
        vm_state.deploy_fee_lamports as u64,
        vm_state.fee_vault_shard_count(),
    )?;

    // SAFETY: `vm_state_account` verified.
    let vm_state_data = unsafe { vm_state_account.borrow_mut_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data_mut(vm_state_data)?;
    let script_id = vm_state.create_script_id();

    // Write script header + bytecode to account
    // SAFETY: `script_account` is owned by this program and exclusively borrowed.
    let script_data = unsafe { script_account.borrow_mut_data_unchecked() };

    // Create header with cached metadata length.
    let mut header = ScriptAccountHeader::create_from_bytecode(
        bytecode,
        *owner.key(),
        script_id,
        permissions, // Use the permissions from the instruction
    );
    header.set_metadata_len(metadata.len());

    header.copy_into_account(script_data)?;
    let metadata_start = ScriptAccountHeader::LEN;
    let metadata_end = metadata_start + metadata.len();
    script_data[metadata_start..metadata_end].copy_from_slice(metadata);
    script_data[metadata_end..metadata_end + bytecode.len()]
        .copy_from_slice(bytecode);

    Ok(())
}

/// Initialize a script account for chunked large-program deployment.
/// If chunk_data is provided, it will be written as the first chunk (optimization).
pub fn init_large_program(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    expected_size: u32,
    chunk_data: Option<&[u8]>,
) -> ProgramResult {
    let chunk_len = chunk_data.map(|c| c.len()).unwrap_or(0);

    require_min_accounts(accounts, 5)?;

    let script_account = &accounts[0];
    let owner = &accounts[1];
    let vm_state_account = &accounts[2];
    let fee_vault_account = &accounts[3];
    let system_program = &accounts[4];

    verify_program_owned(script_account, program_id)?;

    // Verify VM state is owned by this program and initialized
    crate::common::verify_hardcoded_vm_state_account(vm_state_account, program_id)?;
    verify_program_owned(vm_state_account, program_id)?;
    let data = unsafe { vm_state_account.borrow_data_unchecked() };
    let state = FIVEVMState::from_account_data(data)?;
    if !state.is_initialized() {
        return Err(crate::error::program_not_initialized_error());
    }

    require_signer(owner)?;

    let expected_size = expected_size as usize;
    if expected_size < 4 || expected_size > five_protocol::MAX_SCRIPT_SIZE {
        return Err(ProgramError::Custom(8206)); // Invalid expected size
    }

    // Validate chunk size if present
    if let Some(chunk) = chunk_data {
        if chunk.len() > expected_size {
            return Err(ProgramError::Custom(8207)); // Initial chunk too large
        }
    }

    if script_account.data_len() < ScriptAccountHeader::LEN {
        return Err(ProgramError::Custom(7006));
    }

    // SAFETY: The script account is owned by this program; we only read its data.
    let script_data = unsafe { script_account.borrow_data_unchecked() };
    if ScriptAccountHeader::is_valid(&script_data) {
        return Err(ProgramError::Custom(7007));
    }

    // If the full bytecode is supplied in the initial chunk, charge deploy fee now.
    // This closes the fee-bypass path where finalize_script_upload could complete upload
    // without ever entering append_bytecode's completion fee branch.
    if chunk_len == expected_size {
        let final_size = ScriptAccountHeader::LEN + expected_size;
        collect_deploy_fee(
            program_id,
            vm_state_account,
            owner,
            fee_vault_account,
            system_program,
            0,
            final_size,
        )?;
    }

    // SAFETY: `vm_state_account` is verified and uniquely borrowed for mutation.
    let vm_state_data = unsafe { vm_state_account.borrow_mut_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data_mut(vm_state_data)?;
    let script_id = vm_state.create_script_id();

    let mut header = ScriptAccountHeader::new(expected_size, *owner.key(), script_id);
    header.set_upload_len(chunk_len as u32);
    header.set_upload_mode(true);
    header.set_upload_complete(false);

    // SAFETY: The script account is program-owned and borrowed mutably for header write.
    let script_data = unsafe { script_account.borrow_mut_data_unchecked() };
    header.copy_into_account(script_data)?;

    // Write chunk data if present (InitLargeProgramWithChunk optimization)
    if let Some(chunk) = chunk_data {
        let start = ScriptAccountHeader::LEN;
        let end = start + chunk.len();
        if script_data.len() < end {
            return Err(ProgramError::Custom(7006)); // Account too small
        }
        script_data[start..end].copy_from_slice(chunk);
    }

    Ok(())
}

/// Append a bytecode chunk to a large-program script account.
pub fn append_bytecode(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    chunk: &[u8],
) -> ProgramResult {
    require_min_accounts(accounts, 5)?;
    if chunk.is_empty() {
        return Err(ProgramError::Custom(8201)); // Empty chunk
    }

    let script_account = &accounts[0];
    let owner = &accounts[1];
    let vm_state_account = &accounts[2];
    let fee_vault_account = &accounts[3];
    let system_program = &accounts[4];

    validate_vm_and_script_accounts(program_id, script_account, vm_state_account)?;
    require_signer(owner)?;

    let (expected_size, current_len, script_id, permissions) = {
        // SAFETY: The script account is program-owned and borrowed mutably for header access.
        let script_data = unsafe { script_account.borrow_mut_data_unchecked() };
        let header = ScriptAccountHeader::from_account_data_mut(script_data)?;
        if header.owner != *owner.key() {
            return Err(ProgramError::InvalidArgument);
        }
        if !header.upload_mode() {
            return Err(ProgramError::Custom(7008));
        }
        (
            header.bytecode_len(),
            header.upload_len() as usize,
            header.script_id,
            header.permissions,
        )
    };

    if current_len + chunk.len() > expected_size {
        return Err(ProgramError::Custom(8202)); // Chunk exceeds expected size
    }

    let new_len = current_len + chunk.len();
    let new_total_len = ScriptAccountHeader::LEN + new_len;
    if script_account.data_len() < new_total_len {
        safe_realloc(script_account, owner, new_total_len)?;
    }

    // SAFETY: The script account is program-owned and borrowed mutably for data append.
    let script_data = unsafe { script_account.borrow_mut_data_unchecked() };
    let start = ScriptAccountHeader::LEN + current_len;
    let end = ScriptAccountHeader::LEN + new_len;
    script_data[start..end].copy_from_slice(chunk);

    let header = ScriptAccountHeader::from_account_data_mut(script_data)?;
    header.set_upload_len(new_len as u32);

    if new_len == expected_size {
        // Verify account is large enough before slicing
        let bytecode_end = ScriptAccountHeader::LEN + expected_size;
        if script_data.len() < bytecode_end {
            return Err(ProgramError::Custom(7006)); // Account size mismatch
        }

        let bytecode =
            &script_data[ScriptAccountHeader::LEN..bytecode_end];

        if bytecode.len() < 4 || bytecode.len() > five_protocol::MAX_SCRIPT_SIZE {
            return Err(ProgramError::Custom(8203)); // Invalid bytecode size
        }

        if bytecode.len() < five_protocol::FIVE_HEADER_OPTIMIZED_SIZE {
            return Err(ProgramError::Custom(8204)); // Header too small
        }
        if &bytecode[..4] != five_protocol::FIVE_MAGIC {
            return Err(ProgramError::Custom(8205)); // Invalid magic bytes
        }

        // Collect deployment fee if configured
        {
            let final_size = ScriptAccountHeader::LEN + expected_size;
            collect_deploy_fee(
                program_id,
                vm_state_account,
                owner,
                fee_vault_account,
                system_program,
                0,
                final_size,
            )?;
        }

        let mut final_header = ScriptAccountHeader::create_from_bytecode(
            bytecode,
            *owner.key(),
            script_id,
            permissions,
        );
        // Set upload flags BEFORE writing to account (single-write pattern)
        final_header.set_upload_len(0);
        final_header.set_upload_mode(false);
        final_header.set_upload_complete(true);
        // Single write with all flags correctly set
        final_header.copy_into_account(script_data)?;
    }

    Ok(())
}

/// Finalize script upload manually
pub fn finalize_script_upload(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    require_min_accounts(accounts, 2)?;
    let script_account = &accounts[0];
    let owner = &accounts[1];
    verify_program_owned(script_account, program_id)?;

    if !owner.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load header and check status
    let (expected_size, current_len, script_id, permissions) = {
        let script_data = unsafe { script_account.borrow_data_unchecked() };
        let header = ScriptAccountHeader::from_account_data(&script_data)?;

        if header.owner != *owner.key() {
            return Err(ProgramError::InvalidArgument);
        }
        if !header.upload_mode() {
            return Ok(()); // Already finalized
        }
        (
            header.bytecode_len(),
            header.upload_len() as usize,
            header.script_id,
            header.permissions,
        )
    };

    if current_len != expected_size {
        return Err(ProgramError::Custom(8208)); // Finalize size mismatch
    }

    // Verify bytecode
    let script_data = unsafe { script_account.borrow_mut_data_unchecked() };
    let bytecode = &script_data[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + expected_size];

    verify_bytecode_content(bytecode)?;

    // Update header
    let mut final_header = ScriptAccountHeader::create_from_bytecode(
        bytecode,
        *owner.key(),
        script_id,
        permissions,
    );
    // Set upload flags BEFORE writing to account (single-write pattern)
    final_header.set_upload_len(0);
    final_header.set_upload_mode(false);
    final_header.set_upload_complete(true);
    // Single write with all flags correctly set
    final_header.copy_into_account(script_data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pinocchio::account_info::AccountInfo;

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

    fn minimal_valid_bytecode() -> [u8; 11] {
        let mut b = [0u8; 11];
        b[0..4].copy_from_slice(&five_protocol::FIVE_MAGIC);
        b[8] = 1; // public function count
        b[9] = 1; // total function count
        b[10] = five_protocol::opcodes::HALT;
        b
    }

    fn canonical_vm_key(program_id: &Pubkey) -> (Pubkey, u8) {
        crate::common::derive_canonical_vm_state_pda(program_id).unwrap()
    }

    #[test]
    fn initialize_is_one_time_only() {
        let program_id = Pubkey::from([7u8; 32]);
        let (vm_key, vm_bump) = canonical_vm_key(&program_id);
        let authority_key = Pubkey::from([9u8; 32]);
        let system_owner = Pubkey::default();

        let mut vm_lamports = 1_000_000;
        let mut authority_lamports = 1_000_000;
        let mut vm_data = [0u8; FIVEVMState::LEN];
        let mut authority_data = [];

        let vm_account = create_account_info(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );
        let authority = create_account_info(
            &authority_key,
            true,
            false,
            &mut authority_lamports,
            &mut authority_data,
            &system_owner,
        );
        let accounts = [vm_account, authority];

        assert!(initialize(&program_id, &accounts, vm_bump).is_ok());
        assert_eq!(
            initialize(&program_id, &accounts, vm_bump),
            Err(program_already_initialized_error())
        );
    }

    #[test]
    fn deploy_rejects_overwrite_of_existing_script() {
        let program_id = Pubkey::from([11u8; 32]);
        let script_key = Pubkey::from([12u8; 32]);
        let (vm_key, vm_bump) = canonical_vm_key(&program_id);
        let owner_key = Pubkey::from([14u8; 32]);
        let system_owner = Pubkey::default();
        let (fee_vault_key, fee_vault_bump) =
            crate::common::derive_fee_vault_pda(&program_id, 0).unwrap();

        let bytecode = minimal_valid_bytecode();
        let mut script_data = vec![0u8; ScriptAccountHeader::LEN + bytecode.len()];
        let existing_header = ScriptAccountHeader::create_from_bytecode(&bytecode, owner_key, 1, 0);
        existing_header.copy_into_account(&mut script_data).unwrap();
        script_data[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + bytecode.len()]
            .copy_from_slice(&bytecode);

        let mut vm_data = [0u8; FIVEVMState::LEN];
        {
            let vm_state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
            vm_state.initialize(owner_key, vm_bump);
            vm_state.deploy_fee_lamports = 0;
        }

        let mut script_lamports = 1_000_000;
        let mut vm_lamports = 1_000_000;
        let mut owner_lamports = 1_000_000;
        let mut fee_vault_lamports = 0;
        let mut system_lamports = 1;
        let mut owner_data = [];
        let mut fee_vault_data = [];
        let mut system_data = [];

        let script_account = create_account_info(
            &script_key,
            false,
            true,
            &mut script_lamports,
            script_data.as_mut_slice(),
            &program_id,
        );
        let vm_account = create_account_info(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );
        let owner = create_account_info(
            &owner_key,
            true,
            false,
            &mut owner_lamports,
            &mut owner_data,
            &system_owner,
        );
        let fee_vault = create_account_info(
            &fee_vault_key,
            false,
            true,
            &mut fee_vault_lamports,
            &mut fee_vault_data,
            &program_id,
        );
        let system_program = create_account_info(
            &system_owner,
            false,
            false,
            &mut system_lamports,
            &mut system_data,
            &system_owner,
        );

        let accounts = [script_account, vm_account, owner, fee_vault, system_program];
        assert_eq!(
            deploy(&program_id, &accounts, &bytecode, &[], 0, 0),
            Err(ProgramError::Custom(7007))
        );
    }

    #[test]
    fn deploy_does_not_charge_fee_on_failed_overwrite() {
        let program_id = Pubkey::from([31u8; 32]);
        let script_key = Pubkey::from([32u8; 32]);
        let (vm_key, vm_bump) = canonical_vm_key(&program_id);
        let owner_key = Pubkey::from([34u8; 32]);
        let admin_key = Pubkey::from([35u8; 32]);
        let (fee_vault_key, fee_vault_bump) =
            crate::common::derive_fee_vault_pda(&program_id, 0).unwrap();

        let bytecode = minimal_valid_bytecode();
        let mut script_data = vec![0u8; ScriptAccountHeader::LEN + bytecode.len()];
        let existing_header = ScriptAccountHeader::create_from_bytecode(&bytecode, owner_key, 1, 0);
        existing_header.copy_into_account(&mut script_data).unwrap();
        script_data[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + bytecode.len()]
            .copy_from_slice(&bytecode);

        let mut vm_data = [0u8; FIVEVMState::LEN];
        {
            let vm_state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
            vm_state.initialize(admin_key, vm_bump);
            vm_state.deploy_fee_lamports = 10;
        }

        let mut script_lamports = 1_000_000;
        let mut vm_lamports = 1_000_000;
        let mut owner_lamports = 1_000;
        let mut admin_lamports = 500;
        let mut fee_vault_lamports = 0;
        let mut system_lamports = 1;
        let mut owner_data = [];
        let mut admin_data = [];
        let mut fee_vault_data = [];
        let mut system_data = [];
        let system_owner = Pubkey::default();

        let script_account = create_account_info(
            &script_key,
            false,
            true,
            &mut script_lamports,
            script_data.as_mut_slice(),
            &program_id,
        );
        let vm_account = create_account_info(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );
        let owner = create_account_info(
            &owner_key,
            true,
            true,
            &mut owner_lamports,
            &mut owner_data,
            &program_id,
        );
        let admin = create_account_info(
            &admin_key,
            false,
            true,
            &mut admin_lamports,
            &mut admin_data,
            &program_id,
        );
        let fee_vault = create_account_info(
            &fee_vault_key,
            false,
            true,
            &mut fee_vault_lamports,
            &mut fee_vault_data,
            &program_id,
        );
        let system_program = create_account_info(
            &system_owner,
            false,
            false,
            &mut system_lamports,
            &mut system_data,
            &system_owner,
        );

        let owner_before = owner.lamports();
        let admin_before = admin.lamports();

        let accounts = [script_account, vm_account, owner, admin, fee_vault, system_program];
        assert_eq!(
            deploy(&program_id, &accounts, &bytecode, &[], 0, 0),
            Err(ProgramError::Custom(7007))
        );
        assert_eq!(accounts[2].lamports(), owner_before);
        assert_eq!(accounts[3].lamports(), admin_before);
    }

    #[test]
    fn init_large_program_full_chunk_collects_deploy_fee() {
        let program_id = Pubkey::from([41u8; 32]);
        let script_key = Pubkey::from([42u8; 32]);
        let owner_key = Pubkey::from([43u8; 32]);
        let (vm_key, _vm_bump) = canonical_vm_key(&program_id);
        let admin_key = Pubkey::from([45u8; 32]);
        let (fee_vault_key, _fee_vault_bump) =
            crate::common::derive_fee_vault_pda(&program_id, 0).unwrap();

        let bytecode = minimal_valid_bytecode();
        let expected_size = bytecode.len() as u32;

        let mut script_data = vec![0u8; ScriptAccountHeader::LEN + bytecode.len()];
        let mut vm_data = [0u8; FIVEVMState::LEN];
        {
            let vm_state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
            vm_state.initialize(admin_key, 0);
            vm_state.deploy_fee_lamports = 25;
        }

        let mut script_lamports = 1_000_000;
        let mut owner_lamports = 1_000;
        let mut vm_lamports = 1_000_000;
        let mut fee_vault_lamports = 100;
        let mut system_lamports = 1;
        let mut owner_data = [];
        let mut fee_vault_data = [];
        let mut system_data = [];
        let system_owner = Pubkey::default();

        let script_account = create_account_info(
            &script_key,
            false,
            true,
            &mut script_lamports,
            script_data.as_mut_slice(),
            &program_id,
        );
        let owner = create_account_info(
            &owner_key,
            true,
            true,
            &mut owner_lamports,
            &mut owner_data,
            &program_id,
        );
        let vm_account = create_account_info(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );
        let fee_vault = create_account_info(
            &fee_vault_key,
            false,
            true,
            &mut fee_vault_lamports,
            &mut fee_vault_data,
            &program_id,
        );
        let system_program = create_account_info(
            &system_owner,
            false,
            false,
            &mut system_lamports,
            &mut system_data,
            &system_owner,
        );

        let owner_before = owner.lamports();
        let vault_before = fee_vault.lamports();
        let accounts = [script_account, owner, vm_account, fee_vault, system_program];

        assert_eq!(
            init_large_program(&program_id, &accounts, expected_size, Some(&bytecode)),
            Ok(())
        );
        assert_eq!(accounts[1].lamports(), owner_before - 25);
        assert_eq!(accounts[3].lamports(), vault_before + 25);
    }
}
