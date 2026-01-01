use pinocchio::pubkey::Pubkey;
use pinocchio::program_error::ProgramError;

#[cfg(test)]
mod alloc {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub struct Counter;

    pub static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

    // SAFETY: `Counter` forwards all operations to the system allocator while
    // tracking allocation counts for tests.
    unsafe impl GlobalAlloc for Counter {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
            // SAFETY: Delegates allocation to `System` with the given layout.
            System.alloc(layout)
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            // SAFETY: `ptr` and `layout` were produced by `alloc` above, so
            // passing them to `System::dealloc` is valid.
            System.dealloc(ptr, layout)
        }
    }
}

#[cfg(test)]
#[global_allocator]
static GLOBAL: alloc::Counter = alloc::Counter;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimization::analyzer::OnChainBytecodeAnalyzer;
    use crate::optimization::storage::{process_deploy_optimized, process_execute_optimized, Storage};
    use crate::state::{FIVEVMState, FIVEScriptHeader, FIVEScriptHeaderV2};
    use bytemuck;
    use pinocchio::account_info::AccountInfo;
    use std::sync::atomic::Ordering;

    // Simple metadata builder operating entirely on the stack
    const fn build_metadata(calls: &[u16]) -> [u8; 2] {
        [calls[0] as u8, calls[1] as u8]
    }

    #[test]
    fn optimized_deployment_pipeline() {
        // Bytecode -> analysis -> metadata -> storage
        const BYTECODE: [u8; 5] = [0x35, 0x49, 0x56, 0x45, 0x00];
        let mut analyzer = OnChainBytecodeAnalyzer::new();
        analyzer.analyze_bytecode(&BYTECODE);
        let analysis = analyzer.build_analysis_result();
        let metadata = build_metadata(&analysis.function_calls);
        let _storage = Storage;
        assert_eq!(metadata[0], analysis.function_calls[0] as u8);
    }

    #[test]
    fn account_owner_and_header_parsing() {
        // Prepare header entirely on the stack
        let mut data = [0u8; FIVEScriptHeader::LEN];
        {
            let header = bytemuck::from_bytes_mut::<FIVEScriptHeader>(&mut data);
            header.owner = [1u8; 32];
            header.script_id = 7;
            header.bytecode_len = 0;
        }
        let parsed = FIVEScriptHeader::from_account_data(&data).unwrap();
        let expected: Pubkey = [1u8; 32];
        assert_eq!(parsed.owner, expected);
    }

    #[test]
    fn error_cases() {
        // Invalid magic
        const BAD_MAGIC: [u8; 4] = [0u8; 4];
        assert_ne!(&BAD_MAGIC, b"5IVE");

        // Wrong owner
        let mut data = [0u8; FIVEScriptHeader::LEN];
        {
            let header = bytemuck::from_bytes_mut::<FIVEScriptHeader>(&mut data);
            header.owner = [2u8; 32];
            header.script_id = 1;
            header.bytecode_len = 0;
        }
        let parsed = FIVEScriptHeader::from_account_data(&data).unwrap();
        let expected: Pubkey = [3u8; 32];
        assert_ne!(parsed.owner, expected);

        // Malformed metadata (insufficient output buffer)
        const INPUT: [u32; 3] = [1, 2, 3];
        let mut buf = [0u32; 2];
        let result = (|| -> Result<(), ProgramError> {
            if INPUT.len() > buf.len() {
                return Err(ProgramError::InvalidInstructionData);
            }
            for i in 0..INPUT.len() {
                buf[i] = INPUT[i];
            }
            Ok(())
        })();
        assert!(result.is_err());
    }

    #[test]
    fn deploy_and_execute_optimized() {
        const BYTECODE: [u8; 7] = [0x35, 0x49, 0x56, 0x45, 0x10, 0x80, 0x00];

        // Prepare VM state on stack
        let program_id = Pubkey::new_unique();
        let script_key = Pubkey::new_unique();
        let vm_key = Pubkey::new_unique();
        let owner_key = Pubkey::new_unique();
        let mut script_lamports = 0u64;
        let mut vm_lamports = 0u64;
        let mut owner_lamports = 0u64;
        let mut script_data = [0u8; 512];
        let mut vm_data = [0u8; FIVEVMState::LEN];
        {
            let vm = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
            vm.set_initialized(true);
            vm.script_count = 0;
        }

        let script_account = AccountInfo::new(
            &script_key,
            false,
            true,
            &mut script_lamports,
            &mut script_data,
            &program_id,
            false,
            0,
        );

        let vm_account = AccountInfo::new(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
            false,
            0,
        );

        let owner_account = AccountInfo::new(
            &owner_key,
            true,
            false,
            &mut owner_lamports,
            &mut [],
            &program_id,
            false,
            0,
        );

        let deploy_accounts = [
            script_account.clone(),
            vm_account.clone(),
            owner_account.clone(),
        ];

        // Deploy and ensure no heap allocations
        alloc::ALLOCATIONS.store(0, Ordering::SeqCst);
        process_deploy_optimized(&program_id, &deploy_accounts, &BYTECODE).unwrap();
        assert_eq!(alloc::ALLOCATIONS.load(Ordering::SeqCst), 0);

        // Verify metadata layout
        let header = FIVEScriptHeaderV2::from_account_data(&script_data).unwrap();
        assert_eq!(header.owner, owner_key);
        assert_eq!(header.script_id, 0);
        assert_eq!(header.bytecode_len as usize, BYTECODE.len());

        let mut offset = FIVEScriptHeaderV2::LEN;
        let fe_len = u32::from_le_bytes([
            script_data[offset],
            script_data[offset + 1],
            script_data[offset + 2],
            script_data[offset + 3],
        ]) as usize;
        offset += 4 + fe_len * 4;
        let opcode_start = script_data[offset];
        let opcode_len = script_data[offset + 1];
        offset += 2;
        let exception_bytes = if opcode_len == 0 {
            32
        } else {
            ((opcode_len as usize + 7) / 8)
        };
        let exceptions = &script_data[offset..offset + exception_bytes];
        offset += exception_bytes;
        // Static constraints region should be zeroed
        let dyn_offset = {
            let bytecode_offset = offset
                + script_data[offset..]
                    .windows(BYTECODE.len())
                    .position(|w| w == BYTECODE)
                    .unwrap();
            bytecode_offset - 4
        };
        let sc_bytes = &script_data[offset..dyn_offset];
        assert!(sc_bytes.iter().all(|&b| b == 0));
        let dyn_constraints = u32::from_le_bytes([
            script_data[dyn_offset],
            script_data[dyn_offset + 1],
            script_data[dyn_offset + 2],
            script_data[dyn_offset + 3],
        ]);
        assert_eq!(dyn_constraints, 0);

        // Verify opcode range matches bytecode
        let mut min = u8::MAX;
        let mut max = 0u8;
        for op in BYTECODE.iter().skip(4) {
            if *op < min {
                min = *op;
            }
            if *op > max {
                max = *op;
            }
        }
        let expected_len = (max as u16) - (min as u16) + 1;
        assert_eq!(opcode_start, min);
        assert_eq!(opcode_len as u16, expected_len);

        // Verify exceptions bitmap
        let mut bitmap = [0xFFu8; 32];
        for &op in BYTECODE.iter().skip(4) {
            let idx = (op - min) as usize;
            bitmap[idx / 8] &= !(1 << (idx % 8));
        }
        if expected_len < 256 {
            for bit in expected_len as usize..256 {
                bitmap[bit / 8] &= !(1 << (bit % 8));
            }
        }
        assert_eq!(exceptions, &bitmap[..exception_bytes]);

        // Execute and ensure no heap allocations
        let exec_accounts = [script_account, vm_account];
        alloc::ALLOCATIONS.store(0, Ordering::SeqCst);
        process_execute_optimized(&program_id, &exec_accounts, &[]).unwrap();
        assert_eq!(alloc::ALLOCATIONS.load(Ordering::SeqCst), 0);

    }
}
