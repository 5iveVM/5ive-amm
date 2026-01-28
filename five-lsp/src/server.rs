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

                // Phase 2 capabilities (goto-definition and find-references enabled)
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),

                // Phase 3 capabilities (rename enabled)
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: features::semantic::SEMANTIC_TOKEN_TYPES.to_vec(),
                                token_modifiers: features::semantic::SEMANTIC_TOKEN_MODIFIERS.to_vec(),
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: None,
                            ..Default::default()
                        }
                    )
                ),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                rename_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),

                // Phase 4 capabilities (now enabled)
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: None,
                    ..Default::default()
                }),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                inlay_hint_provider: Some(OneOf::Left(InlayHintServerCapabilities::Options(
                    InlayHintOptions {
                        resolve_provider: Some(false),
                        ..Default::default()
                    }
                ))),
                document_formatting_provider: Some(OneOf::Left(true)),

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

    // Phase 1-2 features
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri.clone();
        let position = params.text_document_position_params.position;

        let documents = self.documents.read().await;
        let doc = match documents.get(&uri) {
            Some(d) => d.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // Use write lock to ensure AST is compiled and cached
        let mut bridge = self.bridge.write().await;
        let hover_info = features::hover::get_hover(
            &bridge,
            &doc.content,
            position,
            &uri,
        );

        Ok(hover_info)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let position = params.text_document_position.position;

        let documents = self.documents.read().await;
        let doc = match documents.get(&uri) {
            Some(d) => d.clone(),
            None => return Ok(None),
        };
        drop(documents);

        let mut bridge = self.bridge.write().await;
        let completion_list = features::completion::get_completions(
            &bridge,
            &doc.content,
            position.line as usize,
            position.character as usize,
            &uri,
        );

        Ok(if completion_list.items.is_empty() {
            None
        } else {
            Some(CompletionResponse::List(completion_list))
        })
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri.clone();
        let position = params.text_document_position_params.position;

        // Get source code
        let documents = self.documents.read().await;
        let doc = match documents.get(&uri) {
            Some(d) => d.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // Use bridge to find definition semantically
        let mut bridge = self.bridge.write().await;
        let location = features::goto_definition::get_definition(
            &mut bridge,
            &uri,
            &doc.content,
            position.line,
            position.character,
        );

        Ok(location.map(GotoDefinitionResponse::Scalar))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let position = params.text_document_position.position;

        // Get source code
        let documents = self.documents.read().await;
        let doc = match documents.get(&uri) {
            Some(d) => d.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // Use bridge to find references semantically
        let mut bridge = self.bridge.write().await;
        let references = features::find_references::find_references(
            &mut bridge,
            &uri,
            &doc.content,
            position.line as usize,
            position.character as usize,
        );

        Ok(if references.is_empty() { None } else { Some(references) })
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri.clone();
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        // Get source code
        let documents = self.documents.read().await;
        let doc = match documents.get(&uri) {
            Some(d) => d.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // Use bridge to find and rename all references semantically
        let mut bridge = self.bridge.write().await;
        let workspace_edit = features::rename::rename(
            &mut bridge,
            &doc.content,
            position.line as usize,
            position.character as usize,
            &new_name,
            &uri,
        );

        Ok(workspace_edit)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.clone();

        let documents = self.documents.read().await;
        let doc = match documents.get(&uri) {
            Some(d) => d.clone(),
            None => return Ok(None),
        };
        drop(documents);

        let bridge = self.bridge.read().await;
        let semantic_tokens = features::semantic::get_semantic_tokens(
            &bridge,
            &doc.content,
            &uri,
        );

        // Convert SerializableSemanticToken to SemanticToken format (flat array)
        let mut data = Vec::new();
        let mut last_line = 0u32;
        let mut last_start = 0u32;

        for token in semantic_tokens {
            let line_delta = if token.line >= last_line {
                token.line - last_line
            } else {
                token.line - last_line
            };
            let start_delta = if token.line == last_line {
                token.start_character - last_start
            } else {
                token.start_character
            };

            data.push(line_delta);
            data.push(start_delta);
            data.push(token.length);
            data.push(token.token_type);
            data.push(token.token_modifiers);

            last_line = token.line;
            last_start = token.start_character;
        }

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data,
        })))
    }

    async fn signature_help(
        &self,
        params: SignatureHelpParams,
    ) -> Result<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri.clone();
        let position = params.text_document_position_params.position;

        let documents = self.documents.read().await;
        let doc = match documents.get(&uri) {
            Some(d) => d.clone(),
            None => return Ok(None),
        };
        drop(documents);

        let bridge = self.bridge.read().await;
        let signature = features::signature_help::get_signature_help(
            &bridge,
            &doc.content,
            position.line as usize,
            position.character as usize,
        );

        Ok(signature)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.clone();

        let documents = self.documents.read().await;
        let doc = match documents.get(&uri) {
            Some(d) => d.clone(),
            None => return Ok(None),
        };
        drop(documents);

        let bridge = self.bridge.read().await;
        let symbols = features::document_symbols::get_document_symbols(
            &bridge,
            &doc.content,
            &uri,
        );

        Ok(if symbols.is_empty() {
            None
        } else {
            Some(DocumentSymbolResponse::Nested(symbols))
        })
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let documents = self.documents.read().await;
        let mut all_symbols = Vec::new();

        for (uri, doc) in documents.iter() {
            let symbols = features::workspace_symbols::workspace_symbols(
                &doc.content,
                &params.query,
                uri,
            );
            all_symbols.extend(symbols);
        }

        Ok(if all_symbols.is_empty() {
            None
        } else {
            Some(all_symbols)
        })
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.clone();

        let documents = self.documents.read().await;
        let doc = match documents.get(&uri) {
            Some(d) => d.clone(),
            None => return Ok(None),
        };
        drop(documents);

        let edits = features::formatting::format_document(&doc.content);

        Ok(if edits.is_empty() {
            None
        } else {
            Some(edits)
        })
    }

    async fn inlay_hint(
        &self,
        params: InlayHintParams,
    ) -> Result<Option<Vec<InlayHint>>> {
        let uri = params.text_document.uri.clone();

        let documents = self.documents.read().await;
        let doc = match documents.get(&uri) {
            Some(d) => d.clone(),
            None => return Ok(None),
        };
        drop(documents);

        let hints = features::inlay_hints::get_inlay_hints(&doc.content, params.range.start.line as usize);

        Ok(if hints.is_empty() {
            None
        } else {
            Some(hints)
        })
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri.clone();

        let documents = self.documents.read().await;
        let doc = match documents.get(&uri) {
            Some(d) => d.clone(),
            None => return Ok(None),
        };
        drop(documents);

        // Get diagnostics in the range to provide context for code actions
        let mut actions = Vec::new();

        // For now, provide general code actions without relying on diagnostics
        // In a real implementation, we'd analyze the code in the range
        // and potentially fetch diagnostics to provide better suggestions

        Ok(if actions.is_empty() {
            None
        } else {
            Some(actions.into_iter().map(CodeActionOrCommand::CodeAction).collect())
        })
    }
}
