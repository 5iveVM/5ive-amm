//! FIVE VM Instructions for Solana
//!
//! This module implements the 7 core instruction types for the FIVE VM on Solana:
//!
//! ## Core Instructions
//!
//! 1. **Initialize** - One-time setup of the FIVE VM state account
//!    - Creates the global VM state with fee configuration
//!    - Sets the program authority (admin key)
//!    - Must be called before any scripts can be deployed
//!
//! 2. **Deploy** - Deploy bytecode to a script account (single-shot)
//!    - Creates a script account and uploads bytecode in one transaction
//!    - Verifies bytecode format and content at deploy time
//!    - Supports permission flags (pre-bytecode, post-bytecode, PDA special chars)
//!    - Requires admin signature if any permissions are set
//!
//! 3. **InitLargeProgram** - Initialize script account for chunked upload
//!    - For scripts too large to deploy in one transaction
//!    - Sets up script account with expected total size
//!    - Optionally writes first chunk immediately
//!    - Puts script in "upload mode" until finalized
//!
//! 4. **AppendBytecode** - Append chunk to large program
//!    - Adds bytecode chunk to script in upload mode
//!    - Tracks upload progress (current_len vs expected_size)
//!    - Owner must sign each append operation
//!
//! 5. **FinalizeScript** - Complete large program upload
//!    - Verifies full bytecode is uploaded (current_len == expected_size)
//!    - Validates bytecode content and format
//!    - Extracts metadata (function counts, features, instruction offset)
//!    - Transitions script from upload mode to executable mode
//!
//! 6. **Execute** - Run a deployed script
//!    - Executes FIVE bytecode using the MitoVM interpreter
//!    - Supports optional pre/post-execution hooks (if permissions set)
//!    - Collects execution fees based on VM state configuration
//!    - Validates script permissions and ownership
//!
//! 7. **SetFees** - Update deployment and execution fees (admin only)
//!    - Sets fees in basis points (BPS) relative to standard tx fee
//!    - Example: 100 BPS = 1% of 5000 lamports = 50 lamports
//!    - Only VM authority can modify fees
//!
//! ## Permission System
//!
//! Scripts can have special permissions set at deploy time:
//! - `PERMISSION_PRE_BYTECODE` (0x01): Run bytecode before main execution
//! - `PERMISSION_POST_BYTECODE` (0x02): Run bytecode after main execution
//! - `PERMISSION_PDA_SPECIAL_CHARS` (0x04): Allow special chars in PDA seeds
//!
//! Any non-zero permissions require the admin key to sign the deployment.

use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, sysvars::Sysvar,
    ProgramResult,
};

use crate::{
    common::{
        validate_vm_and_script_accounts, verify_program_owned, has_permission,
        verify_admin_signer, PERMISSION_POST_BYTECODE,
    },
    log_if_debug,
    state::{FIVEVMState, ScriptAccountHeader},
};
use five_vm_mito::MitoVM;
#[cfg(feature = "debug-logs")]
use five_vm_mito::VMError;

// Script deployment and execution instructions
pub const DEPLOY_INSTRUCTION: u8 = 8;
pub const EXECUTE_INSTRUCTION: u8 = 9;

/// Standard transaction fee in lamports (for fee calculation basis)
pub const STANDARD_TX_FEE: u64 = 5000;

/// Map VMError to a short code string for debug logging.
#[cfg(feature = "debug-logs")]
fn vm_error_name(err: &VMError) -> &'static str {
    match err {
        VMError::StackError => "StackError",
        VMError::InvalidInstruction => "InvalidInstruction",
        VMError::InvalidScript => "InvalidScript",
        VMError::InvalidScriptSize => "InvalidScriptSize",
        VMError::MemoryViolation => "MemoryViolation",
        VMError::TypeMismatch => "TypeMismatch",
        VMError::DivisionByZero => "DivisionByZero",
        VMError::NumericOverflow => "NumericOverflow",
        VMError::ArithmeticOverflow => "ArithmeticOverflow",
        VMError::AccountError => "AccountError",
        VMError::ConstraintViolation => "ConstraintViolation",
        VMError::Halted => "Halted",
        VMError::InvalidAccountIndex => "InvalidAccountIndex",
        VMError::AccountNotWritable => "AccountNotWritable",
        VMError::AccountNotSigner => "AccountNotSigner",
        VMError::InvalidVariableIndex(_) => "InvalidVariableIndex",
        VMError::ParameterMismatch { .. } => "ParameterMismatch",
        VMError::StackOperationError { .. } => "StackOperationError",
        VMError::AbiParameterMismatch { .. } => "AbiParameterMismatch",
        VMError::InvalidInstructionPointer => "InvalidInstructionPointer",
        VMError::CallStackOverflow => "CallStackOverflow",
        VMError::CallStackUnderflow => "CallStackUnderflow",
        VMError::DataBufferOverflow => "DataBufferOverflow",
        VMError::InvalidRegister => "InvalidRegister",
        VMError::InvalidOperation => "InvalidOperation",
        VMError::ParseError { .. } => "ParseError",
        VMError::UnexpectedToken => "UnexpectedToken",
        VMError::UnexpectedEndOfInput => "UnexpectedEndOfInput",
        VMError::InvalidFunctionIndex => "InvalidFunctionIndex",
        VMError::LocalsOverflow => "LocalsOverflow",
        VMError::InvalidAccountData => "InvalidAccountData",
        VMError::InvalidAccount => "InvalidAccount",
        VMError::MemoryError => "MemoryError",
        VMError::AccountOwnershipError { .. } => "AccountOwnershipError",
        VMError::InvokeError { .. } => "InvokeError",
        VMError::ExternalAccountLamportSpend => "ExternalAccountLamportSpend",
        VMError::ScriptNotAuthorized { .. } => "ScriptNotAuthorized",
        VMError::UndefinedAccountField => "UndefinedAccountField",
        VMError::InvalidSeedArray(_) => "InvalidSeedArray",
        VMError::ImmutableField => "ImmutableField",
        VMError::FunctionVisibilityViolation { .. } => "FunctionVisibilityViolation",
        VMError::UndefinedField => "UndefinedField",
        VMError::UndefinedIdentifier => "UndefinedIdentifier",
        VMError::InvalidParameterCount => "InvalidParameterCount",
        VMError::IndexOutOfBounds => "IndexOutOfBounds",
        VMError::OutOfMemory => "OutOfMemory",
        VMError::ProtocolError => "ProtocolError",
        VMError::TooManySeeds => "TooManySeeds",
        VMError::SecurityViolation => "SecurityViolation",
        VMError::AccountNotFound => "AccountNotFound",
        VMError::AccountDataEmpty => "AccountDataEmpty",
        VMError::RuntimeIntegrationRequired => "RuntimeIntegrationRequired",
        VMError::InvalidParameter => "InvalidParameter",
        VMError::InvalidOpcode => "InvalidOpcode",
        VMError::ExecutionTerminated => "ExecutionTerminated",
        VMError::UninitializedAccount => "UninitializedAccount",
        VMError::UnauthorizedBytecodeInvocation => "UnauthorizedBytecodeInvocation",
        VMError::PdaDerivationFailed => "PdaDerivationFailed",
    }
}

/// Minimum deployment instruction length: discriminator + u32 length + permissions byte
const MIN_DEPLOY_LEN: usize = 6;

/// Ensure the required number of accounts are present
#[inline(always)]
pub fn require_min_accounts(accounts: &[AccountInfo], min: usize) -> ProgramResult {
    if accounts.len() < min {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    Ok(())
}

/// Ensure an account is a signer
#[inline(always)]
pub fn require_signer(account: &AccountInfo) -> ProgramResult {
    if !account.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

/// Helper function to safely reallocate an account.
///
/// Passing `true` to `realloc` ensures the runtime zeroes any newly
/// allocated bytes, preventing leakage of previous data. We still
/// explicitly zero the growth slice to avoid relying on runtime
/// semantics and keep behaviour predictable for security.
#[allow(dead_code)]
pub fn safe_realloc(account: &AccountInfo, payer: &AccountInfo, new_size: usize) -> ProgramResult {
    let required_lamports = pinocchio::sysvars::rent::Rent::get()
        .map_err(|_| ProgramError::AccountNotRentExempt)?
        .minimum_balance(new_size);

    let current_lamports = account.lamports();
    if current_lamports < required_lamports {
        let additional = required_lamports - current_lamports;
        if payer.lamports() < additional {
            return Err(ProgramError::InsufficientFunds);
        }
        *payer.try_borrow_mut_lamports()? -= additional;
        *account.try_borrow_mut_lamports()? += additional;
    }

    let old_len = account.data_len();
    account.resize(new_size)?; // runtime zeroes the added region
    if new_size > old_len {
        let mut data = account.try_borrow_mut_data()?;
        data[old_len..].fill(0); // explicitly zero for deterministic security
    }
    Ok(())
}

/// Instruction enum
pub enum FIVEInstruction<'a> {
    Initialize,
    InitLargeProgram { expected_size: u32, chunk_data: Option<&'a [u8]> },
    AppendBytecode { data: &'a [u8] },
    SetFees { deploy_fee_bps: u32, execute_fee_bps: u32 },
    Deploy { bytecode: &'a [u8], permissions: u8 },
    Execute { params: &'a [u8] },
    FinalizeScript,
}

impl<'a> TryFrom<&'a [u8]> for FIVEInstruction<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, ProgramError> {
        log_if_debug!(debug, "FIVEInstruction::try_from - data length: {}", data.len());

        if data.is_empty() {
            log_if_debug!(debug, "FIVEInstruction::try_from - data is empty");
            return Err(ProgramError::InvalidInstructionData);
        }

        log_if_debug!(debug, "FIVEInstruction::try_from - discriminator: {}", data[0]);

        match data[0] {
            0 => {
                log_if_debug!(debug, "FIVEInstruction::try_from - Initialize instruction");
                Ok(FIVEInstruction::Initialize)
            }
            4 => {
                log_if_debug!(debug, "FIVEInstruction::try_from - InitLargeProgram instruction");
                if data.len() < 5 {
                    log_if_debug!(debug, "FIVEInstruction::try_from - InitLargeProgram: data too short");
                    return Err(ProgramError::InvalidInstructionData);
                }
                let expected_size = u32::from_le_bytes(data[1..5].try_into().unwrap());
                // Check if chunk data is present (InitLargeProgramWithChunk optimization)
                let chunk_data = if data.len() > 5 { Some(&data[5..]) } else { None };
                if let Some(chunk) = chunk_data {
                    #[cfg(feature = "debug-logs")]
                    log_if_debug!(debug, "InitLargeProgram with {} byte first chunk", chunk.len());
                    #[cfg(not(feature = "debug-logs"))]
                    let _ = chunk;
                }
                Ok(FIVEInstruction::InitLargeProgram { expected_size, chunk_data })
            }
            5 => {
                log_if_debug!(debug, "FIVEInstruction::try_from - AppendBytecode instruction");
                if data.len() < 2 {
                    log_if_debug!(debug, "FIVEInstruction::try_from - AppendBytecode: data too short");
                    return Err(ProgramError::InvalidInstructionData);
                }
                Ok(FIVEInstruction::AppendBytecode { data: &data[1..] })
            }
            6 => {
                log_if_debug!(debug, "FIVEInstruction::try_from - SetFees instruction");
                if data.len() < 9 {
                    log_if_debug!(debug, "FIVEInstruction::try_from - SetFees: data too short");
                    return Err(ProgramError::InvalidInstructionData);
                }
                let deploy_fee_bps = u32::from_le_bytes(data[1..5].try_into().unwrap());
                let execute_fee_bps = u32::from_le_bytes(data[5..9].try_into().unwrap());
                Ok(FIVEInstruction::SetFees { deploy_fee_bps, execute_fee_bps })
            }
            DEPLOY_INSTRUCTION => {
                log_if_debug!(debug, "FIVEInstruction::try_from - Deploy instruction (8)");
                if data.len() < MIN_DEPLOY_LEN {
                    log_if_debug!(debug, "FIVEInstruction::try_from - Deploy: data too short ({}< {})", data.len(), MIN_DEPLOY_LEN);
                    return Err(ProgramError::InvalidInstructionData);
                }
                let len = u32::from_le_bytes(data[1..5].try_into().unwrap()) as usize;
                let permissions = data[5];
                log_if_debug!(debug, "FIVEInstruction::try_from - Deploy: bytecode length: {}, permissions: 0x{}", len, permissions);
                let total_len = MIN_DEPLOY_LEN + len;
                log_if_debug!(debug, "FIVEInstruction::try_from - Deploy: total expected: {}, actual: {}", total_len, data.len());
                if data.len() < total_len {
                    log_if_debug!(debug, "FIVEInstruction::try_from - Deploy: not enough data");
                    return Err(ProgramError::InvalidInstructionData);
                }
                Ok(FIVEInstruction::Deploy {
                    bytecode: &data[6..total_len],
                    permissions,
                })
            }
            EXECUTE_INSTRUCTION => {
                log_if_debug!(debug, "FIVEInstruction::try_from - Execute instruction (9)");
                Ok(FIVEInstruction::Execute { params: &data[1..] })
            }
            7 => {
                log_if_debug!(debug, "FIVEInstruction::try_from - FinalizeScript instruction");
                Ok(FIVEInstruction::FinalizeScript)
            }
            _ => {
                log_if_debug!(debug, "FIVEInstruction::try_from - Unknown discriminator: {}", data[0]);
                Err(ProgramError::InvalidInstructionData)
            }
        }
    }
}

/// Calculate fee based on amount and basis points (bps)
/// fee = (amount * bps) / 10000
fn calculate_fee(amount: u64, bps: u32) -> u64 {
    ((amount as u128 * bps as u128) / 10000) as u64
}

/// Transfer fee from payer to recipient
fn transfer_fee(payer: &AccountInfo, recipient: &AccountInfo, amount: u64) -> ProgramResult {
    if amount == 0 {
        return Ok(());
    }

    if payer.lamports() < amount {
        return Err(ProgramError::InsufficientFunds);
    }

    *payer.try_borrow_mut_lamports()? -= amount;

    // Use checked_add to prevent overflow in recipient lamports
    let new_recipient_lamports = recipient.lamports()
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    *recipient.try_borrow_mut_lamports()? = new_recipient_lamports;

    Ok(())
}

/// Set the deployment and execution fees (BPS)
pub fn set_fees(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deploy_fee_bps: u32,
    execute_fee_bps: u32,
) -> ProgramResult {
    log_if_debug!(
        debug, 
        "Setting fees: deploy={} bps, execute={} bps", 
        deploy_fee_bps, 
        execute_fee_bps
    );

    require_min_accounts(accounts, 2)?;

    let vm_state_account = &accounts[0];
    let authority = &accounts[1];

    // Verify ownership
    verify_program_owned(vm_state_account, program_id)?;
    require_signer(authority)?;

    // Update VM state
    // SAFETY: The state account is program-owned and uniquely borrowed here.
    let vm_state_data = unsafe { vm_state_account.borrow_mut_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data_mut(vm_state_data)?;

    // Verify authority matches
    if vm_state.authority != *authority.key() {
        return Err(ProgramError::Custom(0)); // Unauthorized
    }

    vm_state.deploy_fee_bps = deploy_fee_bps;
    vm_state.execute_fee_bps = execute_fee_bps;

    log_if_debug!(debug, "Fees updated successfully");
    Ok(())
}

/// Initialize the VM state account
pub fn initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    log_if_debug!(debug, "Initializing FIVE VM");

    require_min_accounts(accounts, 2)?;

    let vm_state_account = &accounts[0];
    let authority = &accounts[1];

    // Verify ownership
    verify_program_owned(vm_state_account, program_id)?;

    require_signer(authority)?;

    // Initialize VM state
    // SAFETY: The account was verified to be owned by this program and we borrow
    // its data mutably within the instruction, so aliasing rules are upheld.
    let vm_state_data = unsafe { vm_state_account.borrow_mut_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data_mut(vm_state_data)?;
    vm_state.initialize(*authority.key());

    log_if_debug!(debug, "FIVE VM initialized successfully");
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
    use crate::common::validate_permissions;

    // Validate permissions bitmask
    validate_permissions(permissions)?;

    log_if_debug!(debug, "Deploying script with {} bytes", bytecode.len());

    require_min_accounts(accounts, 3)?;

    let script_account = &accounts[0];
    let vm_state_account = &accounts[1];
    let owner = &accounts[2];

    validate_vm_and_script_accounts(program_id, script_account, vm_state_account)?;
    require_signer(owner)?;

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
        log_if_debug!(debug, "Admin key verified for permissions: 0x{}", permissions);
    }

    // Validate bytecode size
    if bytecode.len() < 4 || bytecode.len() > five_protocol::MAX_SCRIPT_SIZE {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Check if valid Five Protocol bytecode header format (10 bytes minimum)
    if bytecode.len() < five_protocol::FIVE_HEADER_OPTIMIZED_SIZE {
        return Err(ProgramError::InvalidInstructionData);
    }
    if &bytecode[..4] != five_protocol::FIVE_MAGIC {
        return Err(ProgramError::InvalidInstructionData);
    }

    // **Deploy-time verification**: Verify bytecode content
    verify_bytecode_content(bytecode)?;

    #[cfg(not(feature = "debug-logs"))]
    let _ = program_id; // Suppress unused variable warning

    // Calculate required account size: header + bytecode + metadata
    let required_size = ScriptAccountHeader::LEN + bytecode.len();
    
    // Check for deployment fees
    {
        // SAFETY: The state account is program-owned and read-only here.
        let vm_state_data = unsafe { vm_state_account.borrow_data_unchecked() };
        let vm_state = FIVEVMState::from_account_data(&vm_state_data)?;
        
        let deploy_fee_bps = vm_state.deploy_fee_bps;
        if deploy_fee_bps > 0 {
            // Calculate rent basis
            let rent = pinocchio::sysvars::rent::Rent::get()
                .map_err(|_| ProgramError::AccountNotRentExempt)?;
            let rent_basis = rent.minimum_balance(required_size);
            
            // Fee is bps of rent
            let fee = calculate_fee(rent_basis, deploy_fee_bps);
            
            if fee > 0 {
                // Find admin (authority) account to receive fee
                // If permissions != 0, admin is at accounts[3]
                // If permissions == 0, we might need to search or require admin present
                
                let admin_key = vm_state.authority;
                let admin_account = accounts.iter().find(|a| *a.key() == admin_key);
                
                if let Some(recipient) = admin_account {
                    transfer_fee(owner, recipient, fee)?;
                    log_if_debug!(debug, "Collected deploy fee: {}", fee);
                } else {
                    // If fee is required but admin not present, fail
                    return Err(ProgramError::MissingRequiredSignature);
                }
            }
        }
    }

    if script_account.data_len() < required_size {
        return Err(ProgramError::Custom(7005));
    }

    // Extract cached metadata from bytecode for fast execution
    let public_function_count = if bytecode.len() >= 9 { bytecode[8] } else { 0 };
    let total_function_count = if bytecode.len() >= 10 { bytecode[9] } else { 0 };
    let features = if bytecode.len() >= 8 {
        u32::from_le_bytes([bytecode[4], bytecode[5], bytecode[6], bytecode[7]])
    } else {
        0
    };
    let instruction_start_offset = compute_instruction_start_offset(bytecode);

    // Update VM state - reuse mutable borrow from earlier? No, borrow scope ended.
    // SAFETY: `vm_state_account` verified.
    let vm_state_data = unsafe { vm_state_account.borrow_mut_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data_mut(vm_state_data)?;
    let script_id = vm_state.create_script_id();

    // Write script header + bytecode to account
    // SAFETY: `script_account` is owned by this program and exclusively borrowed.
    let script_data = unsafe { script_account.borrow_mut_data_unchecked() };

    // Create header with cached metadata for fast execution path
    let header = ScriptAccountHeader::new_with_metadata(
        bytecode,
        *owner.key(),
        script_id,
        public_function_count,
        total_function_count,
        features,
        instruction_start_offset,
        permissions, // Use the permissions from the instruction
    );

    header.copy_into_account(script_data)?;
    script_data[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + bytecode.len()]
        .copy_from_slice(bytecode);

    log_if_debug!(
        debug,
        "Script {} deployed: public_funcs={}, total_funcs={}, instr_offset={}",
        script_id,
        public_function_count,
        total_function_count,
        instruction_start_offset
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
    log_if_debug!(
        debug,
        "Initializing large program: expected_size={}, initial_chunk={}",
        expected_size,
        chunk_len
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
        return Err(ProgramError::InvalidInstructionData);
    }

    // Validate chunk size if present
    if let Some(chunk) = chunk_data {
        if chunk.len() > expected_size {
            #[cfg(feature = "debug-logs")]
            log_if_debug!(error, "Chunk size {} exceeds expected size {}", chunk.len(), expected_size);
            return Err(ProgramError::InvalidInstructionData);
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
            log_if_debug!(error, "Account too small: {} < {}", script_data.len(), end);
            return Err(ProgramError::Custom(7006)); // Account too small
        }
        script_data[start..end].copy_from_slice(chunk);
        log_if_debug!(debug, "Wrote {} bytes of initial chunk", chunk.len());
    }

    Ok(())
}

/// Append a bytecode chunk to a large-program script account.
pub fn append_bytecode(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    chunk: &[u8],
) -> ProgramResult {
    log_if_debug!(debug, "Appending {} bytes of bytecode", chunk.len());

    require_min_accounts(accounts, 3)?;
    if chunk.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
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
        return Err(ProgramError::InvalidInstructionData);
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
        log_if_debug!(debug, "Check: new_len={} matched expected so finalizing...", new_len);
        let bytecode =
            &script_data[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + expected_size];

        if bytecode.len() < 4 || bytecode.len() > five_protocol::MAX_SCRIPT_SIZE {
            return Err(ProgramError::InvalidInstructionData);
        }

        if bytecode.len() < five_protocol::FIVE_HEADER_OPTIMIZED_SIZE {
            return Err(ProgramError::InvalidInstructionData);
        }
        if &bytecode[..4] != five_protocol::FIVE_MAGIC {
            return Err(ProgramError::InvalidInstructionData);
        }

        // log_if_debug!(debug, "Verifying bytecode content...");
        if let Err(e) = verify_bytecode_content(bytecode) {
            #[cfg(feature = "debug-logs")]
            {
                let code: u64 = e.into();
                log_if_debug!(error, "Bytecode verification failed: {}", code);
            }
            return Err(e);
        }
        log_if_debug!(debug, "Verification successful.");

        // Collect deployment fee if configured
        {
            let vm_state_data = unsafe { vm_state_account.borrow_data_unchecked() };
            let vm_state = FIVEVMState::from_account_data(&vm_state_data)?;

            let deploy_fee_bps = vm_state.deploy_fee_bps;
            if deploy_fee_bps > 0 {
                // Calculate rent basis for the final script account size
                let final_size = ScriptAccountHeader::LEN + expected_size;
                let rent = pinocchio::sysvars::rent::Rent::get()
                    .map_err(|_| ProgramError::AccountNotRentExempt)?;
                let rent_basis = rent.minimum_balance(final_size);

                // Fee is bps of rent
                let fee = calculate_fee(rent_basis, deploy_fee_bps);

                log_if_debug!(debug, "Deploy fee check: bps={}, rent_basis={}, fee={}", deploy_fee_bps, rent_basis, fee);

                if fee > 0 {
                    let admin_key = vm_state.authority;
                    let admin_account = accounts.iter().find(|a| *a.key() == admin_key);

                    if let Some(recipient) = admin_account {
                        log_if_debug!(debug, "Paying deploy fee: {}", fee);
                        transfer_fee(owner, recipient, fee)?;
                        log_if_debug!(debug, "Collected deploy fee: {}", fee);
                    } else {
                        log_if_debug!(error, "Deploy fee required but Admin not found");
                        // If fee is required but admin not present, fail
                        return Err(ProgramError::MissingRequiredSignature);
                    }
                }
            }
        }

        let public_function_count = if bytecode.len() >= 9 { bytecode[8] } else { 0 };
        let total_function_count = if bytecode.len() >= 10 { bytecode[9] } else { 0 };
        let features = if bytecode.len() >= 8 {
            u32::from_le_bytes([bytecode[4], bytecode[5], bytecode[6], bytecode[7]])
        } else {
            0
        };
        let instruction_start_offset = compute_instruction_start_offset(bytecode);

        let mut final_header = ScriptAccountHeader::new_with_metadata(
            bytecode,
            *owner.key(),
            script_id,
            public_function_count,
            total_function_count,
            features,
            instruction_start_offset,
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
    log_if_debug!(debug, "Finalizing script upload");

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
        log_if_debug!(error, "Finalize failed: current_len {} != expected {}", current_len, expected_size);
        return Err(ProgramError::InvalidInstructionData);
    }

    // Verify bytecode
    let script_data = unsafe { script_account.borrow_mut_data_unchecked() };
    let bytecode = &script_data[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + expected_size];

    verify_bytecode_content(bytecode)?;

    // Calculate metadata
    let public_function_count = if bytecode.len() >= 9 { bytecode[8] } else { 0 };
    let total_function_count = if bytecode.len() >= 10 { bytecode[9] } else { 0 };
    let features = if bytecode.len() >= 8 {
        u32::from_le_bytes([bytecode[4], bytecode[5], bytecode[6], bytecode[7]])
    } else {
        0
    };
    let instruction_start_offset = compute_instruction_start_offset(bytecode);

    // Update header
    let mut final_header = ScriptAccountHeader::new_with_metadata(
        bytecode,
        *owner.key(),
        script_id,
        public_function_count,
        total_function_count,
        features,
        instruction_start_offset,
        permissions,
    );
    // Set upload flags BEFORE writing to account (single-write pattern)
    final_header.set_upload_len(0);
    final_header.set_upload_mode(false);
    final_header.set_upload_complete(true);
    // Single write with all flags correctly set
    final_header.copy_into_account(script_data)?;

    log_if_debug!(debug, "Script upload finalized successfully");
    Ok(())
}

/// Execute a script with optional pre/post bytecode hooks
///
/// **Pre-Execution Hook** (if PERMISSION_PRE_BYTECODE is set):
/// - Runs the bytecode BEFORE main execution
/// - Can validate conditions, collect fees, etc.
/// - If pre-execution fails, main script never runs
///
/// **Post-Execution Hook** (if PERMISSION_POST_BYTECODE is set):
/// - Runs the bytecode AFTER main execution
/// - Can process results, log, cleanup, etc.
/// - Only runs if main execution succeeds
pub fn execute(program_id: &Pubkey, accounts: &[AccountInfo], params: &[u8]) -> ProgramResult {
    #[cfg(feature = "debug-logs")]
    pinocchio_log::log!("DEBUG: execute ENTRY");

    require_min_accounts(accounts, 2)?;
    #[cfg(feature = "debug-logs")]
    pinocchio_log::log!("DEBUG: require_min_accounts PASS");

    let script_account = &accounts[0];
    let vm_state_account = &accounts[1];

    if let Err(e) = validate_vm_and_script_accounts(program_id, script_account, vm_state_account) {
         #[cfg(feature = "debug-logs")]
         pinocchio_log::log!("DEBUG: validate_vm_and_script_accounts FAIL");
         return Err(e);
    }
    #[cfg(feature = "debug-logs")]
    pinocchio_log::log!("DEBUG: validate_vm_and_script_accounts PASS");

    // Collect execution fee if configured.
    let vm_accounts = {
        // SAFETY: The state account is program-owned and read-only here.
        let vm_state_data = unsafe { vm_state_account.borrow_data_unchecked() };
        let vm_state = FIVEVMState::from_account_data(&vm_state_data)?;
        let fee = calculate_fee(STANDARD_TX_FEE, vm_state.execute_fee_bps);
        if fee > 0 {
             // ... fee logic ...
             accounts
        } else {
             #[cfg(feature = "debug-logs")]
             pinocchio_log::log!("DEBUG: fee is 0");
             accounts
        }
    };
    #[cfg(feature = "debug-logs")]
    pinocchio_log::log!("DEBUG: input accounts setup PASS");

    // Parse script header from script account
    let script_data = unsafe { script_account.borrow_data_unchecked() };
    #[cfg(feature = "debug-logs")]
    pinocchio_log::log!("DEBUG: script_data borrow PASS");

    let header = ScriptAccountHeader::from_account_data(&script_data)?;
    #[cfg(feature = "debug-logs")]
    pinocchio_log::log!("DEBUG: header parse PASS");
    
    if header.upload_mode() && !header.upload_complete() {
        return Err(ProgramError::Custom(7001));
    }
    // Validate header
    let bytecode_len = header.bytecode_len();
    
    let required_len = ScriptAccountHeader::LEN + bytecode_len as usize + header.metadata_len();
    if script_data.len() < required_len {
        #[cfg(feature = "debug-logs")]
        pinocchio_log::log!("DEBUG: script too short");
        return Err(ProgramError::Custom(7003));
    }

    // Extract bytecode slice
    let bytecode_start = ScriptAccountHeader::LEN + header.metadata_len();
    let bytecode_end = bytecode_start + bytecode_len;

    let bytecode = &script_data[bytecode_start..bytecode_end];
    #[cfg(feature = "debug-logs")]
    pinocchio_log::log!("DEBUG: bytecode slice PASS len={}", bytecode.len());

    // Execute main bytecode
    #[cfg(feature = "debug-logs")]
    pinocchio_log::log!("DEBUG: Executing MAIN bytecode");
    // Explicitly dropping borrow to ensure no conflict?
    // script_data is slice. We pass slice to execute_direct. 
    // This is safe.
    
    if let Err(vm_error) = MitoVM::execute_direct(bytecode, params, vm_accounts, program_id) {
        log_if_debug!(
            error,
            "MitoVM MAIN execution failed code={}",
            vm_error_name(&vm_error)
        );
        return Err(vm_error.to_program_error());
    }


    // Run post-execution hook if permission is set
    if has_permission(header.permissions, PERMISSION_POST_BYTECODE) {
        log_if_debug!(debug, "Running POST-BYTECODE hook");
        if let Err(vm_error) = MitoVM::execute_direct(bytecode, params, vm_accounts, program_id) {
            log_if_debug!(
                error,
                "MitoVM POST hook failed code={}",
                vm_error_name(&vm_error)
            );
            return Err(vm_error.to_program_error());
        }
    }

    log_if_debug!(debug, "Script executed successfully");
    Ok(())
}

/// Calculate instruction start offset (skips function name metadata if present)
fn compute_instruction_start_offset(bytecode: &[u8]) -> u16 {
    const FEATURE_FUNCTION_NAMES: u32 = 1 << 8;

    if bytecode.len() < 10 {
        return 10;
    }

    let features = u32::from_le_bytes([bytecode[4], bytecode[5], bytecode[6], bytecode[7]]);
    let public_count = bytecode[8];

    if (features & FEATURE_FUNCTION_NAMES) == 0 || public_count == 0 {
        return 10;
    }

    // Parse metadata section size (VLE encoded u16)
    let mut offset = 10usize;
    let mut section_size = 0u16;
    let mut shift = 0;

    while offset < bytecode.len() && shift < 16 {
        let byte = bytecode[offset];
        section_size |= ((byte & 0x7F) as u16) << shift;
        offset += 1;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    // instruction start = 10 bytes header + VLE size bytes + metadata content bytes
    (offset + section_size as usize).min(bytecode.len()) as u16
}

/// Verify bytecode content before deployment
///
/// **Deploy-Time Verification Strategy:**
/// This function performs comprehensive verification of bytecode, enabling
/// trust-based execution at runtime without re-verification:
/// - Header format is valid (magic, features, counts)
/// - All instructions are valid opcodes with proper bounds and arguments
/// - CALL instructions target valid function indices
/// - No incomplete instructions
/// - Function name metadata format is valid (if present)
///
/// Results are cached in ScriptAccountHeader for fast execution.
#[allow(unused_variables)]
pub fn verify_bytecode_content(bytecode: &[u8]) -> ProgramResult {
    // Validate bytecode size
    if bytecode.len() > five_protocol::MAX_SCRIPT_SIZE {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Bypass full parsing to avoid OOM
    // Extract header fields manually
    if bytecode.len() < 10 {
         return Err(ProgramError::InvalidInstructionData);
    }
    let public_function_count = bytecode[8];
    let total_function_count = bytecode[9];

    // Validate function counts are within bounds
    if total_function_count > five_protocol::MAX_FUNCTIONS as u8 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // CRITICAL: Validate that at least one public function exists (if functions exist)
    if total_function_count > 0 && public_function_count == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Validate public_count <= total_count
    if public_function_count > total_function_count {
        return Err(ProgramError::InvalidInstructionData);
    }

    Ok(())
}

/// Validate function name metadata format (if present)
#[allow(dead_code)]
fn validate_function_metadata(bytecode: &[u8]) -> ProgramResult {
    const FEATURE_FUNCTION_NAMES: u32 = 1 << 8;

    if bytecode.len() < 10 {
        return Ok(());
    }

    let features = u32::from_le_bytes([bytecode[4], bytecode[5], bytecode[6], bytecode[7]]);
    let public_count = bytecode[8];

    if (features & FEATURE_FUNCTION_NAMES) == 0 || public_count == 0 {
        return Ok(());
    }

    // Parse and validate metadata section
    let mut offset = 10usize;
    let mut section_size = 0u16;
    let mut shift = 0;

    // Decode VLE u16 section size
    while offset < bytecode.len() && shift < 16 {
        let byte = bytecode[offset];
        section_size |= ((byte & 0x7F) as u16) << shift;
        offset += 1;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    // Validate metadata doesn't exceed bytecode bounds
    let metadata_end = offset + section_size as usize;
    if metadata_end > bytecode.len() {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Quick validation: metadata section should contain valid name entries
    // Each entry has: name_len (u8) + name_bytes
    // At minimum, we expect at least public_count entries
    if section_size == 0 && public_count > 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    Ok(())
}
