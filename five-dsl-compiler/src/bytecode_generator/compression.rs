// Compression Module
//
// This module handles all bytecode compression and size optimization techniques.
// It includes VLE encoding, opcode compression, segmenting for large programs,
// pattern-based compression, and data structure optimization.

use super::types::*;
use super::OpcodeEmitter;
use five_protocol::{opcodes::*, Value};
use five_vm_mito::error::VMError;
use std::collections::HashMap;

/// Configuration for Variable Length Encoding (VLE)
#[derive(Debug, Clone)]
pub struct VLEConfig {
    /// Enable VLE for field offsets
    pub enable_field_offsets: bool,
    /// Enable VLE for operand values
    pub enable_operands: bool,
    /// Enable VLE for jump targets
    pub enable_jump_targets: bool,
    /// Minimum value threshold for VLE usage
    pub min_threshold: u32,
}

impl Default for VLEConfig {
    fn default() -> Self {
        Self {
            enable_field_offsets: true,
            enable_operands: true,
            enable_jump_targets: true,
            min_threshold: 128, // Use VLE for values >= 128
        }
    }
}

/// String pool for deduplication and interning
#[derive(Debug, Clone)]
pub struct StringPool {
    /// String to index mapping
    strings: HashMap<String, u8>,
    /// Index to string mapping
    string_table: Vec<String>,
    /// Next available index
    next_index: u8,
}

impl StringPool {
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            string_table: Vec::new(),
            next_index: 0,
        }
    }

    /// Intern a string and return its index
    pub fn intern(&mut self, s: &str) -> u8 {
        if let Some(&index) = self.strings.get(s) {
            return index;
        }

        let index = self.next_index;
        self.strings.insert(s.to_string(), index);
        self.string_table.push(s.to_string());
        self.next_index += 1;
        index
    }

    /// Get string by index
    pub fn get_string(&self, index: u8) -> Option<&str> {
        self.string_table.get(index as usize).map(|s| s.as_str())
    }

    /// Get total number of strings
    pub fn len(&self) -> usize {
        self.string_table.len()
    }

    /// Check if pool is empty
    pub fn is_empty(&self) -> bool {
        self.string_table.is_empty()
    }
}

impl Default for StringPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Chunk management for large programs
#[derive(Debug, Clone)]
pub struct ChunkManager {
    /// Maximum segment size in bytes
    pub max_segment_size: usize,
    /// Chunk metadata
    pub segments: Vec<ChunkInfo>,
    /// Enable segmenting
    pub enabled: bool,
}

/// Information about a program segment
#[derive(Debug, Clone)]
pub struct ChunkInfo {
    /// Chunk identifier
    pub id: u32,
    /// Start offset in original bytecode
    pub start_offset: usize,
    /// End offset in original bytecode
    pub end_offset: usize,
    /// Compressed segment data
    pub compressed_data: Vec<u8>,
    /// Hash for validation
    pub hash: [u8; 32],
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            max_segment_size: 32768, // 32KB segments
            segments: Vec::new(),
            enabled: false, // Only enable for very large programs
        }
    }

    /// Check if segmenting should be enabled for given bytecode size
    pub fn should_enable_segmenting(&mut self, bytecode_size: usize) -> bool {
        let threshold = 200_000; // 200KB threshold
        if bytecode_size > threshold {
            self.enabled = true;
            true
        } else {
            false
        }
    }

    /// Split bytecode into segments
    pub fn create_segments(&mut self, bytecode: &[u8]) -> Result<Vec<ChunkInfo>, VMError> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        self.segments.clear();
        let mut segment_id = 0u32;
        let mut offset = 0;

        while offset < bytecode.len() {
            let end_offset = (offset + self.max_segment_size).min(bytecode.len());
            let segment_data = &bytecode[offset..end_offset];

            // Simple compression (in practice, use a real compression algorithm)
            let compressed_data = self.simple_compress(segment_data);

            // Calculate hash
            let hash = self.calculate_hash(segment_data);

            let segment_info = ChunkInfo {
                id: segment_id,
                start_offset: offset,
                end_offset,
                compressed_data,
                hash,
            };

            self.segments.push(segment_info);
            segment_id += 1;
            offset = end_offset;
        }

        Ok(self.segments.clone())
    }

    /// Simple compression (placeholder for real compression)
    fn simple_compress(&self, data: &[u8]) -> Vec<u8> {
        // Placeholder: just return the data as-is
        // In practice, use LZ4, Zstd, or similar
        data.to_vec()
    }

    /// Calculate hash for segment validation
    fn calculate_hash(&self, data: &[u8]) -> [u8; 32] {
        // Placeholder: use a simple hash
        // In practice, use SHA-256 or Blake3
        let mut hash = [0u8; 32];
        for (i, &byte) in data.iter().enumerate() {
            hash[i % 32] ^= byte;
        }
        hash
    }
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Pattern compressor for common instruction sequences
#[derive(Debug, Clone)]
pub struct PatternCompressor {
    /// Common patterns found in bytecode
    patterns: HashMap<Vec<u8>, u8>,
    /// Next pattern ID
    next_pattern_id: u8,
    /// Pattern usage frequency
    pattern_frequency: HashMap<Vec<u8>, u32>,
}

impl PatternCompressor {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            next_pattern_id: 0,
            pattern_frequency: HashMap::new(),
        }
    }

    /// Analyze bytecode to find common patterns
    pub fn analyze_patterns(&mut self, bytecode: &[u8]) -> Result<(), VMError> {
        // Look for 2-4 byte patterns that repeat frequently
        for pattern_length in 2..=4 {
            for i in 0..=bytecode.len().saturating_sub(pattern_length) {
                let pattern = bytecode[i..i + pattern_length].to_vec();
                *self.pattern_frequency.entry(pattern).or_insert(0) += 1;
            }
        }

        // Keep patterns that appear at least 3 times
        let frequent_patterns: Vec<_> = self
            .pattern_frequency
            .iter()
            .filter(|(_, &count)| count >= 3)
            .map(|(pattern, _)| pattern.clone())
            .collect();

        // Assign IDs to frequent patterns
        for pattern in frequent_patterns {
            if self.patterns.len() < 256 {
                self.patterns.insert(pattern, self.next_pattern_id);
                self.next_pattern_id += 1;
            }
        }

        Ok(())
    }

    /// Compress bytecode using identified patterns
    pub fn compress_with_patterns(&self, bytecode: &[u8]) -> Vec<u8> {
        if self.patterns.is_empty() {
            return bytecode.to_vec();
        }

        let mut compressed = Vec::new();
        let mut i = 0;

        while i < bytecode.len() {
            let mut found_pattern = false;

            // Try to match patterns in order of length (longest first)
            for pattern_length in (2..=4).rev() {
                if i + pattern_length <= bytecode.len() {
                    let potential_pattern = &bytecode[i..i + pattern_length];

                    if let Some(&pattern_id) = self.patterns.get(potential_pattern) {
                        // Emit pattern compression opcode
                        compressed.push(OP_PATTERN);
                        compressed.push(pattern_id);
                        i += pattern_length;
                        found_pattern = true;
                        break;
                    }
                }
            }

            if !found_pattern {
                compressed.push(bytecode[i]);
                i += 1;
            }
        }

        compressed
    }

    /// Get compression ratio achieved
    pub fn get_compression_ratio(&self, original_size: usize, compressed_size: usize) -> f32 {
        if original_size == 0 {
            1.0
        } else {
            compressed_size as f32 / original_size as f32
        }
    }
}

impl Default for PatternCompressor {
    fn default() -> Self {
        Self::new()
    }
}

/// Main compression optimizer
pub struct CompressionOptimizer {
    /// VLE encoding configuration
    vle_config: VLEConfig,
    /// Chunk management for large programs
    segment_manager: ChunkManager,
    /// String interning for deduplication
    string_pool: StringPool,
    /// Opcode pattern compression
    pattern_compressor: PatternCompressor,
    /// Compression feature flags
    enable_vle_encoding: bool,
    enable_compact_fields: bool,
    enable_instruction_compression: bool,
}

impl CompressionOptimizer {
    /// Create a new compression optimizer
    pub fn new() -> Self {
        Self {
            vle_config: VLEConfig::default(),
            segment_manager: ChunkManager::new(),
            string_pool: StringPool::new(),
            pattern_compressor: PatternCompressor::new(),
            enable_vle_encoding: true,
            enable_compact_fields: true,
            enable_instruction_compression: true,
        }
    }

    /// Configure compression features
    pub fn configure(&mut self, vle: bool, compact_fields: bool, instruction_compression: bool) {
        self.enable_vle_encoding = vle;
        self.enable_compact_fields = compact_fields;
        self.enable_instruction_compression = instruction_compression;
    }

    /// Main compression orchestrator
    pub fn compress_bytecode(&mut self, bytecode: &[u8]) -> Result<Vec<u8>, VMError> {
        let mut compressed = bytecode.to_vec();

        // Phase 1: Pattern-based compression
        if self.enable_instruction_compression {
            self.pattern_compressor.analyze_patterns(&compressed)?;
            compressed = self.pattern_compressor.compress_with_patterns(&compressed);
        }

        // Phase 2: Check if segmenting is needed for large programs
        if self
            .segment_manager
            .should_enable_segmenting(compressed.len())
        {
            let segments = self.segment_manager.create_segments(&compressed)?;
            // For segmented programs, return segment manifest instead of full bytecode
            return self.create_segment_manifest(segments);
        }

        // Phase 3: VLE encoding optimization (would be applied during generation)
        // This is handled by emit_vle_* methods during bytecode generation

        Ok(compressed)
    }

    /// Create segment manifest for large programs
    fn create_segment_manifest(&self, segments: Vec<ChunkInfo>) -> Result<Vec<u8>, VMError> {
        let mut manifest = Vec::new();

        // Manifest header
        manifest.extend_from_slice(b"CHNK"); // Chunk magic
        manifest.push(segments.len() as u8);

        // Chunk entries
        for segment in segments {
            manifest.extend_from_slice(&segment.id.to_le_bytes());
            manifest.extend_from_slice(&(segment.start_offset as u32).to_le_bytes());
            manifest.extend_from_slice(&(segment.end_offset as u32).to_le_bytes());
            manifest.extend_from_slice(&(segment.compressed_data.len() as u32).to_le_bytes());
            manifest.extend_from_slice(&segment.hash);
        }

        Ok(manifest)
    }

    /// Emit Variable Length Encoded u32
    pub fn emit_vle_u32<T: OpcodeEmitter>(&self, emitter: &mut T, value: u32) {
        if !self.enable_vle_encoding || value < self.vle_config.min_threshold {
            // Use standard u32 encoding
            emitter.emit_u32(value);
            return;
        }

        // VLE encoding: use continuation bit in MSB
        if value < 0x80 {
            // Single byte for values 0-127
            emitter.emit_u8(value as u8);
        } else if value < 0x4000 {
            // Two bytes for values 128-16383
            emitter.emit_u8(0x80 | (value as u8 & 0x7F));
            emitter.emit_u8((value >> 7) as u8);
        } else if value < 0x200000 {
            // Three bytes for values 16384-2097151
            emitter.emit_u8(0x80 | (value as u8 & 0x7F));
            emitter.emit_u8(0x80 | ((value >> 7) as u8 & 0x7F));
            emitter.emit_u8((value >> 14) as u8);
        } else {
            // Four bytes for larger values
            emitter.emit_u8(0x80 | (value as u8 & 0x7F));
            emitter.emit_u8(0x80 | ((value >> 7) as u8 & 0x7F));
            emitter.emit_u8(0x80 | ((value >> 14) as u8 & 0x7F));
            emitter.emit_u8((value >> 21) as u8);
        }
    }

    /// Emit field offset using VLE if beneficial
    pub fn emit_field_offset<T: OpcodeEmitter>(&self, emitter: &mut T, offset: u32) {
        if self.enable_vle_encoding && self.vle_config.enable_field_offsets {
            self.emit_vle_u32(emitter, offset);
        } else {
            emitter.emit_u32(offset);
        }
    }

    /// Emit compact field operation if beneficial.
    ///
    /// Note: emit_compact_field_load and emit_compact_field_store were REMOVED
    /// because standard LOAD_FIELD/STORE_FIELD already use VLE + zero-copy.
    ///
    /// Check if field ID is a built-in field suitable for compact operations
    #[allow(dead_code)]
    fn is_builtin_field(&self, field_id: u8) -> bool {
        matches!(
            field_id,
            FIELD_LAMPORTS | FIELD_OWNER | FIELD_KEY | FIELD_DATA
        )
    }

    /// Apply bulk operation optimization for expression patterns
    pub fn optimize_bulk_expressions<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        expressions: &[crate::ast::AstNode],
    ) -> Result<bool, VMError> {
        if !self.enable_instruction_compression {
            return Ok(false);
        }

        match expressions.len() {
            2 => {
                // Two expressions - check if both are simple literals
                if expressions.iter().all(|e| self.is_simple_literal(e)) {
                    // Use bulk two operation
                    self.emit_bulk_two_literals(emitter, &expressions[0], &expressions[1])?;
                    return Ok(true);
                }
            }
            3 => {
                // Three expressions - check if all are simple literals
                if expressions.iter().all(|e| self.is_simple_literal(e)) {
                    // Use bulk three operation
                    self.emit_bulk_three_literals(
                        emitter,
                        &expressions[0],
                        &expressions[1],
                        &expressions[2],
                    )?;
                    return Ok(true);
                }
            }
            _ => {}
        }

        Ok(false)
    }

    /// Check if expression is a simple literal suitable for bulk operations
    fn is_simple_literal(&self, expr: &crate::ast::AstNode) -> bool {
        matches!(expr, crate::ast::AstNode::Literal(_))
    }

    /// Emit bulk operation for two literals
    fn emit_bulk_two_literals<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        expr1: &crate::ast::AstNode,
        expr2: &crate::ast::AstNode,
    ) -> Result<(), VMError> {
        if let (crate::ast::AstNode::Literal(val1), crate::ast::AstNode::Literal(val2)) =
            (expr1, expr2)
        {
            emitter.emit_opcode(BULK_PUSH_2);
            self.emit_compressed_value(emitter, val1)?;
            self.emit_compressed_value(emitter, val2)?;
        }
        Ok(())
    }

    /// Emit bulk operation for three literals
    fn emit_bulk_three_literals<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        expr1: &crate::ast::AstNode,
        expr2: &crate::ast::AstNode,
        expr3: &crate::ast::AstNode,
    ) -> Result<(), VMError> {
        if let (
            crate::ast::AstNode::Literal(val1),
            crate::ast::AstNode::Literal(val2),
            crate::ast::AstNode::Literal(val3),
        ) = (expr1, expr2, expr3)
        {
            emitter.emit_opcode(BULK_PUSH_3);
            self.emit_compressed_value(emitter, val1)?;
            self.emit_compressed_value(emitter, val2)?;
            self.emit_compressed_value(emitter, val3)?;
        }
        Ok(())
    }

    /// Emit compressed value using optimal encoding
    fn emit_compressed_value<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        value: &Value,
    ) -> Result<(), VMError> {
        match value {
            Value::U64(n) if *n < 256 => {
                // Use compact u8 encoding for small values
                emitter.emit_u8(five_protocol::types::U8);
                emitter.emit_u8(*n as u8);
            }
            Value::U64(n) => {
                emitter.emit_u8(five_protocol::types::U64);
                if self.enable_vle_encoding {
                    self.emit_vle_u32(emitter, *n as u32);
                } else {
                    emitter.emit_u64(*n);
                }
            }
            Value::Bool(b) => {
                emitter.emit_u8(five_protocol::types::BOOL);
                emitter.emit_u8(if *b { 1 } else { 0 });
            }
            Value::U8(n) => {
                emitter.emit_u8(five_protocol::types::U8);
                emitter.emit_u8(*n);
            }
            Value::String(idx) => {
                emitter.emit_u8(five_protocol::types::STRING);
                emitter.emit_u8(*idx);
            }
            Value::Pubkey(key) => {
                emitter.emit_u8(five_protocol::types::PUBKEY);
                emitter.emit_bytes(key);
            }
            Value::I64(n) => {
                emitter.emit_u8(five_protocol::types::I64);
                emitter.emit_u64(*n as u64);
            }
            Value::U128(n) => {
                emitter.emit_u8(five_protocol::types::U128);
                // Emit as 16 bytes in little-endian format
                emitter.emit_bytes(&n.to_le_bytes());
            }
            Value::Account(idx) => {
                emitter.emit_u8(five_protocol::types::ACCOUNT);
                emitter.emit_u8(*idx);
            }
            Value::Array(idx) => {
                emitter.emit_u8(five_protocol::types::ARRAY);
                emitter.emit_u8(*idx);
            }
            Value::Empty => {
                return Err(VMError::TypeMismatch);
            }
        }
        Ok(())
    }

    /// Generate compression report
    pub fn generate_compression_report(
        &self,
        original_size: usize,
        compressed_size: usize,
    ) -> String {
        let mut report = String::new();
        report.push_str("Compression Optimization Report\n");
        report.push_str("===============================\n\n");

        report.push_str(&format!("Original size: {} bytes\n", original_size));
        report.push_str(&format!("Compressed size: {} bytes\n", compressed_size));

        let ratio = if original_size > 0 {
            compressed_size as f32 / original_size as f32
        } else {
            1.0
        };

        report.push_str(&format!(
            "Compression ratio: {:.2}% ({:.2}x)\n",
            ratio * 100.0,
            1.0 / ratio
        ));

        report.push_str(&format!(
            "Space saved: {} bytes\n",
            original_size.saturating_sub(compressed_size)
        ));

        report.push_str("\nEnabled optimizations:\n");
        report.push_str(&format!("  VLE encoding: {}\n", self.enable_vle_encoding));
        report.push_str(&format!(
            "  Compact fields: {}\n",
            self.enable_compact_fields
        ));
        report.push_str(&format!(
            "  Instruction compression: {}\n",
            self.enable_instruction_compression
        ));
        report.push_str(&format!(
            "  Pattern compression: {} patterns\n",
            self.pattern_compressor.patterns.len()
        ));

        if self.segment_manager.enabled {
            report.push_str(&format!(
                "  Chunking: {} segments\n",
                self.segment_manager.segments.len()
            ));
        }

        report
    }

    /// Get string pool reference
    pub fn get_string_pool(&self) -> &StringPool {
        &self.string_pool
    }

    /// Get mutable string pool reference
    pub fn get_string_pool_mut(&mut self) -> &mut StringPool {
        &mut self.string_pool
    }

    /// Get segment manager reference
    pub fn get_segment_manager(&self) -> &ChunkManager {
        &self.segment_manager
    }

    /// Get pattern compressor reference
    pub fn get_pattern_compressor(&self) -> &PatternCompressor {
        &self.pattern_compressor
    }
}

impl Default for CompressionOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension methods for the main DslBytecodeGenerator
impl super::DslBytecodeGenerator {
    /// Initialize compression optimizer
    pub fn init_compression(&mut self) -> CompressionOptimizer {
        let mut optimizer = CompressionOptimizer::new();
        optimizer.configure(
            self.enable_vle_encoding,
            self.enable_compact_fields,
            self.enable_instruction_compression,
        );
        optimizer
    }

    /// Apply compression optimizations to bytecode
    pub fn apply_compression_optimizations(&mut self) -> Result<Vec<u8>, VMError> {
        let mut optimizer = self.init_compression();
        optimizer.compress_bytecode(&self.bytecode)
    }

    /// Get compression report for generated bytecode
    pub fn get_compression_report(&self) -> Result<String, VMError> {
        let mut optimizer = CompressionOptimizer::new();
        let compressed = optimizer.compress_bytecode(&self.bytecode)?;
        Ok(optimizer.generate_compression_report(self.bytecode.len(), compressed.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vle_encoding() {
        let optimizer = CompressionOptimizer::new();

        struct TestEmitter(Vec<u8>);
        impl OpcodeEmitter for TestEmitter {
            fn emit_opcode(&mut self, opcode: u8) {
                self.0.push(opcode);
            }
            fn emit_u8(&mut self, value: u8) {
                self.0.push(value);
            }
            fn emit_u16(&mut self, value: u16) {
                self.0.extend_from_slice(&value.to_le_bytes());
            }
            fn emit_u32(&mut self, value: u32) {
                self.0.extend_from_slice(&value.to_le_bytes());
            }
            fn emit_u64(&mut self, value: u64) {
                self.0.extend_from_slice(&value.to_le_bytes());
            }
            fn emit_bytes(&mut self, bytes: &[u8]) {
                self.0.extend_from_slice(bytes);
            }
            fn emit_vle_u32(&mut self, value: u32) {
                // Simple VLE implementation for testing
                if value < 128 {
                    self.0.push(value as u8);
                } else {
                    self.0.extend_from_slice(&value.to_le_bytes());
                }
            }
            fn emit_vle_u16(&mut self, value: u16) {
                // Simple VLE implementation for testing
                if value < 128 {
                    self.0.push(value as u8);
                } else {
                    self.0.extend_from_slice(&value.to_le_bytes());
                }
            }
            fn emit_vle_u64(&mut self, value: u64) {
                // Simple VLE implementation for testing
                if value < 128 {
                    self.0.push(value as u8);
                } else {
                    self.0.extend_from_slice(&value.to_le_bytes());
                }
            }
            fn patch_u32(&mut self, position: usize, value: u32) {
                if position + 4 <= self.0.len() {
                    let bytes = value.to_le_bytes();
                    self.0[position..position + 4].copy_from_slice(&bytes);
                }
            }
            fn patch_u16(&mut self, position: usize, value: u16) {
                if position + 2 <= self.0.len() {
                    let bytes = value.to_le_bytes();
                    self.0[position..position + 2].copy_from_slice(&bytes);
                }
            }
            fn should_include_tests(&self) -> bool {
                false
            }
            fn get_position(&self) -> usize {
                self.0.len()
            }
        }

        let mut test_emitter = TestEmitter(Vec::new());

        // Test small value (below threshold, uses standard 4-byte encoding)
        optimizer.emit_vle_u32(&mut test_emitter, 50);
        assert_eq!(test_emitter.0.len(), 4);

        // Test larger value (should use multiple bytes)
        test_emitter.0.clear();
        optimizer.emit_vle_u32(&mut test_emitter, 200);
        assert!(test_emitter.0.len() > 1);
    }

    #[test]
    fn test_string_pool() {
        let mut pool = StringPool::new();

        let idx1 = pool.intern("hello");
        let idx2 = pool.intern("world");
        let idx3 = pool.intern("hello"); // Should reuse existing

        assert_eq!(idx1, idx3);
        assert_ne!(idx1, idx2);
        assert_eq!(pool.len(), 2);
        assert_eq!(pool.get_string(idx1), Some("hello"));
        assert_eq!(pool.get_string(idx2), Some("world"));
    }

    #[test]
    fn test_pattern_compressor() {
        let mut compressor = PatternCompressor::new();

        // Test pattern with repeated sequences
        let bytecode = vec![1, 2, 3, 1, 2, 3, 4, 5, 1, 2, 3];
        compressor.analyze_patterns(&bytecode).unwrap();

        // Should find pattern [1, 2, 3] appears 3 times
        assert!(!compressor.patterns.is_empty());
    }

    #[test]
    fn test_segment_manager() {
        let mut manager = ChunkManager::new();

        // Test small bytecode (should not enable segmenting)
        assert!(!manager.should_enable_segmenting(1000));

        // Test large bytecode (should enable segmenting)
        assert!(manager.should_enable_segmenting(300_000));
    }
}
