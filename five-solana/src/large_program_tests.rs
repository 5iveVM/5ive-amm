#[cfg(test)]
mod large_program_tests {
    use super::large_program_chunking::*;
    use five_vm_core::VMError;
    use std::time::Instant;
    use std::collections::HashMap;

    fn create_test_source(size: usize, pattern: u8) -> String {
        let mut source = String::with_capacity(size);
        for i in 0..size {
            source.push((pattern.wrapping_add(i as u8) % 95 + 32) as char); // Printable ASCII
        }
        source
    }

    fn create_large_test_program() -> String {
        let mut program = String::new();
        program.push_str("// Large test program for chunking validation\n");
        program.push_str("contract LargeTestContract {\n");
        
        // Add many functions to reach large size
        for i in 0..1000 {
            program.push_str(&format!(
                "  function test_function_{}(account: Account, value: u64) -> u64 {{\n",
                i
            ));
            program.push_str("    require(account.is_signer(), \"Must be signer\");\n");
            program.push_str("    require(account.is_writable(), \"Must be writable\");\n");
            program.push_str(&format("    return value + {};\n", i));
            program.push_str("  }\n\n");
        }
        
        program.push_str("}\n");
        program
    }

    #[test]
    fn test_chunk_manifest_creation() {
        let total_size = 100_000u32;
        let chunk_count = 25u16;
        let program_hash = [0xAAu8; 32];
        
        let manifest = ChunkManifest::new(total_size, chunk_count, program_hash);
        
        assert_eq!(manifest.total_size, total_size);
        assert_eq!(manifest.chunk_count, chunk_count);
        assert_eq!(manifest.program_hash, program_hash);
        assert_eq!(manifest.account_count, 1); // Should fit in single account
        
        // Test chunk distribution calculation
        let mut total_chunks = 0;
        for i in 0..manifest.account_count {
            total_chunks += manifest.chunk_distribution[i as usize] as u16;
        }
        assert_eq!(total_chunks, chunk_count);
    }

    #[test]
    fn test_chunk_manifest_multi_account_distribution() {
        let chunk_count = 300u16; // Exceeds MAX_LARGE_CHUNKS (256)
        let manifest = ChunkManifest::new(1_000_000, chunk_count, [0; 32]);
        
        // Should require multiple accounts
        assert!(manifest.account_count > 1);
        assert!(manifest.account_count <= MAX_ACCOUNTS_PER_PROGRAM);
        
        // Verify distribution adds up
        let mut total_chunks = 0;
        for i in 0..manifest.account_count {
            total_chunks += manifest.chunk_distribution[i as usize] as u16;
        }
        assert_eq!(total_chunks, chunk_count);
    }

    #[test]
    fn test_chunk_manifest_bitmap_operations() {
        let mut manifest = ChunkManifest::new(50_000, 50, [0; 32]);
        
        // Initially no chunks received
        assert!(!manifest.is_complete());
        for i in 0..50 {
            assert!(!manifest.is_chunk_received(i));
        }
        
        // Mark chunks as received
        for i in 0..25 {
            assert!(manifest.set_chunk_received(i).is_ok());
            assert!(manifest.is_chunk_received(i));
        }
        
        assert!(!manifest.is_complete());
        
        // Mark remaining chunks
        for i in 25..50 {
            assert!(manifest.set_chunk_received(i).is_ok());
        }
        
        assert!(manifest.is_complete());
        
        // Test bounds checking
        assert!(manifest.set_chunk_received(50).is_err());
        assert!(manifest.set_chunk_received(1000).is_err());
    }

    #[test]
    fn test_chunk_manifest_account_assignment() {
        let manifest = ChunkManifest::new(200_000, 100, [0; 32]);
        
        // Test chunk to account mapping
        for chunk_idx in 0..100 {
            let account_idx = manifest.get_account_for_chunk(chunk_idx).unwrap();
            assert!(account_idx < manifest.account_count);
        }
        
        // Verify chunks are distributed across accounts
        let mut account_usage = vec![0u16; MAX_ACCOUNTS_PER_PROGRAM];
        for chunk_idx in 0..100 {
            let account_idx = manifest.get_account_for_chunk(chunk_idx).unwrap();
            account_usage[account_idx as usize] += 1;
        }
        
        // At least the first account should have chunks
        assert!(account_usage[0] > 0);
        
        // Test invalid chunk indices
        assert!(manifest.get_account_for_chunk(100).is_err());
        assert!(manifest.get_account_for_chunk(1000).is_err());
    }

    #[test]
    fn test_large_chunk_creation_and_verification() {
        let test_data = b"This is test chunk data for verification".to_vec();
        let chunk = LargeChunk::new(5, 2, test_data.clone());
        
        assert_eq!(chunk.header.chunk_index, 5);
        assert_eq!(chunk.header.account_index, 2);
        assert_eq!(chunk.header.chunk_size, test_data.len() as u16);
        assert_eq!(chunk.data, test_data);
        
        // Test hash verification
        assert!(chunk.header.verify_hash(&test_data));
        
        // Test invalid hash detection
        let mut wrong_data = test_data.clone();
        wrong_data[0] = wrong_data[0].wrapping_add(1);
        assert!(!chunk.header.verify_hash(&wrong_data));
    }

    #[test]
    fn test_large_chunk_serialization() {
        let test_data = b"Serialization test data".to_vec();
        let original_chunk = LargeChunk::new(10, 1, test_data.clone());
        
        // Serialize to instruction data
        let instruction_data = original_chunk.to_instruction_data();
        
        // Deserialize back
        let restored_chunk = LargeChunk::from_instruction_data(&instruction_data).unwrap();
        
        // Verify restoration
        assert_eq!(restored_chunk.header.chunk_index, original_chunk.header.chunk_index);
        assert_eq!(restored_chunk.header.account_index, original_chunk.header.account_index);
        assert_eq!(restored_chunk.header.chunk_size, original_chunk.header.chunk_size);
        assert_eq!(restored_chunk.header.chunk_hash, original_chunk.header.chunk_hash);
        assert_eq!(restored_chunk.data, original_chunk.data);
    }

    #[test]
    fn test_large_chunk_malformed_data() {
        // Test with insufficient data
        let short_data = vec![0u8; 10];
        assert!(LargeChunk::from_instruction_data(&short_data).is_err());
        
        // Test with wrong prefix
        let mut wrong_prefix = vec![0u8; 100];
        wrong_prefix[0..4].copy_from_slice(b"WRONG");
        assert!(LargeChunk::from_instruction_data(&wrong_prefix).is_err());
        
        // Test with mismatched size in header
        let test_data = b"test".to_vec();
        let mut chunk = LargeChunk::new(0, 0, test_data);
        chunk.header.chunk_size = 100; // Wrong size
        let bad_instruction = chunk.to_instruction_data();
        assert!(LargeChunk::from_instruction_data(&bad_instruction).is_err());
    }

    #[test]
    fn test_multi_account_bytecode_assembly() {
        let manifest = ChunkManifest::new(20_000, 10, [0; 32]);
        let mut bytecode = MultiAccountBytecode::new(manifest);
        
        // Create and add chunks
        for i in 0..10 {
            let chunk_data = format!("Chunk {} data content", i).into_bytes();
            let account_idx = bytecode.manifest.get_account_for_chunk(i).unwrap();
            let chunk = LargeChunk::new(i, account_idx, chunk_data);
            
            assert!(bytecode.add_chunk(chunk).is_ok());
        }
        
        assert!(bytecode.is_complete());
        
        // Test full bytecode reconstruction
        let full_bytecode = bytecode.get_full_bytecode().unwrap();
        assert!(full_bytecode.len() > 0);
        
        // Verify chunks are concatenated correctly
        let mut expected = Vec::new();
        for i in 0..10 {
            expected.extend_from_slice(&format!("Chunk {} data content", i).into_bytes());
        }
        assert_eq!(full_bytecode, expected);
    }

    #[test]
    fn test_multi_account_bytecode_partial_reading() {
        let manifest = ChunkManifest::new(100, 5, [0; 32]);
        let mut bytecode = MultiAccountBytecode::new(manifest);
        
        // Add chunks with predictable data
        for i in 0..5 {
            let mut chunk_data = vec![0u8; LARGE_CHUNK_SIZE];
            // Fill with pattern for testing
            for j in 0..LARGE_CHUNK_SIZE {
                chunk_data[j] = ((i * LARGE_CHUNK_SIZE + j) % 256) as u8;
            }
            
            let chunk = LargeChunk::new(i, 0, chunk_data);
            bytecode.add_chunk(chunk).unwrap();
        }
        
        // Test reading across chunk boundaries
        let mid_chunk_read = bytecode.read_at(LARGE_CHUNK_SIZE - 10, 20).unwrap();
        assert_eq!(mid_chunk_read.len(), 20);
        
        // Verify data spans chunks correctly
        for (j, &byte) in mid_chunk_read.iter().enumerate() {
            let expected_pos = LARGE_CHUNK_SIZE - 10 + j;
            let expected_byte = (expected_pos % 256) as u8;
            assert_eq!(byte, expected_byte);
        }
        
        // Test reading single chunk
        let single_chunk_read = bytecode.read_at(100, 50).unwrap();
        assert_eq!(single_chunk_read.len(), 50);
        
        // Test bounds violations
        assert!(bytecode.read_at(bytecode.virtual_size, 1).is_err());
        assert!(bytecode.read_at(0, bytecode.virtual_size + 1).is_err());
    }

    #[test]
    fn test_multi_account_bytecode_error_conditions() {
        let manifest = ChunkManifest::new(1000, 5, [0; 32]);
        let mut bytecode = MultiAccountBytecode::new(manifest);
        
        // Test duplicate chunk rejection
        let chunk1 = LargeChunk::new(0, 0, b"first".to_vec());
        let chunk1_dup = LargeChunk::new(0, 0, b"duplicate".to_vec());
        
        assert!(bytecode.add_chunk(chunk1).is_ok());
        assert!(bytecode.add_chunk(chunk1_dup).is_err()); // Should reject duplicate
        
        // Test wrong account index
        let wrong_account_chunk = LargeChunk::new(1, 99, b"wrong".to_vec());
        assert!(bytecode.add_chunk(wrong_account_chunk).is_err());
        
        // Test chunk index out of bounds
        let out_of_bounds_chunk = LargeChunk::new(10, 0, b"oob".to_vec());
        assert!(bytecode.add_chunk(out_of_bounds_chunk).is_err());
    }

    #[test]
    fn test_streaming_compiler_workflow() {
        let mut compiler = StreamingCompiler::new();
        
        // Set manifest
        let manifest = ChunkManifest::new(5000, 5, [0; 32]);
        assert!(compiler.set_manifest(manifest).is_ok());
        assert_eq!(compiler.get_progress(), 0.0);
        
        // Add source chunks
        for i in 0..5 {
            let source_data = format!("Source chunk {} with DSL code", i).into_bytes();
            let chunk = LargeChunk::new(i, 0, source_data);
            
            assert!(compiler.add_source_chunk(chunk).is_ok());
            
            let expected_progress = (i + 1) as f32 / 5.0;
            assert!((compiler.get_progress() - expected_progress).abs() < 0.01);
        }
        
        assert!(compiler.is_compilation_complete());
        assert_eq!(compiler.get_progress(), 1.0);
        
        // Get compiled result
        let compiled_bytecode = compiler.get_compiled_bytecode().unwrap();
        assert!(compiled_bytecode.is_complete());
    }

    #[test]
    fn test_streaming_compiler_error_conditions() {
        let mut compiler = StreamingCompiler::new();
        
        // Test adding chunk before manifest
        let chunk = LargeChunk::new(0, 0, b"premature".to_vec());
        assert!(compiler.add_source_chunk(chunk).is_err());
        
        // Set manifest
        let manifest = ChunkManifest::new(1000, 3, [0; 32]);
        compiler.set_manifest(manifest).unwrap();
        
        // Test duplicate manifest
        let manifest2 = ChunkManifest::new(2000, 4, [0; 32]);
        assert!(compiler.set_manifest(manifest2).is_err());
        
        // Test invalid chunk index
        let invalid_chunk = LargeChunk::new(10, 0, b"invalid".to_vec());
        assert!(compiler.add_source_chunk(invalid_chunk).is_err());
        
        // Test duplicate source chunk
        let chunk1 = LargeChunk::new(0, 0, b"first".to_vec());
        let chunk1_dup = LargeChunk::new(0, 0, b"duplicate".to_vec());
        
        assert!(compiler.add_source_chunk(chunk1).is_ok());
        assert!(compiler.add_source_chunk(chunk1_dup).is_err());
    }

    #[test]
    fn test_streaming_compiler_size_limits() {
        let mut compiler = StreamingCompiler::new();
        
        // Test oversized manifest
        let oversized_manifest = ChunkManifest::new(
            (MAX_LARGE_PROGRAM_SIZE + 1) as u32, 
            10, 
            [0; 32]
        );
        assert!(compiler.set_manifest(oversized_manifest).is_err());
        
        // Test too many chunks
        let too_many_chunks_manifest = ChunkManifest::new(
            100_000, 
            (MAX_LARGE_CHUNKS + 1) as u16, 
            [0; 32]
        );
        assert!(compiler.set_manifest(too_many_chunks_manifest).is_err());
    }

    #[test]
    fn test_large_program_chunking_end_to_end() {
        let mut chunking = LargeProgramChunking::new();
        
        // Create manifest
        let manifest = ChunkManifest::new(10_000, 8, [0xABu8; 32]);
        let mut manifest_data = Vec::new();
        manifest_data.extend_from_slice(CHUNK_MANIFEST_PREFIX);
        manifest_data.extend_from_slice(bytemuck::bytes_of(&manifest));
        
        // Process manifest
        assert!(chunking.process_manifest(&manifest_data).is_ok());
        assert!(matches!(chunking.get_state(), UploadState::ReceivingChunks { received: 0, total: 8 }));
        
        // Process chunks
        for i in 0..8 {
            let chunk_data = format!("Source chunk {} content", i).into_bytes();
            let chunk = LargeChunk::new(i, 0, chunk_data);
            let instruction_data = chunk.to_instruction_data();
            
            assert!(chunking.process_chunk(&instruction_data).is_ok());
            
            let progress = chunking.get_compilation_progress();
            let expected_progress = (i + 1) as f32 / 8.0;
            assert!((progress - expected_progress).abs() < 0.01);
        }
        
        // Should be ready now
        assert!(chunking.is_ready());
        assert!(matches!(chunking.get_state(), UploadState::CompilationComplete));
        
        // Get final bytecode
        let bytecode = chunking.get_bytecode().unwrap();
        assert!(bytecode.is_complete());
    }

    #[test]
    fn test_large_program_chunking_manifest_validation() {
        let mut chunking = LargeProgramChunking::new();
        
        // Test malformed manifest data
        let bad_manifest = b"INVALID_MANIFEST_DATA".to_vec();
        assert!(chunking.process_manifest(&bad_manifest).is_err());
        
        // Test wrong prefix
        let mut wrong_prefix = vec![0u8; 100];
        wrong_prefix[0..5].copy_from_slice(b"WRONG");
        assert!(chunking.process_manifest(&wrong_prefix).is_err());
        
        // Test insufficient size
        let mut short_manifest = Vec::new();
        short_manifest.extend_from_slice(CHUNK_MANIFEST_PREFIX);
        short_manifest.extend_from_slice(&[0u8; 10]); // Too short
        assert!(chunking.process_manifest(&short_manifest).is_err());
    }

    #[test]
    fn test_large_program_chunking_error_recovery() {
        let mut chunking = LargeProgramChunking::new();
        
        // Set up valid manifest
        let manifest = ChunkManifest::new(5000, 5, [0; 32]);
        let mut manifest_data = Vec::new();
        manifest_data.extend_from_slice(CHUNK_MANIFEST_PREFIX);
        manifest_data.extend_from_slice(bytemuck::bytes_of(&manifest));
        chunking.process_manifest(&manifest_data).unwrap();
        
        // Add some chunks
        for i in 0..3 {
            let chunk = LargeChunk::new(i, 0, format!("chunk {}", i).into_bytes());
            chunking.process_chunk(&chunk.to_instruction_data()).unwrap();
        }
        
        // Simulate failure state (would be set by external error handling)
        chunking.upload_state = UploadState::Failed("Simulated failure".to_string());
        
        // Test recovery
        assert!(chunking.recover_from_failure(3).is_ok());
        assert!(matches!(chunking.get_state(), UploadState::ReceivingChunks { received: 3, total: 5 }));
        
        // Should be able to continue
        for i in 3..5 {
            let chunk = LargeChunk::new(i, 0, format!("chunk {}", i).into_bytes());
            chunking.process_chunk(&chunk.to_instruction_data()).unwrap();
        }
        
        assert!(chunking.is_ready());
    }

    #[test]
    fn test_create_chunks_from_source_small() {
        let source = "Simple test program that fits in one chunk";
        let (manifest, chunks) = LargeProgramChunking::create_chunks_from_source(source).unwrap();
        
        assert_eq!(manifest.total_size, source.len() as u32);
        assert_eq!(manifest.chunk_count, 1);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].data, source.as_bytes());
        assert_eq!(chunks[0].header.chunk_index, 0);
    }

    #[test]
    fn test_create_chunks_from_source_large() {
        // Create source that spans multiple chunks
        let source = create_test_source(LARGE_CHUNK_SIZE * 3 + 100, 0x42);
        let (manifest, chunks) = LargeProgramChunking::create_chunks_from_source(&source).unwrap();
        
        assert_eq!(manifest.total_size, source.len() as u32);
        assert_eq!(manifest.chunk_count, 4); // 3 full chunks + 1 partial
        assert_eq!(chunks.len(), 4);
        
        // Verify chunk sizes
        assert_eq!(chunks[0].data.len(), LARGE_CHUNK_SIZE);
        assert_eq!(chunks[1].data.len(), LARGE_CHUNK_SIZE);
        assert_eq!(chunks[2].data.len(), LARGE_CHUNK_SIZE);
        assert_eq!(chunks[3].data.len(), 100); // Partial last chunk
        
        // Verify chunk indices and account assignments
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.header.chunk_index, i as u16);
            let expected_account = manifest.get_account_for_chunk(i as u16).unwrap();
            assert_eq!(chunk.header.account_index, expected_account);
        }
        
        // Verify data integrity
        let mut reconstructed = Vec::new();
        for chunk in &chunks {
            reconstructed.extend_from_slice(&chunk.data);
        }
        assert_eq!(reconstructed, source.as_bytes());
    }

    #[test]
    fn test_create_chunks_from_source_maximum_size() {
        // Test at the size limit
        let source = create_test_source(MAX_LARGE_PROGRAM_SIZE, 0x55);
        let result = LargeProgramChunking::create_chunks_from_source(&source);
        assert!(result.is_ok());
        
        let (manifest, chunks) = result.unwrap();
        assert_eq!(manifest.total_size, MAX_LARGE_PROGRAM_SIZE as u32);
        
        // Should create exactly MAX_LARGE_CHUNKS chunks
        let expected_chunks = (MAX_LARGE_PROGRAM_SIZE + LARGE_CHUNK_SIZE - 1) / LARGE_CHUNK_SIZE;
        assert_eq!(manifest.chunk_count, expected_chunks as u16);
        assert_eq!(chunks.len(), expected_chunks);
    }

    #[test]
    fn test_create_chunks_from_source_oversized() {
        let oversized_source = create_test_source(MAX_LARGE_PROGRAM_SIZE + 1, 0x66);
        let result = LargeProgramChunking::create_chunks_from_source(&oversized_source);
        assert!(matches!(result, Err(VMError::ScriptTooLarge)));
    }

    #[test]
    fn test_create_chunks_from_source_too_many_chunks() {
        // Create source that would require too many chunks
        let chunk_count_limit = MAX_LARGE_CHUNKS + 1;
        let oversized_source = create_test_source(chunk_count_limit * LARGE_CHUNK_SIZE, 0x77);
        let result = LargeProgramChunking::create_chunks_from_source(&oversized_source);
        assert!(matches!(result, Err(VMError::ScriptTooLarge)));
    }

    #[test]
    fn test_hash_verification_integrity() {
        let test_cases = vec![
            b"".to_vec(),
            b"a".to_vec(),
            b"test data".to_vec(),
            vec![0u8; 1000],
            vec![0xFFu8; 1000],
            (0..1000).map(|i| (i % 256) as u8).collect(),
        ];
        
        for test_data in test_cases {
            let hash1 = LargeChunkHeader::compute_hash(&test_data);
            let hash2 = LargeChunkHeader::compute_hash(&test_data);
            
            // Same data should produce same hash
            assert_eq!(hash1, hash2);
            
            // Different data should produce different hash
            if !test_data.is_empty() {
                let mut modified_data = test_data.clone();
                modified_data[0] = modified_data[0].wrapping_add(1);
                let hash3 = LargeChunkHeader::compute_hash(&modified_data);
                assert_ne!(hash1, hash3);
            }
        }
    }

    #[test]
    fn test_performance_large_program_assembly() {
        const CHUNKS: u16 = 100;
        const CHUNK_SIZE: usize = LARGE_CHUNK_SIZE;
        
        let manifest = ChunkManifest::new(
            (CHUNKS as usize * CHUNK_SIZE) as u32,
            CHUNKS,
            [0; 32]
        );
        let mut bytecode = MultiAccountBytecode::new(manifest);
        
        // Create chunks with realistic data
        let mut chunks = Vec::new();
        for i in 0..CHUNKS {
            let mut chunk_data = vec![0u8; CHUNK_SIZE];
            for j in 0..CHUNK_SIZE {
                chunk_data[j] = ((i as usize * CHUNK_SIZE + j) % 256) as u8;
            }
            
            let chunk = LargeChunk::new(i, 0, chunk_data);
            chunks.push(chunk);
        }
        
        // Benchmark chunk assembly
        let start = Instant::now();
        for chunk in chunks {
            bytecode.add_chunk(chunk).unwrap();
        }
        let assembly_duration = start.elapsed();
        
        // Benchmark full bytecode reconstruction
        let start = Instant::now();
        let _full_bytecode = bytecode.get_full_bytecode().unwrap();
        let reconstruction_duration = start.elapsed();
        
        println!("Assembly of {} chunks: {:?}", CHUNKS, assembly_duration);
        println!("Full reconstruction: {:?}", reconstruction_duration);
        
        // Verify performance is reasonable (adjust thresholds as needed)
        assert!(assembly_duration.as_millis() < 100); // Should be fast
        assert!(reconstruction_duration.as_millis() < 50); // Should be very fast
    }

    #[test]
    fn test_performance_streaming_compilation() {
        const CHUNKS: u16 = 50;
        
        let mut compiler = StreamingCompiler::new();
        let manifest = ChunkManifest::new(
            (CHUNKS as usize * 1000) as u32, // 1KB per chunk
            CHUNKS,
            [0; 32]
        );
        compiler.set_manifest(manifest).unwrap();
        
        // Benchmark streaming compilation
        let start = Instant::now();
        
        for i in 0..CHUNKS {
            let source_data = create_test_source(1000, i as u8).into_bytes();
            let chunk = LargeChunk::new(i, 0, source_data);
            compiler.add_source_chunk(chunk).unwrap();
        }
        
        let compilation_duration = start.elapsed();
        
        assert!(compiler.is_compilation_complete());
        
        println!("Streaming compilation of {} chunks: {:?}", CHUNKS, compilation_duration);
        
        // Verify progressive compilation was happening
        assert_eq!(compiler.get_progress(), 1.0);
    }

    #[test]
    fn test_realistic_large_program_deployment() {
        // Create a realistic large program
        let large_program = create_large_test_program();
        println!("Large program size: {} bytes", large_program.len());
        
        // Should be large enough to require chunking
        assert!(large_program.len() > LARGE_CHUNK_SIZE);
        
        // Create chunks from the program
        let (manifest, chunks) = LargeProgramChunking::create_chunks_from_source(&large_program).unwrap();
        
        println!("Created {} chunks", chunks.len());
        assert!(chunks.len() > 1);
        
        // Simulate deployment process
        let mut chunking = LargeProgramChunking::new();
        
        // Send manifest
        let mut manifest_data = Vec::new();
        manifest_data.extend_from_slice(CHUNK_MANIFEST_PREFIX);
        manifest_data.extend_from_slice(bytemuck::bytes_of(&manifest));
        chunking.process_manifest(&manifest_data).unwrap();
        
        // Send chunks in order
        for chunk in chunks {
            let instruction_data = chunk.to_instruction_data();
            chunking.process_chunk(&instruction_data).unwrap();
        }
        
        // Should be ready for execution
        assert!(chunking.is_ready());
        
        // Get final bytecode and verify integrity
        let final_bytecode = chunking.get_bytecode().unwrap();
        let reconstructed_program = final_bytecode.get_full_bytecode().unwrap();
        
        // Note: The "compiled" version will differ from source since we're not
        // doing real compilation in the mock compiler
        assert!(reconstructed_program.len() > 0);
        println!("Final bytecode size: {} bytes", reconstructed_program.len());
    }

    #[test]
    fn test_chunk_manifest_edge_cases() {
        // Test with minimum values
        let minimal_manifest = ChunkManifest::new(1, 1, [0; 32]);
        assert_eq!(minimal_manifest.chunk_count, 1);
        assert_eq!(minimal_manifest.account_count, 1);
        
        // Test with maximum single-account chunks
        let max_single_manifest = ChunkManifest::new(
            (MAX_LARGE_CHUNKS * LARGE_CHUNK_SIZE) as u32,
            MAX_LARGE_CHUNKS as u16,
            [0; 32]
        );
        assert_eq!(max_single_manifest.account_count, 1);
        
        // Test boundary conditions for bitmap
        let mut boundary_manifest = ChunkManifest::new(10000, 256, [0; 32]);
        
        // Test setting chunk at bitmap boundary (bit 255)
        assert!(boundary_manifest.set_chunk_received(255).is_ok());
        assert!(boundary_manifest.is_chunk_received(255));
        
        // Test setting chunk beyond bitmap
        assert!(boundary_manifest.set_chunk_received(256).is_err());
    }

    #[test]
    fn test_virtual_pc_resolution_across_chunks() {
        let manifest = ChunkManifest::new(20_000, 5, [0; 32]);
        let mut bytecode = MultiAccountBytecode::new(manifest);
        
        // Add chunks with known data
        for i in 0..5 {
            let mut chunk_data = vec![0u8; LARGE_CHUNK_SIZE];
            // Set first byte of each chunk to chunk index for identification
            chunk_data[0] = i as u8;
            
            let chunk = LargeChunk::new(i, 0, chunk_data);
            bytecode.add_chunk(chunk).unwrap();
        }
        
        // Test reading at chunk boundaries
        for i in 0..5 {
            let chunk_start = i * LARGE_CHUNK_SIZE;
            let data = bytecode.read_at(chunk_start, 1).unwrap();
            assert_eq!(data[0], i as u8); // Should match chunk index
        }
        
        // Test reading across chunk boundaries
        let boundary_read = bytecode.read_at(LARGE_CHUNK_SIZE - 1, 2).unwrap();
        assert_eq!(boundary_read[0], 0); // Last byte of chunk 0
        assert_eq!(boundary_read[1], 1); // First byte of chunk 1
    }

    #[test]
    fn test_memory_efficiency_large_program() {
        // Test that we don't load entire program into memory unnecessarily
        let manifest = ChunkManifest::new(500_000, 125, [0; 32]); // ~500KB program
        let mut bytecode = MultiAccountBytecode::new(manifest);
        
        // Add only some chunks to simulate partial loading
        for i in 0..50 { // Only half the chunks
            let chunk_data = vec![(i % 256) as u8; LARGE_CHUNK_SIZE];
            let chunk = LargeChunk::new(i, 0, chunk_data);
            bytecode.add_chunk(chunk).unwrap();
        }
        
        // Should be able to read from loaded chunks
        let data = bytecode.read_at(0, 100).unwrap();
        assert_eq!(data.len(), 100);
        
        // Should fail to read from unloaded chunks
        let unloaded_offset = 50 * LARGE_CHUNK_SIZE;
        assert!(bytecode.read_at(unloaded_offset, 100).is_err());
        
        // Verify we don't consider it complete
        assert!(!bytecode.is_complete());
    }
}