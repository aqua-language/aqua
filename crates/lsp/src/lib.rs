use std::error::Error;

use compiler::report::span::Span;
use compiler::Compiler;
use lsp_types::notification::DidSaveTextDocument;
use lsp_types::notification::Notification;
use lsp_types::notification::PublishDiagnostics;
use lsp_types::request::HoverRequest;
use lsp_types::request::WorkspaceDiagnosticRefresh;
use lsp_types::Diagnostic;
use lsp_types::DiagnosticOptions;
use lsp_types::DiagnosticServerCapabilities;
use lsp_types::DiagnosticSeverity;
use lsp_types::DidSaveTextDocumentParams;
use lsp_types::HoverParams;
use lsp_types::HoverProviderCapability;
use lsp_types::InitializeParams;
use lsp_types::Position;
use lsp_types::PublishDiagnosticsParams;
use lsp_types::SaveOptions;
use lsp_types::ServerCapabilities;

use lsp_server::Connection;
use lsp_server::ExtractError;
use lsp_server::Message;
use lsp_server::Request;
use lsp_server::RequestId;
use lsp_types::TextDocumentSyncOptions;
use lsp_types::TextDocumentSyncSaveOptions;

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: None,
                    change: None,
                    will_save: None,
                    will_save_wait_until: None,
                    save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                        include_text: Some(true),
                    })),
                },
            )),
            diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
                identifier: None,
                inter_file_dependencies: false,
                workspace_diagnostics: true,
                work_done_progress_options: Default::default(),
            })),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            ..Default::default()
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn Error + Sync + Send>> {
        let (connection, io_threads) = Connection::stdio();
        let server_capabilities = serde_json::to_value(self.capabilities()).unwrap();
        let initialization_params = connection.initialize(server_capabilities)?;
        self.event_loop(connection, initialization_params)?;
        io_threads.join()?;
        Ok(())
    }

    fn event_loop(
        &self,
        connection: Connection,
        params: serde_json::Value,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        let _params: InitializeParams = serde_json::from_value(params).unwrap();
        for msg in &connection.receiver {
            eprintln!("Got message {msg:?}");
            match msg {
                Message::Request(request) => {
                    if connection.handle_shutdown(&request)? {
                        return Ok(());
                    }
                    let request = match self.cast_request::<WorkspaceDiagnosticRefresh>(request) {
                        Ok(_params) => {
                            continue;
                        }
                        Err(not) => not,
                    };
                    let request = match self.cast_request::<HoverRequest>(request) {
                        Ok((_id, params)) => {
                            self.handle_hover(params, &connection);
                            continue;
                        }
                        Err(not) => not,
                    };
                    connection
                        .sender
                        .send(Message::Response(lsp_server::Response::new_err(
                            request.id,
                            lsp_server::ErrorCode::MethodNotFound as i32,
                            "Unknown request".to_string(),
                        )))
                        .unwrap();
                }
                Message::Response(_resp) => {}
                Message::Notification(notification) => {
                    let notification =
                        match self.cast_notification::<DidSaveTextDocument>(notification) {
                            Ok(params) => {
                                self.handle_did_save_text_document(params, &connection);
                                continue;
                            }
                            Err(notification) => notification,
                        };
                    eprintln!("unhandled notification: {:?}", notification);
                }
            }
        }
        Ok(())
    }

    fn handle_hover(&self, params: HoverParams, _connection: &Connection) {
        let pos = params.text_document_position_params.position;
        let _line = pos.line as usize;
        let _col = pos.character as usize;
        todo!()
    }

    fn handle_did_save_text_document(
        &self,
        params: DidSaveTextDocumentParams,
        connection: &Connection,
    ) {
        let uri = params.text_document.uri;
        let source = params.text.unwrap();
        let name = uri.path().as_str();

        let mut compiler = compiler::Compiler::default();
        compiler.init();
        let program = compiler.parse(name, source);
        compiler.compile(&program);
        let mut report = std::mem::take(&mut compiler.report);

        connection
            .sender
            .send(Message::Notification(lsp_server::Notification {
                method: PublishDiagnostics::METHOD.to_string(),
                params: serde_json::to_value(PublishDiagnosticsParams {
                    uri: uri.clone(),
                    diagnostics: report
                        .diags
                        .drain(..)
                        .map(|diag| self.diagnostic(&mut compiler, &diag))
                        .collect::<Vec<_>>(),
                    version: None,
                })
                .unwrap(),
            }))
            .unwrap();
    }

    fn cast_request<R>(&self, req: lsp_server::Request) -> Result<(RequestId, R::Params), Request>
    where
        R: lsp_types::request::Request,
        R::Params: serde::de::DeserializeOwned,
    {
        match req.extract(R::METHOD) {
            Ok((id, params)) => Ok((id, params)),
            Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
            Err(ExtractError::MethodMismatch(req)) => Err(req),
        }
    }

    fn cast_notification<N>(
        &self,
        notification: lsp_server::Notification,
    ) -> Result<N::Params, lsp_server::Notification>
    where
        N: lsp_types::notification::Notification,
        N::Params: serde::de::DeserializeOwned,
    {
        match notification.extract(N::METHOD) {
            Ok(params) => Ok(params),
            Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
            Err(ExtractError::MethodMismatch(not)) => Err(not),
        }
    }

    fn diagnostic(
        &self,
        compiler: &mut Compiler,
        diag: &compiler::report::Diagnostic,
    ) -> lsp_types::Diagnostic {
        Diagnostic {
            range: self.span_to_range(compiler, diag.label.span),
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: None,
            message: std::iter::once(diag.label.text.as_str())
                .chain(diag.messages.iter().map(|msg| msg.text.as_str()))
                .collect::<Vec<_>>()
                .join("\n"),
            related_information: None,
            tags: None,
            data: None,
        }
    }

    fn span_to_range(&self, compiler: &mut Compiler, span: Span) -> lsp_types::Range {
        let file = span.file().unwrap();
        let pos0 = compiler.sources.get_pos(file, span.start().unwrap());
        let pos1 = compiler.sources.get_pos(file, span.end().unwrap());
        lsp_types::Range {
            start: Position {
                line: pos0.0 as u32,
                character: pos0.1 as u32,
            },
            end: Position {
                line: pos1.0 as u32,
                character: pos1.1 as u32,
            },
        }
    }
}
