//! Semantic Analysis Layer for Five LSP
//!
//! This module provides semantic understanding of Five DSL code by reusing
//! the compiler's AST and type checker infrastructure. It adds:
//! - Workspace-wide symbol indexing
//! - Position-based AST queries
//! - Cross-file definition/reference tracking
//! - Scope-aware symbol resolution
//!
//! The design philosophy is to reuse ~80% of the compiler's semantic infrastructure
//! rather than rebuilding it, adding only LSP-specific query layers.

pub mod index;
pub mod query;
pub mod scope_resolver;

pub use index::SemanticIndex;
pub use query::{ast_node_at_position, symbol_under_cursor};
pub use scope_resolver::ScopeResolver;
