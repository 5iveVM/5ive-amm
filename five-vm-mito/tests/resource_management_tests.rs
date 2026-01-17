use five_vm_mito::{
    MitoVM,
    systems::resource::ResourceManager,
    error::VMErrorCode,
};
use pinocchio::pubkey::Pubkey;

#[test]
fn test_resource_manager_heap_tracking() {
    let mut temp_buffer = [0u8; 1024];
    let mut manager = ResourceManager::new(&mut temp_buffer);
    
    assert_eq!(manager.heap_usage(), 0, "Initial heap usage should be 0");
    
    // Allocate 100 bytes
    let _addr1 = manager.alloc_heap_unsafe(100).unwrap();
    // Default chunk size is 2048, so usage should be 2048
    assert_eq!(manager.heap_usage(), 2048, "Heap usage should reflect first chunk size");
    
    // Allocate another 100 bytes (fits in chunk)
    let _addr2 = manager.alloc_heap_unsafe(100).unwrap();
    assert_eq!(manager.heap_usage(), 2048, "Heap usage should unchanged when reusing chunk");
    
    // Allocate massive chunk (exceeds default)
    let large_size = 5000;
    let _addr3 = manager.alloc_heap_unsafe(large_size).unwrap();
    assert_eq!(manager.heap_usage(), 2048 + large_size, "Heap usage should increase by large allocation size");
}

#[test]
fn test_recursion_stack_overflow() {
    // A script that simply calls the entry point recursively (Function 0)
    // Header V3: magic(4) + features(4) + public_count(1) + total_count(1)
    // Opcode: CALL 0 (params) (func_index)
    
    let mut script = vec![
        b'5', b'I', b'V', b'E', // Magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x01,                   // Public count
        0x01,                   // Total count
    ];
    
    // Function 0 (Entry Point):
    // CALL 0 params 0 func_addr 10 (jump to self)
    script.extend_from_slice(&[
        0x90, // CALL
        0x00, // 0 params
        10, 0x00, // Address 10 (This instruction start)
        0x00 // NOP padding/end (won't be reached)
    ]);
    
    // Execution should fail with StackOverflow or CallStackOverflow eventually
    let result = MitoVM::execute_direct(&script, &[], &[], &Pubkey::default());
    
    assert!(result.is_err());
    let err = result.err().unwrap();
    
    // It could be CallStackOverflow (depth limit) or StackOverflow (memory limit)
    // Since we added check_stack_limit inside CALL, verifying either proves enforcement is active.
    let error_code = VMErrorCode::from(err);
    match error_code {
        VMErrorCode::StackOverflow | VMErrorCode::CallStackOverflow => {
            println!("Caught expected overflow: {:?}", error_code);
        },
        _ => panic!("Expected overflow error, got: {:?}", error_code),
    }
}

#[test]
fn test_temp_buffer_operations() {
    let mut temp_buffer = [0u8; 1024];
    let mut manager = ResourceManager::new(&mut temp_buffer);
    
    // Test 1: Simple Allocation
    let offset1 = manager.alloc_temp(10).unwrap();
    assert_eq!(offset1, 0);
    assert_eq!(manager.temp_pos, 10);
    
    // Test 2: Sequential Allocation
    let offset2 = manager.alloc_temp(20).unwrap();
    assert_eq!(offset2, 10);
    assert_eq!(manager.temp_pos, 30);
    
    // Test 3: Data Write & Read (Mutable access via helper if public, else via buffer directly)
    // get_temp_data_mut is available
    {
        let slice = manager.get_temp_data_mut(offset1, 10).unwrap();
        slice[0] = 0xAA;
        slice[9] = 0xBB;
    }
    
    {
        let slice = manager.get_temp_data(offset1, 10).unwrap();
        assert_eq!(slice[0], 0xAA);
        assert_eq!(slice[9], 0xBB);
    }
    
    // Test 4: Buffer Reset
    manager.reset_temp_buffer();
    assert_eq!(manager.temp_pos, 0);
    
    // Test 5: Reuse after reset
    let offset3 = manager.alloc_temp(5).unwrap();
    assert_eq!(offset3, 0);
}

#[test]
fn test_temp_buffer_overflow() {
    let mut temp_buffer = [0u8; 100]; // Small buffer
    let mut manager = ResourceManager::new(&mut temp_buffer);
    
    // Alloc 90
    manager.alloc_temp(90).unwrap();
    
    // Alloc 20 -> Should fail
    let res = manager.alloc_temp(20);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), VMErrorCode::MemoryError);
}

#[test]
fn test_heap_data_access() {
    let mut temp_buffer = [0u8; 1024];
    let mut manager = ResourceManager::new(&mut temp_buffer);
    
    // Allocate heap chunk
    let addr = manager.alloc_heap_unsafe(50).unwrap();
    
    // Write data
    {
        let data = manager.get_heap_data_mut(addr, 50).unwrap();
        data[0] = 1;
        data[49] = 255;
    }
    
    // Read data
    {
        let data = manager.get_heap_data(addr, 50).unwrap();
        assert_eq!(data[0], 1);
        assert_eq!(data[49], 255);
    }
}

#[test]
fn test_heap_chunk_overflow() {
    let mut temp_buffer = [0u8; 1024];
    let mut manager = ResourceManager::new(&mut temp_buffer);
    
    // Alloc 1: Matches default chunk (2048)
    let addr1 = manager.alloc_heap_unsafe(2000).unwrap();
    
    // Alloc 2: Should fit in remaining 48 bytes of first chunk?
    // Wait, 2048 - 2000 = 48.
    let addr2 = manager.alloc_heap_unsafe(40).unwrap();
    // Check if addr2 is in same chunk (same high byte)
    assert_eq!(addr1 >> 24, addr2 >> 24, "Should be in same chunk");
    
    // Alloc 3: Won't fit (needs 20 bytes, only 8 left). Should create new chunk.
    let addr3 = manager.alloc_heap_unsafe(20).unwrap();
    assert_ne!(addr1 >> 24, addr3 >> 24, "Should be in new chunk");
    
    // Alloc 4: Large alloc, should be new chunk
    let addr4 = manager.alloc_heap_unsafe(5000).unwrap();
    assert_ne!(addr3 >> 24, addr4 >> 24, "Should be in new chunk (large)");
}

#[test]
fn test_stack_usage_reporting() {
    let mut temp_buffer = [0u8; 1024];
    let manager = ResourceManager::new(&mut temp_buffer);
    
    // Just verify it doesn't panic and returns a plausible value (>= 0)
    let usage = manager.stack_usage();
    println!("Reported stack usage: {}", usage);
    // Hard to assert exact value, but shouldn't crash
}
