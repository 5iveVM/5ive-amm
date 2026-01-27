//! LSP feature implementations
//!
//! Phase 1: Diagnostics (errors and warnings)
//! Phase 2: Hover, Completion, Go-to-definition, Find References
//! Phase 3: Semantic tokens, Code actions, Rename, Document Symbols
//! Phase 4: Signature help, Workspace symbols, Inlay hints

pub mod diagnostics;
pub mod hover;
pub mod completion;
pub mod goto_definition;
pub mod find_references;
pub mod semantic;
pub mod code_actions;
pub mod document_symbols;
pub mod rename;
pub mod workspace_symbols;

pub use diagnostics::*;
pub use hover::*;
pub use completion::*;
pub use goto_definition::*;
pub use find_references::*;
pub use semantic::*;
pub use code_actions::*;
pub use document_symbols::*;
pub use rename::*;
pub use workspace_symbols::*;
