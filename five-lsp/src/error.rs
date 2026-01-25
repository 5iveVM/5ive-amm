//! LSP error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LspError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Compiler error: {0}")]
    CompilerError(String),

    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("LSP error: {0}")]
    LspError(String),
}

#[cfg(feature = "native")]
impl From<LspError> for tower_lsp::jsonrpc::Error {
    fn from(err: LspError) -> Self {
        tower_lsp::jsonrpc::Error {
            code: tower_lsp::jsonrpc::ErrorCode::InternalError,
            message: err.to_string().into(),
            data: None,
        }
    }
}
