/// Compression Module Test Suite
///
/// Tests the compression module which handles:
/// - VLE (Variable Length Encoding) configuration
/// - String pool deduplication and interning
/// - Chunk management for large programs
/// - Pattern-based compression
/// - Compression optimization strategies
use five_dsl_compiler::bytecode_generator::compression::{
    ChunkManager, CompressionOptimizer, PatternCompressor, StringPool, VLEConfig,
};

// ============================================================================
// Test Group 1: VLE Configuration
// ============================================================================

#[test]
fn test_vle_config_default() {
    let config = VLEConfig::default();

    assert!(
        config.enable_field_offsets,
        "Field offsets should be enabled by default"
    );
    assert!(
        config.enable_operands,
        "Operands should be enabled by default"
    );
    assert!(
        config.enable_jump_targets,
        "Jump targets should be enabled by default"
    );
    assert_eq!(config.min_threshold, 128, "Default threshold should be 128");
}

#[test]
fn test_vle_config_custom() {
    let config = VLEConfig {
        enable_field_offsets: false,
        enable_operands: true,
        enable_jump_targets: false,
        min_threshold: 256,
    };

    assert!(!config.enable_field_offsets);
    assert!(config.enable_operands);
    assert!(!config.enable_jump_targets);
    assert_eq!(config.min_threshold, 256);
}

// ============================================================================
// Test Group 2: String Pool Deduplication
// ============================================================================

#[test]
fn test_string_pool_creation() {
    let pool = StringPool::new();

    assert_eq!(pool.len(), 0, "New pool should be empty");
    assert!(pool.is_empty(), "New pool should report as empty");
}

#[test]
fn test_string_pool_intern_single() {
    let mut pool = StringPool::new();

    let index = pool.intern("test_string");

    assert_eq!(index, 0, "First string should get index 0");
    assert_eq!(pool.len(), 1, "Pool should have 1 string");
    assert!(!pool.is_empty(), "Pool should not be empty");
}

#[test]
fn test_string_pool_deduplication() {
    let mut pool = StringPool::new();

    let index1 = pool.intern("duplicate");
    let index2 = pool.intern("duplicate");

    assert_eq!(index1, index2, "Same string should return same index");
    assert_eq!(pool.len(), 1, "Pool should only store string once");
}

#[test]
fn test_string_pool_multiple_strings() {
    let mut pool = StringPool::new();

    let idx_hello = pool.intern("hello");
    let idx_world = pool.intern("world");
    let idx_foo = pool.intern("foo");

    assert_eq!(idx_hello, 0);
    assert_eq!(idx_world, 1);
    assert_eq!(idx_foo, 2);
    assert_eq!(pool.len(), 3);
}

#[test]
fn test_string_pool_get_string() {
    let mut pool = StringPool::new();

    let idx = pool.intern("test_value");

    assert_eq!(pool.get_string(idx), Some("test_value"));
    assert_eq!(
        pool.get_string(99),
        None,
        "Invalid index should return None"
    );
}

#[test]
fn test_string_pool_interleaved_operations() {
    let mut pool = StringPool::new();

    let idx1 = pool.intern("first");
    let idx2 = pool.intern("second");
    let idx1_again = pool.intern("first"); // Duplicate
    let idx3 = pool.intern("third");

    assert_eq!(idx1, 0);
    assert_eq!(idx2, 1);
    assert_eq!(idx1_again, 0, "Duplicate should return original index");
    assert_eq!(idx3, 2);
    assert_eq!(pool.len(), 3, "Should only have 3 unique strings");
}

#[test]
fn test_string_pool_empty_string() {
    let mut pool = StringPool::new();

    let idx = pool.intern("");

    assert_eq!(idx, 0);
    assert_eq!(pool.get_string(idx), Some(""));
}

#[test]
fn test_string_pool_long_string() {
    let mut pool = StringPool::new();

    let long_string = "a".repeat(1000);
    let idx = pool.intern(&long_string);

    assert_eq!(pool.get_string(idx), Some(long_string.as_str()));
}

// ============================================================================
// Test Group 3: Chunk Management
// ============================================================================

#[test]
fn test_chunk_manager_creation() {
    let manager = ChunkManager::new();

    assert!(
        !manager.enabled,
        "Chunking should be disabled by default (only for large programs)"
    );
    assert!(manager.max_segment_size > 0, "Should have max segment size");
    assert_eq!(
        manager.segments.len(),
        0,
        "Should have no segments initially"
    );
}

#[test]
fn test_chunk_manager_should_enable_segmenting() {
    let mut manager = ChunkManager::new();

    // Small bytecode - should not segment (threshold is 200KB)
    let small_result = manager.should_enable_segmenting(1000);
    assert!(
        !small_result,
        "Small bytecode should not trigger segmenting"
    );

    // Medium bytecode - should not segment
    let medium_result = manager.should_enable_segmenting(100_000);
    assert!(
        !medium_result,
        "Medium bytecode (100KB) should not trigger segmenting"
    );

    // Large bytecode - should segment (> 200KB threshold)
    let large_result = manager.should_enable_segmenting(250_000);
    assert!(
        large_result,
        "Large bytecode (>200KB) should trigger segmenting"
    );
}

#[test]
fn test_chunk_manager_create_segments_small() {
    let mut manager = ChunkManager::new();

    let bytecode = vec![0u8; 100]; // Small bytecode
    let result = manager.create_segments(&bytecode);

    assert!(result.is_ok(), "Should successfully handle small bytecode");
    let segments = result.unwrap();

    // Small bytecode should result in single segment or none
    assert!(
        segments.len() <= 1,
        "Small bytecode should not be segmented"
    );
}

#[test]
fn test_chunk_manager_create_segments_large() {
    let mut manager = ChunkManager::new();

    // Create large bytecode (larger than default segment size)
    let bytecode = vec![0u8; 100_000];
    let result = manager.create_segments(&bytecode);

    assert!(result.is_ok(), "Should successfully segment large bytecode");
}

// ============================================================================
// Test Group 4: Pattern Compression
// ============================================================================

#[test]
fn test_pattern_compressor_creation() {
    let compressor = PatternCompressor::new();

    // Should initialize without error
    let _ = compressor;
}

#[test]
fn test_pattern_compressor_analyze_patterns() {
    let mut compressor = PatternCompressor::new();

    let bytecode = vec![1, 2, 3, 1, 2, 3, 4, 5, 6]; // Has repeating pattern [1, 2, 3]
    let result = compressor.analyze_patterns(&bytecode);

    assert!(result.is_ok(), "Should analyze patterns without error");
}

#[test]
fn test_pattern_compressor_compress_with_patterns() {
    let compressor = PatternCompressor::new();

    let bytecode = vec![1, 2, 3, 4, 5];
    let compressed = compressor.compress_with_patterns(&bytecode);

    // Should return some compressed form (may or may not be smaller depending on implementation)
    assert!(
        !compressed.is_empty(),
        "Compressed result should not be empty"
    );
}

#[test]
fn test_pattern_compressor_compression_ratio() {
    let compressor = PatternCompressor::new();

    // Ratio is compressed/original, so 50/100 = 0.5
    let ratio = compressor.get_compression_ratio(100, 50);
    assert!(
        (ratio - 0.5).abs() < 0.01,
        "Ratio should be ~0.5 for 50% size reduction"
    );

    let ratio_identity = compressor.get_compression_ratio(100, 100);
    assert!(
        (ratio_identity - 1.0).abs() < 0.01,
        "Ratio should be 1.0 for no compression"
    );
}

#[test]
fn test_pattern_compressor_compression_ratio_expansion() {
    let compressor = PatternCompressor::new();

    // When compressed size is larger (expansion), ratio > 1.0
    let ratio = compressor.get_compression_ratio(100, 150);
    assert!(
        ratio > 1.0,
        "Ratio should be greater than 1.0 when expansion occurs"
    );
    assert!(
        (ratio - 1.5).abs() < 0.01,
        "Ratio should be ~1.5 for 150% size"
    );
}

// ============================================================================
// Test Group 5: Compression Optimizer
// ============================================================================

#[test]
fn test_compression_optimizer_creation() {
    let optimizer = CompressionOptimizer::new();

    // Should initialize with default settings
    let _ = optimizer;
}

#[test]
fn test_compression_optimizer_configure() {
    let mut optimizer = CompressionOptimizer::new();

    // Test configuration without error
    optimizer.configure(true, true, true);
    optimizer.configure(false, false, false);
}

#[test]
fn test_compression_optimizer_compress_bytecode() {
    let mut optimizer = CompressionOptimizer::new();

    let bytecode = vec![1, 2, 3, 4, 5];
    let result = optimizer.compress_bytecode(&bytecode);

    assert!(result.is_ok(), "Should compress bytecode without error");
}

#[test]
fn test_compression_optimizer_compress_small() {
    let mut optimizer = CompressionOptimizer::new();

    // Use small bytecode (pattern analyzer needs at least 4+ bytes for window)
    let bytecode = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let result = optimizer.compress_bytecode(&bytecode);

    assert!(result.is_ok(), "Should handle small bytecode");
}

#[test]
fn test_compression_optimizer_get_string_pool() {
    let optimizer = CompressionOptimizer::new();

    let pool = optimizer.get_string_pool();
    assert!(
        pool.is_empty(),
        "New optimizer should have empty string pool"
    );
}

#[test]
fn test_compression_optimizer_get_string_pool_mut() {
    let mut optimizer = CompressionOptimizer::new();

    let pool = optimizer.get_string_pool_mut();
    pool.intern("test");

    assert_eq!(
        optimizer.get_string_pool().len(),
        1,
        "Should be able to modify pool"
    );
}

#[test]
fn test_compression_optimizer_get_segment_manager() {
    let optimizer = CompressionOptimizer::new();

    let manager = optimizer.get_segment_manager();
    assert_eq!(
        manager.segments.len(),
        0,
        "New optimizer should have no segments"
    );
}

#[test]
fn test_compression_optimizer_get_pattern_compressor() {
    let optimizer = CompressionOptimizer::new();

    let _compressor = optimizer.get_pattern_compressor();
    // Should be able to get pattern compressor reference
}

// ============================================================================
// Test Group 6: Integration Tests
// ============================================================================

#[test]
fn test_compression_pipeline_small_bytecode() {
    let mut optimizer = CompressionOptimizer::new();

    // Small bytecode
    let bytecode = vec![0x90, 0x02, 0x30, 0x00]; // CALL instruction
    let result = optimizer.compress_bytecode(&bytecode);

    assert!(result.is_ok());
}

#[test]
fn test_compression_pipeline_with_patterns() {
    let mut optimizer = CompressionOptimizer::new();

    // Bytecode with repeating patterns
    let mut bytecode = vec![];
    for _ in 0..10 {
        bytecode.extend_from_slice(&[0x19, 0x64, 0x20]); // Repeat pattern
    }

    let result = optimizer.compress_bytecode(&bytecode);
    assert!(result.is_ok());
}

#[test]
fn test_string_pool_with_optimizer() {
    let mut optimizer = CompressionOptimizer::new();

    let pool = optimizer.get_string_pool_mut();
    pool.intern("function1");
    pool.intern("function2");
    pool.intern("function1"); // Duplicate

    assert_eq!(pool.len(), 2, "Should deduplicate strings");
}

#[test]
fn test_compression_with_all_optimizations() {
    let mut optimizer = CompressionOptimizer::new();

    // Enable all optimizations
    optimizer.configure(true, true, true);

    let bytecode = vec![1, 2, 3, 4, 5, 1, 2, 3, 4, 5]; // Has patterns
    let result = optimizer.compress_bytecode(&bytecode);

    assert!(result.is_ok());
}

#[test]
fn test_compression_with_no_optimizations() {
    let mut optimizer = CompressionOptimizer::new();

    // Disable all optimizations
    optimizer.configure(false, false, false);

    let bytecode = vec![1, 2, 3, 4, 5];
    let result = optimizer.compress_bytecode(&bytecode);

    assert!(result.is_ok());
}
