//! Error types for the MitoVM with enhanced function call support

use heapless::{String as HString, Vec as HVec};
use pinocchio::program_error::ProgramError;

type Str16 = HString<16>;
type Str32 = HString<32>;
type Str64 = HString<64>;
type Vec8<T> = HVec<T, 8>;

/// Standard result type for all VM operations.
pub type Result<T> = std::result::Result<T, VMError>;

/// Compact result type for hot paths where error size matters.
/// Uses 1-byte error codes instead of full VMError.
pub type CompactResult<T> = std::result::Result<T, VMErrorCode>;

/// Compact error code for on-chain execution (1 byte).
///
/// This enum provides ultra-compact error representation for on-chain
/// VM execution where every byte matters. Use this in hot paths and
/// convert to `VMError` only when rich context is needed.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VMErrorCode {
    /// Stack error
    StackError = 1,
    /// Stack overflow
    StackOverflow = 59,
    /// Stack underflow
    StackUnderflow = 60,
    /// Invalid instruction or bytecode
    InvalidInstruction = 2,
    /// Invalid script format or magic bytes
    InvalidScript = 3,
    /// Script exceeds maximum allowed size
    InvalidScriptSize = 4,
    /// Memory access violation
    MemoryViolation = 5,
    /// Type mismatch in operation
    TypeMismatch = 6,
    /// Division by zero
    DivisionByZero = 7,
    /// Numeric overflow when narrowing u128 to u64
    NumericOverflow = 8,
    /// Arithmetic overflow/underflow in checked operations
    ArithmeticOverflow = 9,
    /// Account access error
    AccountError = 10,
    /// Constraint validation failed
    ConstraintViolation = 11,
    /// Script execution halted
    Halted = 12,
    /// Invalid account index
    InvalidAccountIndex = 13,
    /// Account not writable when write attempted
    AccountNotWritable = 14,
    /// Account not signer when signature required
    AccountNotSigner = 15,
    /// Invalid variable index
    InvalidVariableIndex = 16,
    /// Parameter count mismatch
    ParameterMismatch = 17,
    /// Stack operation error
    StackOperationError = 18,
    /// ABI parameter mismatch
    AbiParameterMismatch = 19,
    /// Instruction pointer out of bounds
    InvalidInstructionPointer = 20,
    /// Function call stack overflow
    CallStackOverflow = 21,
    /// Function call stack underflow
    CallStackUnderflow = 22,
    /// Data buffer overflow
    DataBufferOverflow = 23,
    /// Invalid operation
    InvalidOperation = 25,
    /// Parse error during compilation
    ParseError = 26,
    /// Unexpected token during parsing
    UnexpectedToken = 27,
    /// Unexpected end of input
    UnexpectedEndOfInput = 28,
    /// Invalid function index
    InvalidFunctionIndex = 29,
    /// Too many local variables
    LocalsOverflow = 30,
    /// Invalid account data operation
    InvalidAccountData = 31,
    /// Invalid account reference
    InvalidAccount = 32,
    /// Memory allocation or access error
    MemoryError = 33,
    /// Account ownership error
    AccountOwnershipError = 34,
    /// Cross-program invocation failed
    InvokeError = 35,
    /// External account lamport spend without signature
    ExternalAccountLamportSpend = 36,
    /// Script not authorized to access account
    ScriptNotAuthorized = 37,
    /// Undefined account field access
    UndefinedAccountField = 38,
    /// Invalid seed array
    InvalidSeedArray = 39,
    /// Attempt to modify immutable field
    ImmutableField = 40,
    /// Function visibility violation
    FunctionVisibilityViolation = 41,
    /// Undefined field access
    UndefinedField = 42,
    /// Undefined identifier
    UndefinedIdentifier = 43,
    /// Invalid parameter count
    InvalidParameterCount = 44,
    /// Index out of bounds
    IndexOutOfBounds = 45,
    /// Out of memory
    OutOfMemory = 46,
    /// Protocol error
    ProtocolError = 47,
    /// Too many PDA seeds
    TooManySeeds = 48,
    /// Unauthorized bytecode invocation
    UnauthorizedBytecodeInvocation = 49,
    /// PDA derivation failed
    PdaDerivationFailed = 50,
    /// Security violation detected
    SecurityViolation = 51,
    /// Account not found
    AccountNotFound = 52,
    /// Account data is empty
    AccountDataEmpty = 53,
    /// Runtime integration required
    RuntimeIntegrationRequired = 54,
    /// Invalid parameter
    InvalidParameter = 55,
    /// Invalid opcode
    InvalidOpcode = 56,
    /// Execution terminated
    ExecutionTerminated = 57,
    /// Uninitialized account
    UninitializedAccount = 58,
}

impl VMErrorCode {
    /// Convert error code to static error message
    pub const fn message(self) -> &'static str {
        match self {
            Self::StackError => "Stack error",
            Self::StackOverflow => "Stack overflow",
            Self::StackUnderflow => "Stack underflow",
            Self::InvalidInstruction => "Invalid instruction or bytecode",
            Self::InvalidScript => "Invalid script format or magic bytes",
            Self::InvalidScriptSize => "Script exceeds maximum allowed size",
            Self::MemoryViolation => "Memory access violation",
            Self::TypeMismatch => "Type mismatch in operation",
            Self::DivisionByZero => "Division by zero",
            Self::NumericOverflow => "Numeric overflow when narrowing u128 to u64",
            Self::ArithmeticOverflow => "Arithmetic overflow in checked operation",
            Self::AccountError => "Account access error",
            Self::ConstraintViolation => "Constraint validation failed",
            Self::Halted => "Script execution halted",
            Self::InvalidAccountIndex => "Invalid account index",
            Self::AccountNotWritable => "Account not writable when write attempted",
            Self::AccountNotSigner => "Account not signer when signature required",
            Self::InvalidVariableIndex => "Invalid variable index",
            Self::ParameterMismatch => "Parameter count mismatch",
            Self::StackOperationError => "Stack operation error",
            Self::AbiParameterMismatch => "ABI parameter mismatch",
            Self::InvalidInstructionPointer => "Instruction pointer out of bounds",
            Self::CallStackOverflow => "Function call stack overflow",
            Self::CallStackUnderflow => "Function call stack underflow",
            Self::DataBufferOverflow => "Data buffer overflow",
            Self::InvalidOperation => "Invalid operation",
            Self::ParseError => "Parse error during compilation",
            Self::UnexpectedToken => "Unexpected token during parsing",
            Self::UnexpectedEndOfInput => "Unexpected end of input",
            Self::InvalidFunctionIndex => "Invalid function index",
            Self::LocalsOverflow => "Too many local variables",
            Self::InvalidAccountData => "Invalid account data operation",
            Self::InvalidAccount => "Invalid account reference",
            Self::MemoryError => "Memory allocation or access error",
            Self::AccountOwnershipError => "Account ownership error",
            Self::InvokeError => "Cross-program invocation failed",
            Self::ExternalAccountLamportSpend => "External account lamport spend without signature",
            Self::ScriptNotAuthorized => "Script not authorized to access account",
            Self::UndefinedAccountField => "Undefined account field access",
            Self::InvalidSeedArray => "Invalid seed array",
            Self::ImmutableField => "Attempt to modify immutable field",
            Self::FunctionVisibilityViolation => "Function visibility violation",
            Self::UndefinedField => "Undefined field access",
            Self::UndefinedIdentifier => "Undefined identifier",
            Self::InvalidParameterCount => "Invalid parameter count",
            Self::IndexOutOfBounds => "Index out of bounds",
            Self::OutOfMemory => "Out of memory",
            Self::ProtocolError => "Protocol error",
            Self::TooManySeeds => "Too many PDA seeds",
            Self::UnauthorizedBytecodeInvocation => "Unauthorized bytecode invocation",
            Self::PdaDerivationFailed => "PDA derivation failed",
            Self::SecurityViolation => "Security violation detected",
            Self::AccountNotFound => "Account not found",
            Self::AccountDataEmpty => "Account data is empty",
            Self::RuntimeIntegrationRequired => "Runtime integration required",
            Self::InvalidParameter => "Invalid parameter",
            Self::InvalidOpcode => "Invalid opcode",
            Self::ExecutionTerminated => "Execution terminated",
            Self::UninitializedAccount => "Uninitialized account",
        }
    }
}

impl std::fmt::Display for VMErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for VMErrorCode {}

impl From<VMErrorCode> for ProgramError {
    fn from(code: VMErrorCode) -> Self {
        match code {
            VMErrorCode::StackError => ProgramError::InvalidInstructionData,
            VMErrorCode::StackOverflow => ProgramError::InvalidInstructionData,
            VMErrorCode::StackUnderflow => ProgramError::InvalidInstructionData,
            VMErrorCode::InvalidInstruction => ProgramError::InvalidInstructionData,
            VMErrorCode::InvalidScript => ProgramError::InvalidInstructionData,
            VMErrorCode::InvalidScriptSize => ProgramError::InvalidInstructionData,
            VMErrorCode::MemoryViolation => ProgramError::Custom(9001),
            VMErrorCode::TypeMismatch => ProgramError::InvalidInstructionData,
            VMErrorCode::DivisionByZero => ProgramError::InvalidInstructionData,
            VMErrorCode::NumericOverflow => ProgramError::ArithmeticOverflow,
            VMErrorCode::ArithmeticOverflow => ProgramError::ArithmeticOverflow,
            VMErrorCode::AccountError => ProgramError::Custom(9002),
            VMErrorCode::ConstraintViolation => ProgramError::Custom(9003),
            VMErrorCode::Halted => ProgramError::InvalidInstructionData,
            VMErrorCode::InvalidAccountIndex => ProgramError::NotEnoughAccountKeys,
            VMErrorCode::AccountNotWritable => ProgramError::Custom(9004),
            VMErrorCode::AccountNotSigner => ProgramError::MissingRequiredSignature,
            VMErrorCode::InvalidVariableIndex => ProgramError::InvalidInstructionData,
            VMErrorCode::ParameterMismatch => ProgramError::Custom(9014),
            VMErrorCode::StackOperationError => ProgramError::InvalidInstructionData,
            VMErrorCode::AbiParameterMismatch => ProgramError::Custom(9015),
            VMErrorCode::InvalidInstructionPointer => ProgramError::InvalidInstructionData,
            VMErrorCode::CallStackOverflow => ProgramError::InvalidInstructionData,
            VMErrorCode::CallStackUnderflow => ProgramError::InvalidInstructionData,
            VMErrorCode::DataBufferOverflow => ProgramError::Custom(9005),
            VMErrorCode::InvalidOperation => ProgramError::InvalidInstructionData,
            VMErrorCode::ParseError => ProgramError::InvalidInstructionData,
            VMErrorCode::UnexpectedToken => ProgramError::InvalidInstructionData,
            VMErrorCode::UnexpectedEndOfInput => ProgramError::InvalidInstructionData,
            VMErrorCode::InvalidFunctionIndex => ProgramError::InvalidInstructionData,
            VMErrorCode::LocalsOverflow => ProgramError::InvalidInstructionData,
            VMErrorCode::InvalidAccountData => ProgramError::Custom(9006),
            VMErrorCode::InvalidAccount => ProgramError::Custom(9007),
            VMErrorCode::MemoryError => ProgramError::Custom(9008),
            VMErrorCode::AccountOwnershipError => ProgramError::Custom(1100),
            VMErrorCode::InvokeError => ProgramError::Custom(1103),
            VMErrorCode::ExternalAccountLamportSpend => ProgramError::Custom(1104),
            VMErrorCode::ScriptNotAuthorized => ProgramError::Custom(1105),
            VMErrorCode::UndefinedAccountField => ProgramError::Custom(9009),
            VMErrorCode::InvalidSeedArray => ProgramError::InvalidArgument,
            VMErrorCode::ImmutableField => ProgramError::Custom(9010),
            VMErrorCode::FunctionVisibilityViolation => ProgramError::InvalidInstructionData,
            VMErrorCode::UndefinedField => ProgramError::Custom(9011),
            VMErrorCode::UndefinedIdentifier => ProgramError::InvalidInstructionData,
            VMErrorCode::InvalidParameterCount => ProgramError::Custom(9014),
            VMErrorCode::IndexOutOfBounds => ProgramError::InvalidInstructionData,
            VMErrorCode::OutOfMemory => ProgramError::Custom(9012),
            VMErrorCode::ProtocolError => ProgramError::InvalidInstructionData,
            VMErrorCode::TooManySeeds => ProgramError::InvalidArgument,
            VMErrorCode::UnauthorizedBytecodeInvocation => ProgramError::Custom(1110),
            VMErrorCode::PdaDerivationFailed => ProgramError::InvalidSeeds,
            VMErrorCode::SecurityViolation => ProgramError::Custom(1109),
            VMErrorCode::AccountNotFound => ProgramError::InvalidArgument,
            VMErrorCode::AccountDataEmpty => ProgramError::Custom(9013),
            VMErrorCode::RuntimeIntegrationRequired => ProgramError::Custom(1106),
            VMErrorCode::InvalidParameter => ProgramError::InvalidArgument,
            VMErrorCode::InvalidOpcode => ProgramError::InvalidInstructionData,
            VMErrorCode::ExecutionTerminated => ProgramError::Custom(1108),
            VMErrorCode::UninitializedAccount => ProgramError::UninitializedAccount,
        }
    }
}

/// Comprehensive error types for VM execution failures including detailed context for debugging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VMError {
    /// Stack overflow or underflow
    StackError,
    /// Invalid instruction or bytecode
    InvalidInstruction,
    /// Invalid script format or magic bytes
    InvalidScript,
    /// Script exceeds maximum allowed size
    InvalidScriptSize,
    /// Memory access violation
    MemoryViolation,
    /// Type mismatch in operation
    TypeMismatch,
    /// Division by zero
    DivisionByZero,
    /// Numeric overflow when narrowing u128 to u64
    NumericOverflow,
    /// Arithmetic overflow/underflow in checked operations (ADD_CHECKED, SUB_CHECKED, MUL_CHECKED)
    ArithmeticOverflow,
    /// Account access error
    AccountError,
    /// Constraint validation failed
    ConstraintViolation,
    /// Script execution halted
    Halted,
    /// Invalid account index (legacy - specific version below)
    InvalidAccountIndex,
    /// Account not writable when write attempted
    AccountNotWritable,
    /// Account not signer when signature required
    AccountNotSigner,
    /// Invalid variable index, optionally with additional info
    InvalidVariableIndex(Option<u32>),
    /// Enhanced parameter mismatch error with full context
    ParameterMismatch {
        function_name: Option<Str32>,
        expected_count: u32,
        actual_count: u32,
        parameter_types: Vec8<Str16>,
        suggested_call: Option<Str64>,
    },
    /// Enhanced stack operation error with context
    StackOperationError {
        operation: Str32,
        required_items: u32,
        available_items: u32,
        instruction_pointer: usize,
        stack_state: Vec8<Str32>,
    },
    /// ABI-aware parameter mismatch with minimal VM data for client-side enhancement
    AbiParameterMismatch {
        function_index: u32,
        expected_param_count: u32,
        actual_param_count: u32,
        failed_param_index: u32,
    },
    /// Instruction pointer out of bounds
    InvalidInstructionPointer,
    /// Function call stack overflow
    CallStackOverflow,
    /// Function call stack underflow (return without call)
    CallStackUnderflow,
    /// Data buffer overflow
    DataBufferOverflow,
    /// Invalid operation
    InvalidOperation,
    /// Parse error during compilation with detailed context
    ParseError {
        expected: Str32,
        found: Str32,
        position: usize,
    },
    /// Unexpected token during parsing
    UnexpectedToken,
    /// Unexpected end of input
    UnexpectedEndOfInput,
    /// Invalid function index in function call
    InvalidFunctionIndex,
    /// Too many local variables allocated
    LocalsOverflow,
    /// Invalid account data operation
    InvalidAccountData,
    /// Invalid account reference
    InvalidAccount,
    /// Memory allocation or access error
    MemoryError,
    /// Specific ownership errors with context
    AccountOwnershipError {
        account_type: AccountType,
        account: Str64,
        expected_owner: Str64,
        actual_owner: Str64,
    },
    /// Cross-program invocation failed
    InvokeError {
        message: Str64,
    },
    /// External account lamport spend without signature
    ExternalAccountLamportSpend,
    /// Script not authorized to access this state account
    #[cfg(feature = "debug-logs")]
    ScriptNotAuthorized {
        account: Str64,
        current_script_address: Str64,
        authorized_script_address: Str64,
    },
    /// Minimal variant for production builds
    #[cfg(not(feature = "debug-logs"))]
    ScriptNotAuthorized {
        account_idx: u8,
        current_script_address: [u8; 32],
        authorized_script_address: [u8; 32],
    },
    /// Undefined account field access
    UndefinedAccountField,
    InvalidSeedArray(Str64),
    ImmutableField,
    /// Function visibility violation - attempt to call private function externally
    FunctionVisibilityViolation {
        function_index: u32,
        message: Str64,
    },
    UndefinedField,
    UndefinedIdentifier,
    /// Undefined identifier with additional source context and optional nearest match.
    UndefinedIdentifierWithContext {
        identifier: Str64,
        did_you_mean: Option<Str64>,
    },
    /// Duplicate imported symbol within a single namespace.
    DuplicateImport {
        symbol: Str64,
        namespace: Str16,
        import_ordinal: u32,
    },
    /// Invalid parameter count (legacy - specific version below)
    InvalidParameterCount,
    IndexOutOfBounds,
    OutOfMemory,
    ProtocolError,
    /// Too many seeds provided for PDA derivation
    TooManySeeds,
    /// Five bytecode account not authorized by import verification
    UnauthorizedBytecodeInvocation,
    /// Failed to derive PDA from provided seeds
    PdaDerivationFailed,

    /// Security rule violation detected during compilation
    SecurityViolation,
    /// Account not found or invalid account index
    AccountNotFound,
    /// Account data is empty when data expected
    AccountDataEmpty,
    /// Runtime integration with Solana required for this operation
    RuntimeIntegrationRequired,
    /// Invalid parameter provided to operation
    InvalidParameter,
    /// Invalid opcode encountered
    InvalidOpcode,
    /// Execution terminated by syscall (abort/panic)
    ExecutionTerminated,
    /// Account is uninitialized (zero lamports and data)
    UninitializedAccount,
}

/// Helper enum for AccountOwnershipError to consolidate variants
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountType {
    Script,
    VMState,
    User,
}

impl VMError {
    /// Convert VM error to ProgramError for Solana compatibility
    pub fn to_program_error(self) -> ProgramError {
        match self {
            VMError::StackError => ProgramError::InvalidInstructionData,
            VMError::InvalidInstruction => ProgramError::InvalidInstructionData,
            VMError::InvalidScript => ProgramError::InvalidInstructionData,
            VMError::InvalidScriptSize => ProgramError::InvalidInstructionData,
            VMError::MemoryViolation => ProgramError::Custom(9001),
            VMError::TypeMismatch => ProgramError::InvalidInstructionData,
            VMError::DivisionByZero => ProgramError::InvalidInstructionData,
            VMError::AccountError => ProgramError::Custom(9002),
            VMError::ConstraintViolation => ProgramError::Custom(9003),
            VMError::Halted => ProgramError::InvalidInstructionData,
            VMError::InvalidAccountIndex => ProgramError::NotEnoughAccountKeys,
            VMError::AccountNotWritable => ProgramError::Custom(9004),
            VMError::AccountNotSigner => ProgramError::MissingRequiredSignature,
            VMError::InvalidVariableIndex(_) => ProgramError::InvalidInstructionData,
            VMError::ParameterMismatch { .. } => ProgramError::Custom(9014),
            VMError::StackOperationError { .. } => ProgramError::InvalidInstructionData,
            VMError::AbiParameterMismatch { .. } => ProgramError::Custom(9015),
            VMError::InvalidInstructionPointer => ProgramError::InvalidInstructionData,
            VMError::CallStackOverflow => ProgramError::InvalidInstructionData,
            VMError::CallStackUnderflow => ProgramError::InvalidInstructionData,
            VMError::DataBufferOverflow => ProgramError::Custom(9005),
            VMError::InvalidOperation => ProgramError::InvalidInstructionData,
            VMError::ParseError { .. } => ProgramError::InvalidInstructionData,
            VMError::UnexpectedToken => ProgramError::InvalidInstructionData,
            VMError::UnexpectedEndOfInput => ProgramError::InvalidInstructionData,
            VMError::InvalidFunctionIndex => ProgramError::InvalidInstructionData,
            VMError::FunctionVisibilityViolation { .. } => ProgramError::InvalidInstructionData,
            VMError::LocalsOverflow => ProgramError::InvalidInstructionData,
            VMError::InvalidAccountData => ProgramError::Custom(9006),
            VMError::InvalidAccount => ProgramError::Custom(9007),
            VMError::MemoryError => ProgramError::Custom(9008),
            VMError::AccountOwnershipError { .. } => ProgramError::Custom(1100), // Can differentiate with custom codes if needed
            VMError::InvokeError { .. } => ProgramError::Custom(1103),
            VMError::ExternalAccountLamportSpend => ProgramError::Custom(1104),
            VMError::ScriptNotAuthorized { .. } => ProgramError::Custom(1105),
            VMError::UndefinedAccountField => ProgramError::Custom(9009),
            VMError::InvalidSeedArray(_) => ProgramError::InvalidArgument,
            VMError::ImmutableField => ProgramError::Custom(9010),
            VMError::UndefinedField => ProgramError::Custom(9011),
            VMError::UndefinedIdentifier | VMError::UndefinedIdentifierWithContext { .. } => {
                ProgramError::InvalidInstructionData
            }
            VMError::DuplicateImport { .. } => ProgramError::InvalidInstructionData,
            VMError::InvalidParameterCount => ProgramError::Custom(9014),
            VMError::IndexOutOfBounds => ProgramError::InvalidInstructionData,
            VMError::OutOfMemory => ProgramError::Custom(9012),
            VMError::ProtocolError => ProgramError::InvalidInstructionData,
            VMError::TooManySeeds => ProgramError::InvalidArgument,
            VMError::UnauthorizedBytecodeInvocation => ProgramError::Custom(1110),
            VMError::PdaDerivationFailed => ProgramError::InvalidSeeds,
            VMError::AccountNotFound => ProgramError::InvalidArgument,
            VMError::AccountDataEmpty => ProgramError::Custom(9013),
            VMError::RuntimeIntegrationRequired => ProgramError::Custom(1106),
            VMError::InvalidParameter => ProgramError::InvalidArgument,
            VMError::InvalidOpcode => ProgramError::InvalidInstructionData,
            VMError::ExecutionTerminated => ProgramError::Custom(1108),
            VMError::SecurityViolation => ProgramError::Custom(1109),
            VMError::NumericOverflow => ProgramError::ArithmeticOverflow,
            VMError::ArithmeticOverflow => ProgramError::ArithmeticOverflow,
            VMError::UninitializedAccount => ProgramError::UninitializedAccount,
        }
    }
}

impl From<VMError> for VMErrorCode {
    fn from(error: VMError) -> Self {
        match error {
            VMError::StackError => VMErrorCode::StackError,
            VMError::InvalidInstruction => VMErrorCode::InvalidInstruction,
            VMError::InvalidScript => VMErrorCode::InvalidScript,
            VMError::InvalidScriptSize => VMErrorCode::InvalidScriptSize,
            VMError::MemoryViolation => VMErrorCode::MemoryViolation,
            VMError::TypeMismatch => VMErrorCode::TypeMismatch,
            VMError::DivisionByZero => VMErrorCode::DivisionByZero,
            VMError::NumericOverflow => VMErrorCode::NumericOverflow,
            VMError::ArithmeticOverflow => VMErrorCode::ArithmeticOverflow,
            VMError::AccountError => VMErrorCode::AccountError,
            VMError::ConstraintViolation => VMErrorCode::ConstraintViolation,
            VMError::Halted => VMErrorCode::Halted,
            VMError::InvalidAccountIndex => VMErrorCode::InvalidAccountIndex,
            VMError::AccountNotWritable => VMErrorCode::AccountNotWritable,
            VMError::AccountNotSigner => VMErrorCode::AccountNotSigner,
            VMError::InvalidVariableIndex(_) => VMErrorCode::InvalidVariableIndex,
            VMError::ParameterMismatch { .. } => VMErrorCode::ParameterMismatch,
            VMError::StackOperationError { .. } => VMErrorCode::StackOperationError,
            VMError::AbiParameterMismatch { .. } => VMErrorCode::AbiParameterMismatch,
            VMError::InvalidInstructionPointer => VMErrorCode::InvalidInstructionPointer,
            VMError::CallStackOverflow => VMErrorCode::CallStackOverflow,
            VMError::CallStackUnderflow => VMErrorCode::CallStackUnderflow,
            VMError::DataBufferOverflow => VMErrorCode::DataBufferOverflow,
            VMError::InvalidOperation => VMErrorCode::InvalidOperation,
            VMError::ParseError { .. } => VMErrorCode::ParseError,
            VMError::UnexpectedToken => VMErrorCode::UnexpectedToken,
            VMError::UnexpectedEndOfInput => VMErrorCode::UnexpectedEndOfInput,
            VMError::InvalidFunctionIndex => VMErrorCode::InvalidFunctionIndex,
            VMError::LocalsOverflow => VMErrorCode::LocalsOverflow,
            VMError::InvalidAccountData => VMErrorCode::InvalidAccountData,
            VMError::InvalidAccount => VMErrorCode::InvalidAccount,
            VMError::MemoryError => VMErrorCode::MemoryError,
            VMError::AccountOwnershipError { .. } => VMErrorCode::AccountOwnershipError,
            VMError::InvokeError { .. } => VMErrorCode::InvokeError,
            VMError::ExternalAccountLamportSpend => VMErrorCode::ExternalAccountLamportSpend,
            VMError::ScriptNotAuthorized { .. } => VMErrorCode::ScriptNotAuthorized,
            VMError::UndefinedAccountField => VMErrorCode::UndefinedAccountField,
            VMError::InvalidSeedArray(_) => VMErrorCode::InvalidSeedArray,
            VMError::ImmutableField => VMErrorCode::ImmutableField,
            VMError::FunctionVisibilityViolation { .. } => VMErrorCode::FunctionVisibilityViolation,
            VMError::UndefinedField => VMErrorCode::UndefinedField,
            VMError::UndefinedIdentifier | VMError::UndefinedIdentifierWithContext { .. } => {
                VMErrorCode::UndefinedIdentifier
            }
            VMError::DuplicateImport { .. } => VMErrorCode::InvalidOperation,
            VMError::InvalidParameterCount => VMErrorCode::InvalidParameterCount,
            VMError::IndexOutOfBounds => VMErrorCode::IndexOutOfBounds,
            VMError::OutOfMemory => VMErrorCode::OutOfMemory,
            VMError::ProtocolError => VMErrorCode::ProtocolError,
            VMError::TooManySeeds => VMErrorCode::TooManySeeds,
            VMError::UnauthorizedBytecodeInvocation => VMErrorCode::UnauthorizedBytecodeInvocation,
            VMError::PdaDerivationFailed => VMErrorCode::PdaDerivationFailed,
            VMError::SecurityViolation => VMErrorCode::SecurityViolation,
            VMError::AccountNotFound => VMErrorCode::AccountNotFound,
            VMError::AccountDataEmpty => VMErrorCode::AccountDataEmpty,
            VMError::RuntimeIntegrationRequired => VMErrorCode::RuntimeIntegrationRequired,
            VMError::InvalidParameter => VMErrorCode::InvalidParameter,
            VMError::InvalidOpcode => VMErrorCode::InvalidOpcode,
            VMError::ExecutionTerminated => VMErrorCode::ExecutionTerminated,
            VMError::UninitializedAccount => VMErrorCode::UninitializedAccount,
        }
    }
}

impl From<VMError> for ProgramError {
    fn from(error: VMError) -> Self {
        error.to_program_error()
    }
}

impl From<VMErrorCode> for VMError {
    fn from(code: VMErrorCode) -> Self {
        match code {
            VMErrorCode::StackError => VMError::StackError,
            VMErrorCode::StackOverflow => VMError::StackError,
            VMErrorCode::StackUnderflow => VMError::StackError,
            VMErrorCode::InvalidInstruction => VMError::InvalidInstruction,
            VMErrorCode::InvalidScript => VMError::InvalidScript,
            VMErrorCode::InvalidScriptSize => VMError::InvalidScriptSize,
            VMErrorCode::MemoryViolation => VMError::MemoryViolation,
            VMErrorCode::TypeMismatch => VMError::TypeMismatch,
            VMErrorCode::DivisionByZero => VMError::DivisionByZero,
            VMErrorCode::NumericOverflow => VMError::NumericOverflow,
            VMErrorCode::ArithmeticOverflow => VMError::ArithmeticOverflow,
            VMErrorCode::AccountError => VMError::AccountError,
            VMErrorCode::ConstraintViolation => VMError::ConstraintViolation,
            VMErrorCode::Halted => VMError::Halted,
            VMErrorCode::InvalidAccountIndex => VMError::InvalidAccountIndex,
            VMErrorCode::AccountNotWritable => VMError::AccountNotWritable,
            VMErrorCode::AccountNotSigner => VMError::AccountNotSigner,
            VMErrorCode::InvalidVariableIndex => VMError::InvalidVariableIndex(None),
            VMErrorCode::ParameterMismatch => VMError::InvalidParameterCount,
            VMErrorCode::StackOperationError => VMError::StackError,
            VMErrorCode::AbiParameterMismatch => VMError::InvalidParameterCount,
            VMErrorCode::InvalidInstructionPointer => VMError::InvalidInstructionPointer,
            VMErrorCode::CallStackOverflow => VMError::CallStackOverflow,
            VMErrorCode::CallStackUnderflow => VMError::CallStackUnderflow,
            VMErrorCode::DataBufferOverflow => VMError::DataBufferOverflow,
            VMErrorCode::InvalidOperation => VMError::InvalidOperation,
            VMErrorCode::ParseError => VMError::UnexpectedToken,
            VMErrorCode::UnexpectedToken => VMError::UnexpectedToken,
            VMErrorCode::UnexpectedEndOfInput => VMError::UnexpectedEndOfInput,
            VMErrorCode::InvalidFunctionIndex => VMError::InvalidFunctionIndex,
            VMErrorCode::LocalsOverflow => VMError::LocalsOverflow,
            VMErrorCode::InvalidAccountData => VMError::InvalidAccountData,
            VMErrorCode::InvalidAccount => VMError::InvalidAccount,
            VMErrorCode::MemoryError => VMError::MemoryError,
            VMErrorCode::AccountOwnershipError => VMError::AccountError,
            VMErrorCode::InvokeError => VMError::InvalidOperation,
            VMErrorCode::ExternalAccountLamportSpend => VMError::ExternalAccountLamportSpend,
            VMErrorCode::ScriptNotAuthorized => VMError::InvalidOperation,
            VMErrorCode::UndefinedAccountField => VMError::UndefinedAccountField,
            VMErrorCode::InvalidSeedArray => VMError::InvalidOperation,
            VMErrorCode::ImmutableField => VMError::ImmutableField,
            VMErrorCode::FunctionVisibilityViolation => VMError::FunctionVisibilityViolation {
                function_index: 0,
                message: Default::default(),
            },
            VMErrorCode::UndefinedField => VMError::UndefinedField,
            VMErrorCode::UndefinedIdentifier => VMError::UndefinedIdentifier,
            VMErrorCode::InvalidParameterCount => VMError::InvalidParameterCount,
            VMErrorCode::IndexOutOfBounds => VMError::IndexOutOfBounds,
            VMErrorCode::OutOfMemory => VMError::OutOfMemory,
            VMErrorCode::ProtocolError => VMError::ProtocolError,
            VMErrorCode::TooManySeeds => VMError::TooManySeeds,
            VMErrorCode::UnauthorizedBytecodeInvocation => VMError::UnauthorizedBytecodeInvocation,
            VMErrorCode::PdaDerivationFailed => VMError::PdaDerivationFailed,
            VMErrorCode::SecurityViolation => VMError::SecurityViolation,
            VMErrorCode::AccountNotFound => VMError::AccountNotFound,
            VMErrorCode::AccountDataEmpty => VMError::AccountDataEmpty,
            VMErrorCode::RuntimeIntegrationRequired => VMError::RuntimeIntegrationRequired,
            VMErrorCode::InvalidParameter => VMError::InvalidParameter,
            VMErrorCode::InvalidOpcode => VMError::InvalidOpcode,
            VMErrorCode::ExecutionTerminated => VMError::ExecutionTerminated,
            VMErrorCode::UninitializedAccount => VMError::UninitializedAccount,
        }
    }
}

impl From<five_protocol::ProtocolError> for VMError {
    fn from(error: five_protocol::ProtocolError) -> Self {
        match error {
            five_protocol::ProtocolError::BufferTooSmall => VMError::MemoryError,
            five_protocol::ProtocolError::InvalidInstruction => VMError::InvalidInstruction,
            five_protocol::ProtocolError::TypeMismatch => VMError::TypeMismatch,
            five_protocol::ProtocolError::InvalidAccountData => VMError::InvalidAccountData,
        }
    }
}

impl std::fmt::Display for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut msg = |s: &str| write!(f, "{}", s);
        match self {
            VMError::StackError => msg("Stack overflow or underflow"),
            VMError::InvalidInstruction => msg("Invalid instruction or bytecode"),
            VMError::InvalidScript => msg("Invalid script format or magic bytes"),
            VMError::MemoryViolation => msg("Memory access violation"),
            VMError::TypeMismatch => msg("Type mismatch in operation"),
            VMError::DivisionByZero => msg("Division by zero"),
            VMError::AccountError => msg("Account access error"),
            VMError::ConstraintViolation => msg("Constraint validation failed"),
            VMError::Halted => msg("Script execution halted"),
            VMError::InvalidAccountIndex => msg("Invalid account index"),
            VMError::AccountNotWritable => msg("Account not writable when write attempted"),
            VMError::AccountNotSigner => msg("Account not signer when signature required"),
            VMError::InvalidVariableIndex(index_opt) => {
                if let Some(index) = index_opt {
                    write!(f, "Invalid variable index: {}", index)
                } else {
                    msg("Invalid variable index")
                }
            }
            VMError::ParameterMismatch {
                function_name: _,
                expected_count,
                actual_count,
                parameter_types,
                suggested_call,
            } => {
                write!(f, "❌ Function Parameter Mismatch\n\n")?;
                write!(
                    f,
                    "Expected {} parameters but received {}\n\n",
                    expected_count, actual_count
                )?;

                if !parameter_types.is_empty() {
                    write!(f, "Expected parameter types: [")?;
                    for (i, param_type) in parameter_types.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", param_type)?;
                    }
                    write!(f, "]\n\n")?;
                }

                if let Some(suggestion) = suggested_call {
                    writeln!(
                        f,
                        "💡 Fix this by providing the correct number of parameters:"
                    )?;
                    writeln!(f, "   • Example: {}", suggestion)?;
                }

                write!(f, "📍 Context: Function called with wrong parameter count")
            }
            VMError::StackOperationError {
                operation,
                required_items,
                available_items,
                instruction_pointer,
                stack_state,
            } => {
                write!(f, "❌ Stack Operation Error\n\n")?;
                write!(
                    f,
                    "Operation '{}' requires {} stack items but only {} available\n\n",
                    operation, required_items, available_items
                )?;

                writeln!(f, "📍 Context:")?;
                writeln!(
                    f,
                    "   • Instruction pointer: 0x{:04X}",
                    instruction_pointer
                )?;
                writeln!(f, "   • Operation: {}", operation)?;
                write!(f, "   • Stack state: [")?;
                for (i, item) in stack_state.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]\n\n")?;

                write!(f, "💡 Help: Check that all variables are properly initialized before this operation")
            }
            VMError::AbiParameterMismatch {
                function_index,
                expected_param_count,
                actual_param_count,
                failed_param_index,
            } => {
                write!(
                    f,
                    "PARAMETER_MISMATCH:function_index={},expected={},actual={},failed_at={}",
                    function_index, expected_param_count, actual_param_count, failed_param_index
                )
            }
            VMError::InvalidInstructionPointer => msg("Instruction pointer out of bounds"),
            VMError::CallStackOverflow => msg("Function call stack overflow"),
            VMError::CallStackUnderflow => {
                msg("Function call stack underflow (return without call)")
            }
            VMError::DataBufferOverflow => msg("Data buffer overflow"),
            VMError::InvalidOperation => msg("Invalid operation"),
            VMError::ParseError {
                expected,
                found,
                position,
            } => {
                write!(
                    f,
                    "Parse error at position {}: expected {}, found {}",
                    position, expected, found
                )
            }
            VMError::UnexpectedToken => msg("Unexpected token during parsing"),
            VMError::UnexpectedEndOfInput => msg("Unexpected end of input"),
            VMError::InvalidFunctionIndex => msg("Invalid function index in function call"),
            VMError::FunctionVisibilityViolation {
                function_index: _,
                message,
            } => msg(message.as_str()),
            VMError::LocalsOverflow => msg("Too many local variables allocated"),
            VMError::InvalidAccountData => msg("Invalid account data operation"),
            VMError::InvalidAccount => msg("Invalid account reference"),
            VMError::MemoryError => msg("Memory allocation or access error"),
            VMError::AccountOwnershipError {
                account_type,
                account,
                expected_owner,
                actual_owner,
            } => {
                let type_str = match account_type {
                    AccountType::Script => "Script",
                    AccountType::VMState => "VM state",
                    AccountType::User => "User",
                };
                write!(
                    f,
                    "{} account {} ownership error: expected owner {}, actual owner {}",
                    type_str, account, expected_owner, actual_owner
                )
            }
            VMError::InvokeError { message } => {
                write!(f, "Cross-program invocation failed: {}", message)
            }
            VMError::ExternalAccountLamportSpend => {
                msg("External account lamport spend without signature")
            }
            #[cfg(feature = "debug-logs")]
            VMError::ScriptNotAuthorized {
                account,
                current_script_address,
                authorized_script_address,
            } => {
                write!(f, "Script not authorized to access account {}: current script address {}, authorized address {}",
                       account, current_script_address, authorized_script_address)
            }
            #[cfg(not(feature = "debug-logs"))]
            VMError::ScriptNotAuthorized { account_idx, .. } => {
                write!(
                    f,
                    "Script not authorized to access account index {}",
                    account_idx
                )
            }
            VMError::UndefinedAccountField => msg("Undefined account field access"),
            VMError::InvalidSeedArray(msg_content) => {
                write!(f, "Invalid seed array: {}", msg_content)
            }
            VMError::ImmutableField => msg("Attempt to modify an immutable field"),
            VMError::UndefinedField => msg("Attempt to access an undefined field"),
            VMError::UndefinedIdentifier => msg("Attempt to access an undefined identifier"),
            VMError::UndefinedIdentifierWithContext {
                identifier,
                did_you_mean,
            } => {
                if let Some(candidate) = did_you_mean {
                    write!(
                        f,
                        "Cannot find value '{}' in this scope (did you mean '{}'?)",
                        identifier, candidate
                    )
                } else {
                    write!(f, "Cannot find value '{}' in this scope", identifier)
                }
            }
            VMError::DuplicateImport {
                symbol,
                namespace,
                ..
            } => {
                write!(
                    f,
                    "Duplicate imported {} symbol '{}' in the same namespace",
                    namespace, symbol
                )
            }
            VMError::InvalidParameterCount => msg("Invalid parameter count"),
            VMError::IndexOutOfBounds => msg("Index out of bounds"),
            VMError::OutOfMemory => msg("Out of memory"),
            VMError::ProtocolError => msg("Protocol error"),
            VMError::TooManySeeds => msg("Too many seeds provided for PDA derivation"),
            VMError::UnauthorizedBytecodeInvocation => {
                msg("Five bytecode account not authorized by import verification - the target account was not declared in the import metadata")
            }
            VMError::PdaDerivationFailed => {
                msg("Failed to derive PDA from provided seeds - check that seeds and program ID are correct")
            }
            VMError::AccountNotFound => msg("Account not found or invalid account index"),
            VMError::AccountDataEmpty => msg("Account data is empty when data was expected"),
            VMError::RuntimeIntegrationRequired => {
                msg("Runtime integration with Solana required for this operation")
            }
            VMError::InvalidParameter => msg("Invalid parameter provided to operation"),
            VMError::InvalidOpcode => msg("Invalid opcode encountered"),
            VMError::ExecutionTerminated => msg("Execution terminated by syscall"),
            VMError::SecurityViolation => msg("Security rule violation detected"),
            VMError::NumericOverflow => msg("Numeric overflow when narrowing u128 to u64"),
            VMError::ArithmeticOverflow => msg(
                "Arithmetic overflow in checked operation (ADD_CHECKED/SUB_CHECKED/MUL_CHECKED)",
            ),
            VMError::UninitializedAccount => {
                msg("Account is uninitialized (zero lamports and data)")
            }
            VMError::InvalidScriptSize => msg("Script exceeds maximum allowed size"),
        }
    }
}

impl std::error::Error for VMError {}

/// Enhanced error context builders for better diagnostics
impl VMError {
    /// Create a parameter mismatch error with full context
    pub fn parameter_mismatch(
        function_name: Option<Str32>,
        expected_count: u32,
        actual_count: u32,
        parameter_types: Vec8<Str16>,
        suggested_call: Option<Str64>,
    ) -> Self {
        Self::ParameterMismatch {
            function_name,
            expected_count,
            actual_count,
            parameter_types,
            suggested_call,
        }
    }

    /// Create a stack operation error with context
    pub fn stack_operation_error(
        operation: Str32,
        required_items: u32,
        available_items: u32,
        instruction_pointer: usize,
        stack_state: Vec8<Str32>,
    ) -> Self {
        Self::StackOperationError {
            operation,
            required_items,
            available_items,
            instruction_pointer,
            stack_state,
        }
    }

    /// Create a parameter mismatch error for function calls
    pub fn function_parameter_mismatch(
        function_name: &str,
        expected_params: &[&str],
        actual_count: u32,
    ) -> Self {
        use core::fmt::Write;
        let mut fname = Str32::new();
        let _ = fname.push_str(function_name);

        let mut types = Vec8::<Str16>::new();
        for p in expected_params.iter().take(8) {
            let mut s = Str16::new();
            let _ = s.push_str(p);
            let _ = types.push(s);
        }

        let suggested_call = if expected_params.len() == 2 {
            let mut s = Str64::new();
            let _ = write!(s, "{}(100, 50)", function_name);
            Some(s)
        } else if expected_params.len() == 1 {
            let mut s = Str64::new();
            let _ = write!(s, "{}(42)", function_name);
            Some(s)
        } else {
            None
        };

        Self::parameter_mismatch(
            Some(fname),
            expected_params.len() as u32,
            actual_count,
            types,
            suggested_call,
        )
    }

    /// Create ABI-aware parameter mismatch error (minimal data for client-side enhancement)
    pub fn abi_parameter_mismatch(
        function_index: u32,
        expected_param_count: u32,
        actual_param_count: u32,
        failed_param_index: u32,
    ) -> Self {
        Self::AbiParameterMismatch {
            function_index,
            expected_param_count,
            actual_param_count,
            failed_param_index,
        }
    }

    /// Create an enhanced stack underflow error
    pub fn stack_underflow_with_context(
        operation: &str,
        required: u32,
        available: u32,
        ip: usize,
        stack_contents: &[five_protocol::ValueRef],
    ) -> Self {
        use core::fmt::Write;
        let mut stack_state: Vec8<Str32> = Vec8::new();
        for v in stack_contents.iter().take(8) {
            let mut s = Str32::new();
            match v {
                five_protocol::ValueRef::U64(n) => {
                    let _ = write!(s, "u64({})", n);
                }
                five_protocol::ValueRef::U128(n) => {
                    let _ = write!(s, "u128({})", n);
                }
                five_protocol::ValueRef::Bool(b) => {
                    let _ = write!(s, "bool({})", b);
                }
                five_protocol::ValueRef::U8(n) => {
                    let _ = write!(s, "u8({})", n);
                }
                five_protocol::ValueRef::Empty => {
                    let _ = s.push_str("empty");
                }
                five_protocol::ValueRef::PubkeyRef(_) => {
                    let _ = s.push_str("pubkey_ref");
                }
                five_protocol::ValueRef::AccountRef(idx, offset) => {
                    let _ = write!(s, "account_ref({}, {})", idx, offset);
                }
                five_protocol::ValueRef::StringRef(_) => {
                    let _ = s.push_str("string_ref");
                }
                five_protocol::ValueRef::ArrayRef(_) => {
                    let _ = s.push_str("array_ref");
                }
                five_protocol::ValueRef::HeapString(_) => {
                    let _ = s.push_str("heap_string");
                }
                five_protocol::ValueRef::HeapArray(_) => {
                    let _ = s.push_str("heap_array");
                }
                five_protocol::ValueRef::I64(n) => {
                    let _ = write!(s, "i64({})", n);
                }
                five_protocol::ValueRef::ResultRef(status, value) => {
                    let _ = write!(s, "result({}, {})", status, value);
                }
                five_protocol::ValueRef::InputRef(offset) => {
                    let _ = write!(s, "input_ref({})", offset);
                }
                five_protocol::ValueRef::TempRef(offset, size) => {
                    let _ = write!(s, "temp_ref({}, {})", offset, size);
                }
                five_protocol::ValueRef::TupleRef(offset, size) => {
                    let _ = write!(s, "tuple_ref({}, {})", offset, size);
                }
                five_protocol::ValueRef::OptionalRef(offset, size) => {
                    let _ = write!(s, "optional_ref({}, {})", offset, size);
                }
            }
            let _ = stack_state.push(s);
        }

        let mut op = Str32::new();
        let _ = op.push_str(operation);

        Self::stack_operation_error(op, required, available, ip, stack_state)
    }

    /// Create an undefined identifier error with optional nearest-match context.
    pub fn undefined_identifier(identifier: &str, did_you_mean: Option<&str>) -> Self {
        let mut identifier_buf = Str64::new();
        let _ = identifier_buf.push_str(identifier);

        let did_you_mean_buf = did_you_mean.map(|candidate| {
            let mut candidate_buf = Str64::new();
            let _ = candidate_buf.push_str(candidate);
            candidate_buf
        });

        Self::UndefinedIdentifierWithContext {
            identifier: identifier_buf,
            did_you_mean: did_you_mean_buf,
        }
    }

    /// Create a duplicate-import error with namespace context.
    pub fn duplicate_import(symbol: &str, namespace: &str, import_ordinal: u32) -> Self {
        let mut symbol_buf = Str64::new();
        let _ = symbol_buf.push_str(symbol);

        let mut namespace_buf = Str16::new();
        let _ = namespace_buf.push_str(namespace);

        Self::DuplicateImport {
            symbol: symbol_buf,
            namespace: namespace_buf,
            import_ordinal,
        }
    }
}
