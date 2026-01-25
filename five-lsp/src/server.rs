//! Core LSP server implementation
//!
//! Implements the Language Server Protocol using tower-lsp,
//! handling document synchronization and feature requests.

use crate::bridge::CompilerBridge;
use crate::document::DocumentStore;
use crate::features;
use crate::workspace::Workspace;
use lsp_types::*;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::{Client, LanguageServer};

#[cfg(feature = "native")]
use tokio::sync::RwLock;
#[cfg(feature = "native")]
use tower_lsp::lsp_types::MessageType;

pub struct FiveLanguageServer {
    client: Client,
    documents: Arc<RwLock<DocumentStore>>,
    workspace: Arc<RwLock<Workspace>>,
    bridge: Arc<RwLock<CompilerBridge>>,
}

impl FiveLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(DocumentStore::new())),
            workspace: Arc::new(RwLock::new(Workspace::new())),
            bridge: Arc::new(RwLock::new(CompilerBridge::new())),
        }
    }

    async fn publish_diagnostics(&self, uri: Url, version: i32) -> Result<()> {
        let documents = self.documents.read().await;
        let doc = match documents.get(&uri) {
            Some(d) => d.clone(),
            None => return Ok(()),
        };

        let mut bridge = self.bridge.write().await;
        match features::get_diagnostics(&mut bridge, &uri, &doc.content) {
            Ok(diagnostics) => {
                self.client
                    .publish_diagnostics(uri, diagnostics, Some(version))
                    .await;
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to get diagnostics: {}", e);
                Ok(())
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for FiveLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Register workspace roots
        if let Some(root_uri) = params.root_uri {
            let mut workspace = self.workspace.write().await;
            workspace.add_root(root_uri);
        }

        for root in params.workspace_folders.unwrap_or_default() {
            let mut workspace = self.workspace.write().await;
            workspace.add_root(root.uri);
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // Phase 1: Diagnostics (document synchronization)
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),

                // Phase 2 capabilities (future - disabled for MVP)
                hover_provider: Some(HoverProviderCapability::Simple(false)),
                completion_provider: None,
                definition_provider: Some(OneOf::Left(false)),
                references_provider: Some(OneOf::Left(false)),

                // Phase 3 capabilities (future)
                semantic_tokens_provider: None,
                code_action_provider: None,
                rename_provider: Some(OneOf::Left(false)),
                document_symbol_provider: Some(OneOf::Left(false)),

                // Phase 4 capabilities (future)
                signature_help_provider: None,
                workspace_symbol_provider: Some(OneOf::Left(false)),
                inlay_hint_provider: None,

                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "Five LSP".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Five Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let language_id = params.text_document.language_id;
        let content = params.text_document.text;
        let version = params.text_document.version;

        // Store document
        let mut documents = self.documents.write().await;
        documents.open(uri.clone(), language_id.clone(), content.clone());

        // Register file in workspace
        let mut workspace = self.workspace.write().await;
        workspace.register_file(uri.clone());
        drop(workspace);
        drop(documents);

        // Publish diagnostics
        if let Err(e) = self.publish_diagnostics(uri, version).await {
            tracing::error!("Failed to publish diagnostics on open: {}", e);
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Update document with changes
        let mut documents = self.documents.write().await;
        for change in params.content_changes {
            match change {
                TextDocumentContentChangeEvent {
                    range: None,
                    range_length: _,
                    text,
                } => {
                    // Full document update
                    documents.update_content(&uri, text, version);
                }
                TextDocumentContentChangeEvent {
                    range: Some(range),
                    range_length: _,
                    text,
                } => {
                    // Incremental update
                    documents.apply_change(&uri, Some(range), text);
                }
            }
        }
        drop(documents);

        // Publish diagnostics
        if let Err(e) = self.publish_diagnostics(uri, version).await {
            tracing::error!("Failed to publish diagnostics on change: {}", e);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document
        let mut documents = self.documents.write().await;
        documents.close(&uri);
        drop(documents);

        // Unregister from workspace
        let mut workspace = self.workspace.write().await;
        workspace.unregister_file(&uri);

        // Clear diagnostics
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;

        // On save, clear caches to ensure fresh compilation
        let mut bridge = self.bridge.write().await;
        bridge.clear_caches();
        drop(bridge);

        // Re-publish diagnostics with fresh compilation
        if let Err(e) = self.publish_diagnostics(uri, 0).await {
            tracing::error!("Failed to publish diagnostics on save: {}", e);
        }
    }

    // Phase 2 features (disabled for MVP)
    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        Ok(None)
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(None)
    }

    async fn goto_definition(
        &self,
        _params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        Ok(None)
    }

    async fn references(&self, _params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        Ok(None)
    }
}
