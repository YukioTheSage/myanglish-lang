use dashmap::DashMap;
use ropey::Rope;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use mlang::formatter;

mod analysis;
mod completion;
mod hover;
mod semantic_tokens;

use analysis::AnalysisResult;

pub struct MlangBackend {
    client: Client,
    documents: DashMap<Url, Rope>,
    analysis_cache: DashMap<Url, Arc<AnalysisResult>>,
}

impl MlangBackend {
    fn new(client: Client) -> Self {
        MlangBackend {
            client,
            documents: DashMap::new(),
            analysis_cache: DashMap::new(),
        }
    }

    async fn analyze_and_publish(&self, uri: &Url) {
        let Some(rope) = self.documents.get(uri).map(|r| r.clone()) else {
            return;
        };
        let source = rope.to_string();
        let result = Arc::new(analysis::analyze(&source));

        // Convert errors to diagnostics
        let mut diagnostics = Vec::new();

        for err in &result.parse_errors {
            let line = if err.line > 0 { err.line - 1 } else { 0 };
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position::new(line as u32, err.column as u32),
                    end: Position::new(line as u32, (err.column + 1) as u32),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("mlang".to_string()),
                message: err.message.clone(),
                ..Default::default()
            });
        }

        for err in &result.type_errors {
            // Type errors don't have precise positions yet, show at start
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 1),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("mlang-typecheck".to_string()),
                message: err.message.clone(),
                ..Default::default()
            });
        }

        self.analysis_cache.insert(uri.clone(), result);
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for MlangBackend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        "(".to_string(),
                        "<".to_string(),
                    ]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: semantic_tokens::legend(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(false),
                            ..Default::default()
                        },
                    ),
                ),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "mlang language server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let rope = Rope::from_str(&params.text_document.text);
        self.documents.insert(uri.clone(), rope);
        self.analyze_and_publish(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        if let Some(change) = params.content_changes.into_iter().last() {
            let rope = Rope::from_str(&change.text);
            self.documents.insert(uri.clone(), rope);
            self.analyze_and_publish(&uri).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.remove(&uri);
        self.analysis_cache.remove(&uri);
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let Some(rope) = self.documents.get(uri).map(|r| r.clone()) else {
            return Ok(None);
        };
        let analysis = self.analysis_cache.get(uri).map(|a| a.clone());

        Ok(hover::get_hover(&rope, pos, analysis.as_deref()))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let rope = self.documents.get(uri).map(|r| r.clone());
        let analysis = self.analysis_cache.get(uri).map(|a| a.clone());

        Ok(Some(CompletionResponse::Array(
            completion::get_completions(rope.as_ref(), pos, analysis.as_deref()),
        )))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let Some(rope) = self.documents.get(uri).map(|r| r.clone()) else {
            return Ok(None);
        };
        let Some(analysis) = self.analysis_cache.get(uri).map(|a| a.clone()) else {
            return Ok(None);
        };

        if let Some(location) = analysis::find_definition(&rope, pos, &analysis, uri) {
            Ok(Some(GotoDefinitionResponse::Scalar(location)))
        } else {
            Ok(None)
        }
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;
        let Some(rope) = self.documents.get(uri).map(|r| r.clone()) else {
            return Ok(None);
        };
        let source = rope.to_string();

        match formatter::format_source(&source) {
            Ok(formatted) => {
                if formatted == source {
                    return Ok(Some(vec![]));
                }
                let last_line = rope.len_lines().saturating_sub(1) as u32;
                let last_char = rope.line(last_line as usize).len_chars() as u32;
                let full_range = Range {
                    start: Position::new(0, 0),
                    end: Position::new(last_line, last_char),
                };
                Ok(Some(vec![TextEdit {
                    range: full_range,
                    new_text: formatted,
                }]))
            }
            Err(errors) => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("Formatter errors: {}", errors.join("; ")),
                    )
                    .await;
                Ok(None)
            }
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = &params.text_document.uri;
        let Some(rope) = self.documents.get(uri).map(|r| r.clone()) else {
            return Ok(None);
        };
        let source = rope.to_string();
        let tokens = semantic_tokens::get_semantic_tokens(&source);
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        })))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| MlangBackend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
