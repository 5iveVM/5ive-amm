//! Type definitions for bytecode instructions and metadata.

/// Metadata describing a discovered PUSH-like instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushInfo {
    /// Offset of the opcode byte in the stream.
    pub offset: usize,
    /// Opcode value (one of PUSH_U8/PUSH_U16/PUSH_U32/PUSH_U64/PUSH_I64/etc).
    pub opcode: u8,
    /// Normalized numeric value (u64 where possible).
    pub value: u64,
    /// Number of immediate bytes consumed (not counting opcode).
    pub width: usize,
}

/// Parsed metadata for a CALL instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallSite {
    pub offset: usize,
    pub param_count: u8,
    pub function_address: u16,
    /// Optional inline function name or name reference description.
    pub name_metadata: Option<String>,
}

/// Structured view of a decoded instruction at a specific offset.
///
/// This gives callers programmatic access to instruction semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    PushU64(PushInfo),
    Call(CallSite),
    SetLocal {
        offset: usize,
        index: u8,
    },
    GetLocal {
        offset: usize,
        index: u8,
    },
    AllocLocals {
        offset: usize,
    },
    DeallocLocals {
        offset: usize,
    },
    LoadField {
        instr_offset: usize,
        account_index: u8,
        field_offset: u32,
    },
    StoreField {
        instr_offset: usize,
        account_index: u8,
        field_offset: u32,
    },
    CallNative {
        offset: usize,
        syscall_id: u8,
    },
    Invoke {
        offset: usize,
    },
    InvokeSigned {
        offset: usize,
    },
    PushStringLiteral {
        offset: usize,
        len: usize,
    },
    PushArrayLiteral {
        offset: usize,
        len: usize,
    },
    CheckSigner {
        offset: usize,
        account_index: u8,
    },
    CheckWritable {
        offset: usize,
        account_index: u8,
    },
    GetKey {
        offset: usize,
        account_index: u8,
    },
    Opcode(u8),
    Unknown,
}
