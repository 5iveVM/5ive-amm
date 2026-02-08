//! Compiler-specific error types with context.

use five_vm_mito::error::VMErrorCode;
use crate::tokenizer::TokenSpan;
use std::fmt;

/// Boxed result type for compiler operations (8 bytes for Err variant).
/// Using `Box<CompilerError>` keeps Result size small.
/// of how much context is attached to errors.
pub type CompilerResult<T> = Result<T, Box<CompilerError>>;

/// Source location for error reporting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: Option<String>,
    pub line: u32,
    pub column: u32,
}

impl SourceLocation {
    pub fn new(line: u32, column: u32) -> Self {
        Self {
            file: None,
            line,
            column,
        }
    }

    pub fn with_file(file: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            file: Some(file.into()),
            line,
            column,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(file) = &self.file {
            write!(f, "{}:{}:{}", file, self.line, self.column)
        } else {
            write!(f, "line {}:{}", self.line, self.column)
        }
    }
}

/// Comprehensive error type for compiler operations.
/// 
/// This enum is designed to carry rich context for debugging and user-facing
/// error messages. It should always be boxed when used in Result types to
/// keep the Result size small.
#[derive(Debug, Clone)]
pub enum CompilerError {
    /// VM error code passthrough (for errors originating from VM operations)
    VM(VMErrorCode),

    /// Standard VMError passthrough (for backward compatibility)
    VMError(five_vm_mito::error::VMError),

    /// Parse error with source location and context
    Parse {
        message: String,
        location: Option<SourceLocation>,
        source_snippet: Option<String>,
        expected: Option<String>,
        found: Option<String>,
    },

    /// Type checking error
    Type {
        message: String,
        expected: Option<String>,
        found: Option<String>,
        location: Option<SourceLocation>,
    },

    /// Bytecode generation error
    CodeGen {
        message: String,
        context: Option<String>,
    },

    /// Semantic analysis error
    Semantic {
        message: String,
        location: Option<SourceLocation>,
    },

    /// Configuration error
    Config(String),

    /// IO error (file reading, etc.)
    Io(String),

    /// Generic error with message
    Other(String),
}

impl CompilerError {
    // =========== Convenience constructors ===========

    /// Create a parse error with full context
    pub fn parse(message: impl Into<String>) -> Box<Self> {
        Box::new(Self::Parse {
            message: message.into(),
            location: None,
            source_snippet: None,
            expected: None,
            found: None,
        })
    }

    /// Create a parse error with location
    pub fn parse_at(message: impl Into<String>, line: u32, column: u32) -> Box<Self> {
        Box::new(Self::Parse {
            message: message.into(),
            location: Some(SourceLocation::new(line, column)),
            source_snippet: None,
            expected: None,
            found: None,
        })
    }

    /// Create a type error
    pub fn type_error(message: impl Into<String>) -> Box<Self> {
        Box::new(Self::Type {
            message: message.into(),
            expected: None,
            found: None,
            location: None,
        })
    }

    /// Create a type mismatch error
    pub fn type_mismatch(
        expected: impl Into<String>,
        found: impl Into<String>,
        location: Option<SourceLocation>,
    ) -> Box<Self> {
        let expected_str = expected.into();
        let found_str = found.into();
        Box::new(Self::Type {
            message: format!("Type mismatch: expected {}, found {}", expected_str, found_str),
            expected: Some(expected_str),
            found: Some(found_str),
            location,
        })
    }

    /// Create a code generation error
    pub fn codegen(message: impl Into<String>) -> Box<Self> {
        Box::new(Self::CodeGen {
            message: message.into(),
            context: None,
        })
    }

    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Box<Self> {
        Box::new(Self::Config(message.into()))
    }

    /// Create an IO error
    pub fn io(message: impl Into<String>) -> Box<Self> {
        Box::new(Self::Io(message.into()))
    }

    /// Create a generic error
    pub fn other(message: impl Into<String>) -> Box<Self> {
        Box::new(Self::Other(message.into()))
    }

    /// Convert from VMError (for backward compatibility)
    pub fn from_vm_error(error: five_vm_mito::error::VMError) -> Box<Self> {
        Box::new(Self::VMError(error))
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VM(code) => write!(f, "{}", code),
            Self::VMError(e) => write!(f, "{}", e),
            Self::Parse {
                message,
                location,
                source_snippet,
                expected,
                found,
            } => {
                write!(f, "Parse error: {}", message)?;
                if let Some(loc) = location {
                    write!(f, " at {}", loc)?;
                }
                if let (Some(exp), Some(fnd)) = (expected, found) {
                    write!(f, "\n  expected: {}\n  found: {}", exp, fnd)?;
                }
                if let Some(snippet) = source_snippet {
                    write!(f, "\n  {}", snippet)?;
                }
                Ok(())
            }
            Self::Type {
                message,
                expected,
                found,
                location,
            } => {
                write!(f, "Type error: {}", message)?;
                if let Some(loc) = location {
                    write!(f, " at {}", loc)?;
                }
                if let (Some(exp), Some(fnd)) = (expected, found) {
                    write!(f, "\n  expected: {}\n  found: {}", exp, fnd)?;
                }
                Ok(())
            }
            Self::CodeGen { message, context } => {
                write!(f, "Code generation error: {}", message)?;
                if let Some(ctx) = context {
                    write!(f, "\n  context: {}", ctx)?;
                }
                Ok(())
            }
            Self::Semantic { message, location } => {
                write!(f, "Semantic error: {}", message)?;
                if let Some(loc) = location {
                    write!(f, " at {}", loc)?;
                }
                Ok(())
            }
            Self::Config(msg) => write!(f, "Configuration error: {}", msg),
            Self::Io(msg) => write!(f, "IO error: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CompilerError {}

// =========== Conversions ===========

impl From<VMErrorCode> for CompilerError {
    fn from(code: VMErrorCode) -> Self {
        Self::VM(code)
    }
}

impl From<VMErrorCode> for Box<CompilerError> {
    fn from(code: VMErrorCode) -> Self {
        Box::new(CompilerError::VM(code))
    }
}

impl From<five_vm_mito::error::VMError> for CompilerError {
    fn from(error: five_vm_mito::error::VMError) -> Self {
        Self::VMError(error)
    }
}

impl From<five_vm_mito::error::VMError> for Box<CompilerError> {
    fn from(error: five_vm_mito::error::VMError) -> Self {
        Box::new(CompilerError::VMError(error))
    }
}

impl From<std::io::Error> for Box<CompilerError> {
    fn from(error: std::io::Error) -> Self {
        Box::new(CompilerError::Io(error.to_string()))
    }
}

impl From<String> for Box<CompilerError> {
    fn from(message: String) -> Self {
        Box::new(CompilerError::Other(message))
    }
}

impl From<&str> for Box<CompilerError> {
    fn from(message: &str) -> Self {
        Box::new(CompilerError::Other(message.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_sizes() {
        // Box should be pointer-sized (8 bytes on 64-bit)
        assert_eq!(std::mem::size_of::<Box<CompilerError>>(), 8);

        // Result with boxed error should be small
        assert!(std::mem::size_of::<CompilerResult<()>>() <= 16);
    }

    #[test]
    fn test_error_display() {
        let err = CompilerError::parse("Unexpected token");
        assert!(err.to_string().contains("Unexpected token"));

        let err = CompilerError::type_mismatch("u64", "bool", None);
        assert!(err.to_string().contains("u64"));
        assert!(err.to_string().contains("bool"));
    }
}
