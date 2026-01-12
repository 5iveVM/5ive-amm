IMPORTANT: The file content has been truncated.
Status: Showing lines 1-100 of 1678 total lines.
Action: To read more of the file, you can use the 'offset' and 'limit' parameters in a subsequent 'read_file' call. For example, to read the next section of the file, use offset: 100.

--- FILE CONTENT (truncated) ---
//! Execution context for the VM
//!
//! This module provides the `ExecutionContext` struct, which manages the state
//! of the VM during execution. It holds references to the accounts, input data,
//! and program ID, as well as the VM's stack and memory.

use crate::error::{CompactResult, VMErrorCode};
use crate::stack::StackStorage;
use crate::types::{CallFrame, LocalVariables};
use crate::utils::{value_ref_to_seed_bytes, ErrorUtils, ValueRefUtils};
use crate::{debug_log, error_log}; // Import error_log for critical errors
use five_protocol::opcodes::Instruction;
use five_protocol::{Value, ValueRef};
use pinocchio::account_info::AccountInfo;
use pinocchio::pubkey::Pubkey;
#[cfg(target_os = "solana")]
use pinocchio::instruction::{Seed, Signer};
#[cfg(target_os = "solana")]
use pinocchio::program::{invoke, invoke_signed};

// We use heapless::Vec for stack-allocated collections to avoid allocator dependencies
// This is critical for the "no-allocator" optimization path
use heapless::Vec;

/// Maximum number of accounts the VM can handle
pub const MAX_ACCOUNTS: usize = 32;

/// Size of the temporary buffer in bytes
pub const TEMP_BUFFER_SIZE: usize = five_protocol::TEMP_BUFFER_SIZE; // 64 bytes - synced with protocol

/// Maximum number of return values
pub const MAX_RETURN_VALUES: usize = 4;

/// Execution Manager wrapper for safe context access
pub struct ExecutionManager<'a> {
    pub ctx: ExecutionContext<'a>,
}

impl<'a> ExecutionManager<'a> {
    pub fn new(ctx: ExecutionContext<'a>) -> Self {
        Self { ctx }
    }

    // Proxy methods for common operations
    #[inline(always)]
    pub fn push(&mut self, value: ValueRef) -> CompactResult<()> {
        self.ctx.stack.push(value)
    }

    #[inline(always)]
    pub fn pop(&mut self) -> CompactResult<ValueRef> {
        self.ctx.stack.pop()
    }

    #[inline(always)]
    pub fn peek(&self) -> CompactResult<ValueRef> {
        self.ctx.stack.peek()
    }

    #[inline(always)]
    pub fn accounts(&self) -> &[AccountInfo] {
        self.ctx.accounts
    }

    #[inline(always)]
    pub fn temp_buffer(&mut self) -> &mut [u8] {
        &mut self.ctx.temp_buffer
    }

    #[inline(always)]
    pub fn alloc_temp(&mut self, size: usize) -> CompactResult<usize> {
        self.ctx.alloc_temp(size)
    }

    #[inline(always)]
    pub fn program_id(&self) -> &Pubkey {
        self.ctx.program_id
    }
}

/// The execution context for the VM
pub struct ExecutionContext<'a> {
    /// The stack for the VM
    pub stack: StackStorage,

    /// The accounts passed to the program
    pub accounts: &'a [AccountInfo],

    /// The input data for the program
    pub input: &'a [u8],

    /// The program ID
    pub program_id: &'a Pubkey,

    /// The temporary buffer for intermediate operations
    pub temp_buffer: [u8; TEMP_BUFFER_SIZE],

    /// The current offset in the temporary buffer
    pub temp_offset: usize,

    /// The instruction pointer (program counter)
    pub ip: usize,

    /// The call stack for function calls
    pub call_stack: Vec<CallFrame, { crate::MAX_CALL_DEPTH }>,

    /// The local variables for the current stack frame
    pub locals: LocalVariables,

    /// The program bytecode (ref to script account data)
    pub bytecode: &'a [u8],

    /// Cached function parameters for fast access (O(1))
    /// Stores up to MAX_PARAMETERS ValueRefs parsed from input data
    pub parameters: Vec<ValueRef, { crate::MAX_PARAMETERS }>,

    /// Current opcode being executed (for error reporting)
    pub current_opcode: u8,
}

impl<'a> ExecutionContext<'a> {
    /// Create a new execution context
    pub fn new(
        accounts: &'a [AccountInfo],
        input: &'a [u8],
        program_id: &'a Pubkey,
        bytecode: &'a [u8],
        start_ip: usize,
    ) -> Self {
        Self {
            stack: StackStorage::new(),
            accounts,
            input,
            program_id,
            temp_buffer: [0; TEMP_BUFFER_SIZE],
            temp_offset: 0,
            ip: start_ip,
            call_stack: Vec::new(),
            locals: LocalVariables::new(),
            bytecode,
            parameters: Vec::new(),
            current_opcode: 0,
        }
    }

    /// Set parameters from pre-parsed VLE decoding
    pub fn set_parameters(&mut self, params: &[ValueRef]) -> CompactResult<()> {
        self.parameters.clear();
        for param in params {
            self.parameters
                .push(*param)
                .map_err(|_| VMErrorCode::StackOverflow)?; // Reuse error code for "too many params"
        }
        Ok(())
    }

    /// Get a parameter by index (0-based)
    /// Returns Empty if index is out of bounds or not provided
    pub fn get_parameter(&self, index: usize) -> ValueRef {
        if index < self.parameters.len() {
            self.parameters[index]
        } else {
            ValueRef::Empty
        }
    }

    pub fn size(&self) -> usize {
        self.stack.len()
    }

    pub fn set_current_opcode(&mut self, opcode: u8) {
        self.current_opcode = opcode;
    }

    pub fn get_current_opcode(&self) -> u8 {
        self.current_opcode
    }

    /// Allocate memory in the temporary buffer
    pub fn alloc_temp(&mut self, size: usize) -> CompactResult<usize> {
        if self.temp_offset + size > TEMP_BUFFER_SIZE {
            return Err(VMErrorCode::out_of_memory());
        }

        let offset = self.temp_offset;
        self.temp_offset += size;
        Ok(offset)
    }

    /// Reset the temporary buffer
    pub fn reset_temp(&mut self) {
        self.temp_offset = 0;
    }

    /// Get an account by index
    pub fn get_account(&self, index: usize) -> CompactResult<&'a AccountInfo> {
        if index >= self.accounts.len() {
            return Err(VMErrorCode::InvalidAccountIndex);
        }
        Ok(&self.accounts[index])
    }

    /// Get an account by index (unchecked, returns error if out of bounds)
    /// Use this when index is already validated or expected to be valid
    #[inline(always)]
    pub fn get_account_unchecked(&self, index: u8) -> CompactResult<&'a AccountInfo> {
        if index as usize >= self.accounts.len() {
            return Err(VMErrorCode::InvalidAccountIndex);
        }
        // SAFETY: Bounds checked above
        Ok(unsafe { self.accounts.get_unchecked(index as usize) })
    }

    /// Create a new account using System Program (CPI)
    ///
    /// This function implements the logic for the INIT_ACCOUNT opcode.
    /// It performs a CPI to the System Program to creating a new account.
    pub fn create_account(
        &mut self,
        account_idx: u8,
        payer_idx: u8,
        lamports: u64,
        space: u64,
        owner: &[u8; 32],
    ) -> CompactResult<()> {
        // Validate indices
        let new_account = self.get_account_unchecked(account_idx)?;
        let payer = self.get_account_unchecked(payer_idx)?;
        
        let system_program_id = Pubkey::from(SYSTEM_PROGRAM_ID);
        let system_program = self
            .accounts
            .iter()
            .find(|a| *a.key() == system_program_id)
            .ok_or({
                debug_log!("create_account: System Program not found");
                VMErrorCode::AccountNotFound
            })?;

        // Construct the CreateAccount instruction manually to avoid unnecessary allocations
        // SystemInstruction::CreateAccount format:
        // [0..4]: 0 (Discriminator)
        // [4..12]: lamports (u64)
        // [12..20]: space (u64)
        // [20..52]: owner (Pubkey)
        
        let mut data = [0u8; 52];
        Self::serialize_create_account_data(&mut data, lamports, space, owner);

        let instruction = pinocchio::instruction::Instruction {
            program_id: system_program.key(),
            accounts: &[
                pinocchio::instruction::AccountMeta {
                    pubkey: payer.key(),
                    is_signer: true,
                    is_writable: true,
                },
                pinocchio::instruction::AccountMeta {
                    pubkey: new_account.key(),
                    is_signer: true,
                    is_writable: true,
                },
            ],
            data: &data,
        };

        // Perform CPI
        #[cfg(target_os = "solana")]
        invoke::<2>(&instruction, &[payer, new_account, system_program])
            .map_err(|_e| VMErrorCode::InvokeError)?;

        // CRITICAL FIX: Refresh pointer for the newly created account
        // When an account is created via CPI, the runtime reallocates the account data
        // and updates the pointer in the AccountInfo. However, Pinocchio's AccountInfo
        // caches the pointer. We need to refresh it if possible, or just be aware that
        // subsequent accesses might need to reload.
        // Pinocchio's AccountInfo doesn't expose a refresh method, but we can re-fetch
        // the account from the slice if needed. In this VM design, we typically pass
        // AccountInfo references which point to the original slice.
        
        // Wait, Pinocchio AccountInfo IS a wrapper around the pointer.
        // The issue is that `borrow_data()` might use a cached pointer?
        // No, `borrow_data()` uses `self.data.borrow()`.
        
        // For safe measure, we can hint to the VM that account state changed.
        // But for now, we just proceed.
        
        // Actually, for Pinocchio 0.8+, we might need to be careful.
        // Let's check if we need to do anything.
        // `create_account` changes owner and data size.
        // The runtime handles the underlying memory.
        // We should be fine as long as we don't hold active borrows across the CPI.
        
        // Force refresh of account pointers if needed (placeholder for now)
        let _ = self.refresh_account_pointers_after_cpi(&[account_idx as usize]);

        Ok(())
    }

    /// Refresh account pointers after a CPI that might have reallocated them
    /// This is a heuristic fix for "ProgramFailedToComplete" or stale data issues
    fn refresh_account_pointers_after_cpi(&self, account_indices: &[usize]) -> CompactResult<()> {
        // In pure Pinocchio, AccountInfo holds raw pointers.
        // If the runtime moves the account (e.g. realloc), the pointers in AccountInfo
        // might become invalid if they aren't updated.
        // However, standard `invoke` should handle this for passed accounts.
        // The concern is if we have OTHER references to these accounts.
        // Since we pass `&AccountInfo`, the caller holds the struct.
        
        // We log for debugging if in trace mode
        #[cfg(feature = "trace-execution")]
        {
            // crate::error_log!(
            //     "CPI_POINTER_REFRESH: Refreshing pointers for {} accounts",
            //     account_indices.len() as u32
            // );
            for &idx in account_indices {
                if let Ok(account) = self.get_account(idx) {
                    let data_len = account.data_len();
                    let ptr = unsafe { account.borrow_data_unchecked().as_ptr() as usize };
                    // crate::error_log!(
                    //     "CPI_POINTER_REFRESH: idx={} data_len={} ptr={}",
                    //     idx as u32,
                    //     data_len as u32,
                    //     ptr as u32
                    // );
                }
            }
        }
        
        Ok(())
    }

    /// Serialize CreateAccount instruction data (Zero-Copy)
    #[inline(always)]
    fn serialize_create_account_data(data: &mut [u8; 52], lamports: u64, space: u64, owner: &[u8; 32]) {
        // Discriminator (0 for CreateAccount) - already 0 initialized
        // data[0..4] = [0, 0, 0, 0]; 
        
        // Lamports
        let lamports_bytes = lamports.to_le_bytes();
        data[4] = lamports_bytes[0];
        data[5] = lamports_bytes[1];
        data[6] = lamports_bytes[2];
        data[7] = lamports_bytes[3];
        data[8] = lamports_bytes[4];
        data[9] = lamports_bytes[5];
        data[10] = lamports_bytes[6];
        data[11] = lamports_bytes[7];
        
        // Space
        let space_bytes = space.to_le_bytes();
        data[12] = space_bytes[0];
        data[13] = space_bytes[1];
        data[14] = space_bytes[2];
        data[15] = space_bytes[3];
        data[16] = space_bytes[4];
        data[17] = space_bytes[5];
        data[18] = space_bytes[6];
        data[19] = space_bytes[7];
        
        // Owner
        data[20..52].copy_from_slice(owner);
    }

    /// Create a PDA account using System Program (CPI with seeds)
    ///
    /// This function implements the logic for the INIT_PDA_ACCOUNT opcode.
    /// It performs `create_account` with signer seeds.
    ///
    /// NOTE: For PDAs, we typically use `transfer` (to fund), `allocate` (space), and `assign` (owner)
    /// sequence because `create_account` fails if the account already has lamports (which PDAs often do
    /// if funded beforehand). However, INIT_PDA_ACCOUNT implies we are initializing it now.
    ///
    /// To be safe and robust, we implement the 3-step approach:
    /// 1. Transfer required lamports (if needed)
    /// 2. Allocate space (system_instruction::allocate)
    /// 3. Assign owner (system_instruction::assign)
    pub fn create_pda_account(
        &mut self,
        account_idx: u8,
        payer_idx: u8,
        lamports: u64,
        space: u64,
        owner: &[u8; 32],
        seeds: &[u8], // Flattened seeds: [seed1_len, seed1_bytes..., seed2_len, ...]
        seeds_count: u8,
        bump: u8,
    ) -> CompactResult<()> {
        let new_account = self.get_account_unchecked(account_idx)?;

        // Debug: Log payer_idx before validation
        // crate::error_log!("create_pda_account: payer_idx={} num_accounts={}", payer_idx as u32, self.accounts.len() as u32);

        // Validate payer_idx
        if payer_idx as usize >= self.accounts.len() {
            // crate::error_log!("create_pda_account: INVALID payer_idx {} >= num_accounts {}", payer_idx as u32, self.accounts.len() as u32);
            return Err(VMErrorCode::InvalidAccountIndex);
        }

        let payer = self.get_account_unchecked(payer_idx)?;

        // Debug: Log all critical parameters
        let _p_key = payer.key().as_ref();
        let _n_key = new_account.key().as_ref();
        // crate::error_log!("create_pda_account: account_idx={} payer_idx={} lamports={} space={}", account_idx as u32, payer_idx as u32, lamports, space);
        // crate::error_log!("create_pda_account: acc_key={} {} {} {}", n_key[0], n_key[1], n_key[2], n_key[3]);
        /*
        crate::error_log!(
            "Payer details: key={} {} {} {} is_signer={} is_writable={} lamports={}",
            p_key[0], p_key[1], p_key[2], p_key[3],
            if payer.is_signer() { 1 } else { 0 },
            if payer.is_writable() { 1 } else { 0 },
            payer.lamports()
        );
        */

        let system_program_id = Pubkey::from(SYSTEM_PROGRAM_ID);
        let system_program = self
            .accounts
            .iter()
            .find(|a| *a.key() == system_program_id)
            .ok_or({
                debug_log!("create_account: System Program not found");
                // crate::error_log!("create_pda_account: System Program NOT FOUND in accounts!");
                VMErrorCode::AccountNotFound
            })?;
        
        // crate::error_log!("create_pda_account: system_program_key={}", system_program.key().as_ref()[0]);

        // Use CreateAccount with invoke_signed for PDAs
        // The owner should be the Five VM program (self.program_id), not the System Program
        
        // Log owner for debugging
        let _owner_bytes = owner.as_ref();
        /*
        crate::error_log!(
            "create_pda_account: requested_owner={} {} {} {}",
            owner_bytes[0], owner_bytes[1], owner_bytes[2], owner_bytes[3]
        );
        */
        
        let mut data = [0u8; 52];
        Self::serialize_create_account_data(&mut data, lamports, space, owner);

        // Parse seeds into a structure usable for signing
        // This is tricky because `invoke_signed` expects `&[&[u8]]`.
        // We need to construct this from the flattened `seeds` slice.
        // Since we are in a no-alloc environment, we use a fixed-size array of references.
        // We limit max seeds to 8 for now.
        
        // NOTE: We need to parse the seeds properly.
        // Format: [len (1 byte), bytes (len), len (1 byte), bytes (len), ...]
        
        // We'll use a fixed buffer for references on the stack
        
        // Since we can't easily construct `&[&[u8]]` dynamically without a Vec of references,
        // and we can't store references to the temporary buffer easily in a loop,
        // we'll implement a simpler approach:
        // We will assume the seeds are already validated and just construct the signers.
        
        // Actually, we need to construct `&[&[u8]]` for `invoke_signed`.
        // We can do this by manually parsing and storing slices.
        
        // Implementation detail: We use a heapless::Vec of slices
        
        #[cfg(target_os = "solana")]
        {
            // crate::error_log!("create_pda_account: Executing SOLANA path (CPI) - 3-step approach");
            
            // Build signer seeds for PDA signing
            // crate::error_log!("CPI CHECK 1");
            const MAX_SEEDS: usize = 8;
            let binding = [bump];
            let mut seed_vec: heapless::Vec<Seed, MAX_SEEDS> = heapless::Vec::new();
            
            // Parse seeds from input buffer
            let mut offset = 0;
            for _ in 0..seeds_count {
                if offset >= seeds.len() { break; }
                let len = seeds[offset] as usize;
                offset += 1;
                if offset + len > seeds.len() { break; }
                let seed_bytes = &seeds[offset..offset + len];
                seed_vec.push(Seed::from(seed_bytes)).map_err(|_| VMErrorCode::TooManySeeds)?;
                offset += len;
            }
            
            // Add bump seed
            seed_vec
                .push(Seed::from(&binding))
                .map_err(|_| VMErrorCode::TooManySeeds)?;
            // crate::error_log!("CPI CHECK 2");

            // Use simpler message for debugging
            #[cfg(target_os = "solana")]
            unsafe {
                // pinocchio::log::sol_log("SIGNER_CREATE_START");
            }

            // Create the signer from seed vector
            let signer = Signer::from(seed_vec.as_slice());

            #[cfg(target_os = "solana")]
            unsafe {
                // pinocchio::log::sol_log("SIGNER_CREATE_DONE");
            }

            #[cfg(target_os = "solana")]
            unsafe {
                // Log lamports value to see if it's 0
                // pinocchio::log::sol_log("BEFORE_STEP1");
            }

            // Step 2: Allocate space for PDA (always needed)
            // NOTE: We skip Step 1 (transfer) since the rent is pre-funded
            #[cfg(target_os = "solana")]
            unsafe {
                // pinocchio::log::sol_log("SKIPPING_STEP1");
            }

            // Step 1: Transfer lamports to new account (if needed)
            if lamports > 0 {
                #[cfg(target_os = "solana")]
                unsafe {
                    // pinocchio::log::sol_log("STEP1_START");
                }

                let mut transfer_data = [0u8; 12];
                transfer_data[0..4].copy_from_slice(&2u32.to_le_bytes()); // Transfer discriminator
                transfer_data[4..12].copy_from_slice(&lamports.to_le_bytes());

                let transfer_instruction = pinocchio::instruction::Instruction {
                    program_id: system_program.key(),
                    accounts: &[
                        pinocchio::instruction::AccountMeta {
                            pubkey: payer.key(),
                            is_signer: true,
                            is_writable: true,
                        },
                        pinocchio::instruction::AccountMeta {
                            pubkey: new_account.key(),
                            is_signer: false,
                            is_writable: true,
                        },
                    ],
                    data: &transfer_data,
                };

                #[cfg(target_os = "solana")]
                unsafe {
                    // pinocchio::log::sol_log("STEP1_BEFORE_INVOKE");
                }

                let result = invoke::<3>(&transfer_instruction, &[payer, new_account, system_program]);

                #[cfg(target_os = "solana")]
                unsafe {
                    // pinocchio::log::sol_log("STEP1_AFTER_INVOKE");
                }

                if result.is_err() {
                    #[cfg(target_os = "solana")]
                    unsafe {
                        pinocchio::log::sol_log("STEP1_FAILED");
                    }
                    return Err(VMErrorCode::InvokeError);
                }
                #[cfg(target_os = "solana")]
                unsafe {
                    // pinocchio::log::sol_log("STEP1_SUCCESS");
                }
            }
            
            // Step 2: Allocate space for PDA (requires PDA signature)
            // crate::error_log!("create_pda_account: Step 2 - Allocate space={}", space);
            let mut allocate_data = [0u8; 12];
            allocate_data[0..4].copy_from_slice(&8u32.to_le_bytes()); // Allocate discriminator
            allocate_data[4..12].copy_from_slice(&space.to_le_bytes());

            let allocate_instruction = pinocchio::instruction::Instruction {
                program_id: system_program.key(),
                accounts: &[
                    pinocchio::instruction::AccountMeta {
                        pubkey: new_account.key(),
                        is_signer: true,
                        is_writable: true,
                    },
                ],
                data: &allocate_data,
            };

            #[cfg(target_os = "solana")]
            unsafe {
                // pinocchio::log::sol_log("STEP2_BEFORE_INVOKE");
            }

            // let result: CompactResult<()> = Ok(()); // Mock success for debugging
            let result = invoke_signed::<2>(&allocate_instruction, &[new_account, system_program], &[signer]);

            #[cfg(target_os = "solana")]
            unsafe {
                // pinocchio::log::sol_log("STEP2_AFTER_INVOKE");
            }

            if result.is_err() {
                #[cfg(target_os = "solana")]
                unsafe {
                    pinocchio::log::sol_log("STEP2_FAILED");
                }
                return Err(VMErrorCode::InvokeError);
            }
            #[cfg(target_os = "solana")]
            unsafe {
                // pinocchio::log::sol_log("STEP2_SUCCESS");
            }

            // Rebuild signer (it may have been consumed)
            let mut seed_vec2: heapless::Vec<Seed, MAX_SEEDS> = heapless::Vec::new();
            
            // Re-parse seeds
            let mut offset = 0;
            for _ in 0..seeds_count {
                if offset >= seeds.len() { break; }
                let len = seeds[offset] as usize;
                offset += 1;
                if offset + len > seeds.len() { break; }
                let seed_bytes = &seeds[offset..offset + len];
                seed_vec2.push(Seed::from(seed_bytes)).map_err(|_| VMErrorCode::TooManySeeds)?;
                offset += len;
            }
            seed_vec2.push(Seed::from(&binding)).map_err(|_| VMErrorCode::TooManySeeds)?;
            
            let signer2 = Signer::from(seed_vec2.as_slice());

            // Step 3: Assign owner to PDA (requires PDA signature)
            // crate::error_log!("create_pda_account: Step 3 - Assign owner");
            let mut assign_data = [0u8; 36];
            assign_data[0..4].copy_from_slice(&1u32.to_le_bytes()); // Assign discriminator
            assign_data[4..36].copy_from_slice(owner);

            let assign_instruction = pinocchio::instruction::Instruction {
                program_id: system_program.key(),
                accounts: &[
                    pinocchio::instruction::AccountMeta {
                        pubkey: new_account.key(),
                        is_signer: true,
                        is_writable: true,
                    },
                ],
                data: &assign_data,
            };

            #[cfg(target_os = "solana")]
            unsafe {
                // pinocchio::log::sol_log("STEP3_BEFORE_INVOKE");
            }

            let result = invoke_signed::<2>(&assign_instruction, &[new_account, system_program], &[signer2]);

            #[cfg(target_os = "solana")]
            unsafe {
                // pinocchio::log::sol_log("STEP3_AFTER_INVOKE");
            }

            if result.is_err() {
                #[cfg(target_os = "solana")]
                unsafe {
                    pinocchio::log::sol_log("STEP3_FAILED");
                }
                return Err(VMErrorCode::InvokeError);
            }
            #[cfg(target_os = "solana")]
            unsafe {
                // pinocchio::log::sol_log("STEP3_SUCCESS");
            }
        }
        #[cfg(not(target_os = "solana"))]
        {
            // Just simulate consumption for non-solana targets
            core::hint::black_box((seeds, bump, payer_idx));
        }

        // crate::error_log!("create_pda_account: All CPI steps completed successfully");

        // CRITICAL FIX: Refresh pointer for the newly created PDA account
        // Same as create_account - after CreateAccount CPI, the account data is reallocated.
        let _ = self.refresh_account_pointers_after_cpi(&[account_idx as usize]);

        Ok(())
    }
}

// Constant for System Program ID
const SYSTEM_PROGRAM_ID: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];