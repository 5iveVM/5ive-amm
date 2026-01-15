use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    common::{
        validate_vm_and_script_accounts, verify_program_owned, validate_permissions,
        verify_admin_signer,
    },
    debug_log,
    state::{FIVEVMState, ScriptAccountHeader},
};

use super::{
    fees::{collect_deploy_fee},
    verify::{verify_bytecode_content},
    require_min_accounts, require_signer, safe_realloc,
};

/// Minimum deployment instruction length: discriminator + u32 length + permissions byte
pub const MIN_DEPLOY_LEN: usize = 6;

/// Initialize the VM state account
pub fn initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    debug_log!("Initializing FIVE VM");

    require_min_accounts(accounts, 2)?;

    let vm_state_account = &accounts[0];
    let authority = &accounts[1];

    // Verify ownership
    verify_program_owned(vm_state_account, program_id)?;

    require_signer(authority)?;

    // Initialize VM state
    // SAFETY: Account verified owned by program, mutable borrow is safe.
    let vm_state_data = unsafe { vm_state_account.borrow_mut_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data_mut(vm_state_data)?;
    vm_state.initialize(*authority.key());

    debug_log!("FIVE VM initialized successfully");
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
pub fn deploy(program_id: &Pubkey, accounts: &[AccountInfo], bytecode: &[u8], permissions: u8) -> ProgramResult {

    // Validate permissions bitmask
    validate_permissions(permissions)?;

    debug_log!("Deploying script with {} bytes", bytecode.len());
    debug_log!("FIVE: deploy start bytes={}", bytecode.len());

    require_min_accounts(accounts, 3)?;
    debug_log!("FIVE: accounts OK");

    let script_account = &accounts[0];
    let vm_state_account = &accounts[1];
    let owner = &accounts[2];

    debug_log!("FIVE: calling validate_vm_and_script");
    validate_vm_and_script_accounts(program_id, script_account, vm_state_account)?;
    debug_log!("FIVE: validate OK");
    require_signer(owner)?;
    debug_log!("FIVE: signer OK");

    // If any permissions are set, require admin key (VM authority) signature
    if permissions != 0 {
        // Get the admin key from VM state authority
        let vm_state_data = unsafe { vm_state_account.borrow_data_unchecked() };
        let vm_state = FIVEVMState::from_account_data(&vm_state_data)?;
        let admin_key = vm_state.authority;

        // Admin account must be present and be the signer when special permissions are used
        require_min_accounts(accounts, 4)?;
        let admin_account = &accounts[3];
        verify_admin_signer(admin_account, &admin_key)?;
        debug_log!("Admin key verified for permissions: 0x{}", permissions);
    }

    debug_log!("FIVE: size check");
    // Validate bytecode size
    if bytecode.len() < 4 || bytecode.len() > five_protocol::MAX_SCRIPT_SIZE {
        return Err(ProgramError::Custom(8001));
    }
    debug_log!("FIVE: size OK");

    // Check if valid Five Protocol bytecode header format (10 bytes minimum)
    if bytecode.len() < five_protocol::FIVE_HEADER_OPTIMIZED_SIZE {
        return Err(ProgramError::Custom(8002));
    }
    if &bytecode[..4] != five_protocol::FIVE_MAGIC {
        return Err(ProgramError::Custom(8003));
    }
    debug_log!("FIVE: header OK, calling verify");

    // Verify bytecode content
    verify_bytecode_content(bytecode)?;

    #[cfg(not(feature = "debug-logs"))]
    let _ = program_id; // Suppress unused variable warning

    // Calculate required account size: header + bytecode + metadata
    let required_size = ScriptAccountHeader::LEN + bytecode.len();

    // Check for deployment fees
    collect_deploy_fee(vm_state_account, accounts, owner, required_size)?;

    if script_account.data_len() < required_size {
        return Err(ProgramError::Custom(7005));
    }

    // SAFETY: `vm_state_account` verified.
    let vm_state_data = unsafe { vm_state_account.borrow_mut_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data_mut(vm_state_data)?;
    let script_id = vm_state.create_script_id();

    // Write script header + bytecode to account
    // SAFETY: `script_account` is owned by this program and exclusively borrowed.
    let script_data = unsafe { script_account.borrow_mut_data_unchecked() };

    // Create header with cached metadata
    let header = ScriptAccountHeader::create_from_bytecode(
        bytecode,
        *owner.key(),
        script_id,
        permissions, // Use the permissions from the instruction
    );

    header.copy_into_account(script_data)?;
    script_data[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + bytecode.len()]
        .copy_from_slice(bytecode);

    debug_log!(
        "Script {} deployed: header_created",
        script_id
    );
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
    debug_log!(
        "InitLargeProgram: expected={}, chunk={}",
        expected_size, chunk_len
    );

    require_min_accounts(accounts, 3)?;

    let script_account = &accounts[0];
    let owner = &accounts[1];
    let vm_state_account = &accounts[2];

    // Verify VM state is owned by this program and initialized
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
            #[cfg(feature = "debug-logs")]
            debug_log!("Chunk size {} exceeds expected size {}", chunk.len(), expected_size);
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
            debug_log!("Account too small: {} < {}", script_data.len(), end);
            return Err(ProgramError::Custom(7006)); // Account too small
        }
        script_data[start..end].copy_from_slice(chunk);
        debug_log!("Wrote {} bytes of initial chunk", chunk.len());
    }

    Ok(())
}

/// Append a bytecode chunk to a large-program script account.
pub fn append_bytecode(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    chunk: &[u8],
) -> ProgramResult {
    debug_log!("Appending {} bytes of bytecode", chunk.len());

    require_min_accounts(accounts, 3)?;
    if chunk.is_empty() {
        return Err(ProgramError::Custom(8201)); // Empty chunk
    }

    let script_account = &accounts[0];
    let owner = &accounts[1];
    let vm_state_account = &accounts[2];

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
        debug_log!("Check: new_len={} matched expected so finalizing...", new_len);
        let bytecode =
            &script_data[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + expected_size];

        if bytecode.len() < 4 || bytecode.len() > five_protocol::MAX_SCRIPT_SIZE {
            return Err(ProgramError::Custom(8203)); // Invalid bytecode size
        }

        if bytecode.len() < five_protocol::FIVE_HEADER_OPTIMIZED_SIZE {
            return Err(ProgramError::Custom(8204)); // Header too small
        }
        if &bytecode[..4] != five_protocol::FIVE_MAGIC {
            return Err(ProgramError::Custom(8205)); // Invalid magic bytes
        }

        // debug_log!("Verifying bytecode content...");
        if let Err(e) = verify_bytecode_content(bytecode) {
            #[cfg(feature = "debug-logs")]
            {
                let code: u64 = e.into();
                debug_log!("Bytecode verification failed: {}", code);
            }
            return Err(e);
        }
        debug_log!("Verification successful.");

        // Collect deployment fee if configured
        {
            let final_size = ScriptAccountHeader::LEN + expected_size;
            collect_deploy_fee(vm_state_account, accounts, owner, final_size)?;
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
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    debug_log!("Finalizing script upload");

    require_min_accounts(accounts, 2)?;
    let script_account = &accounts[0];
    let owner = &accounts[1];

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
        debug_log!("Finalize failed: current_len {} != expected {}", current_len, expected_size);
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

    debug_log!("Script upload finalized successfully");
    Ok(())
}
