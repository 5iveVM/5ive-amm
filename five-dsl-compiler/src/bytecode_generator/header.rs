use super::DslBytecodeGenerator;
use five_protocol::{OptimizedHeader, FEATURE_IMPORT_VERIFICATION};

impl DslBytecodeGenerator {
    /// Emit 5IVE magic bytes at the beginning (legacy V1 format)
    pub fn emit_magic_bytes(&mut self) {
        self.emit_bytes(b"5IVE");
    }

    /// Emit optimized V2 header with explicit public/total function counts.
    ///
    /// Header layout (10 bytes):
    /// 0..4   - magic "5IVE"
    /// 4..8   - features (u32 little-endian)
    /// 8      - public_function_count (u8)
    /// 9      - total_function_count (u8)
    pub fn emit_optimized_header_v2_with_imports(&mut self, public_count: u8, total_count: u8, has_imports: bool) {
        // Build feature bitmap as u32 (feature bit tells the VM that CALL opcodes embed metadata directly after the instruction).
        let mut production_features = five_protocol::FEATURE_FUSED_BRANCH
            | five_protocol::FEATURE_NO_VALIDATION
            | five_protocol::FEATURE_MINIMAL_ERRORS
            | five_protocol::FEATURE_COLD_START_OPT;

        #[cfg(feature = "call-metadata")]
        {
            production_features |= five_protocol::FEATURE_FUNCTION_METADATA;
        }

        if public_count > 0 && self.include_debug_info {
            production_features |= five_protocol::FEATURE_FUNCTION_NAMES;
        }

        // NEW: Add FEATURE_IMPORT_VERIFICATION flag if imports exist
        if has_imports {
            production_features |= FEATURE_IMPORT_VERIFICATION;
        }

        self.log_header(
            "OptimizedHeaderV2",
            &format!(
                "10 bytes: magic='5IVE', features=0x{:08X}, public_functions={}, total_functions={}",
                production_features, public_count, total_count
            ),
        );

        // Build header struct (in-memory representation)
        let header = OptimizedHeader {
            magic: [b'5', b'I', b'V', b'E'],
            features: production_features,
            public_function_count: public_count,
            total_function_count: total_count,
        };

        // Emit header bytes according to agreed layout
        self.emit_bytes(&header.magic); // 0..4
        self.emit_u32(header.features); // 4..8 (u32 le)
        self.emit_u8(header.public_function_count); // 8
        self.emit_u8(header.total_function_count); // 9

        // Logging helpers for diagnostics
        self.log_opcode("MAGIC", "Five VM bytecode magic bytes '5IVE'");
        self.log_opcode_with_params(
            "FEATURES",
            &format!("0x{:08X}", header.features),
            "Production optimizations: fused_branch, no_validation, minimal_errors, cold_start",
        );
        self.log_opcode_with_params(
            "PUBLIC_FUNC_COUNT",
            &format!("{}", header.public_function_count),
            "Number of public functions exposed by this script",
        );
        self.log_opcode_with_params(
            "TOTAL_FUNC_COUNT",
            &format!("{}", header.total_function_count),
            "Total number of functions in this script",
        );
    }

    /// Legacy wrapper for backward compatibility
    /// Calls emit_optimized_header_v2_with_imports with has_imports = false
    pub fn emit_optimized_header_v2(&mut self, public_count: u8, total_count: u8) {
        self.emit_optimized_header_v2_with_imports(public_count, total_count, false);
    }
}
