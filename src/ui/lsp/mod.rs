//! The howmany language server.
//!
//! One job: as you type, say what the file in front of you is made of, and say
//! so where you are already looking. The counts are the same ones the report
//! prints -- the same classifier, the same comment tables -- so the number
//! above your first line and the number in `howmany .` can never disagree.
//!
//! The server counts the editor's in-memory buffer rather than the file on
//! disk, which is the whole point: unsaved work counts too.

pub mod surface;
pub mod uri;

use crate::core::counter::{self, CodeCounter};
use crate::utils::config::{HowManyConfig, LspPreferences};
use crate::utils::errors::{HowManyError, Result};
use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
    PublishDiagnostics,
};
use lsp_types::request::CodeLensRequest;
use lsp_types::{
    CodeLensOptions, CodeLensParams, PublishDiagnosticsParams, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
};
use std::collections::HashMap;
use std::path::PathBuf;

impl From<lsp_server::ProtocolError> for HowManyError {
    fn from(err: lsp_server::ProtocolError) -> Self {
        Self::language_server(err.to_string())
    }
}

/// Everything the server knows about one open buffer.
struct Document {
    text: String,
    /// The classifier key -- an extension, or a recognized name like
    /// `Dockerfile`. Resolved once when the buffer opens, because it cannot
    /// change without the buffer being reopened under a new URI.
    key: String,
}

impl Document {
    fn new(uri: &Uri, text: String) -> Self {
        let path = uri::to_path(uri).unwrap_or_else(|| PathBuf::from(uri::file_name(uri)));
        Self {
            text,
            key: counter::classify_key(&path).into_owned(),
        }
    }
}

/// Serve the protocol over stdin and stdout until the client says to stop.
///
/// Nothing may be written to stdout but protocol messages, so anything this
/// wants to say goes to stderr, where every client routes it to a log.
pub fn serve() -> Result<()> {
    let (connection, threads) = Connection::stdio();

    let capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        code_lens_provider: Some(CodeLensOptions {
            resolve_provider: Some(false),
        }),
        ..Default::default()
    })?;

    connection.initialize(capabilities)?;
    let served = Server::new().run(&connection);

    // The writer thread runs until every sender is gone, so the connection has
    // to be dropped before waiting on it. Holding it across the join is a
    // deadlock: the process stays alive after the editor has said goodbye.
    drop(connection);
    served?;

    threads.join().map_err(HowManyError::from)
}

struct Server {
    documents: HashMap<Uri, Document>,
    counter: CodeCounter,
    prefs: LspPreferences,
}

impl Server {
    fn new() -> Self {
        // A malformed config must not take the editor down with it; the server
        // says so once on stderr and carries on with the defaults.
        let prefs = HowManyConfig::load()
            .inspect_err(|err| eprintln!("howmany: using default settings ({err})"))
            .unwrap_or_default()
            .lsp;

        Self {
            documents: HashMap::new(),
            counter: CodeCounter::new(),
            prefs,
        }
    }

    fn run(&mut self, connection: &Connection) -> Result<()> {
        for message in &connection.receiver {
            match message {
                Message::Request(request) => {
                    if connection.handle_shutdown(&request)? {
                        return Ok(());
                    }
                    if let Some(response) = self.respond(request) {
                        connection.sender.send(response.into()).ok();
                    }
                }
                Message::Notification(notification) => {
                    for published in self.absorb(notification) {
                        connection.sender.send(published.into()).ok();
                    }
                }
                Message::Response(_) => {}
            }
        }

        Ok(())
    }

    /// Answer a request, or ignore it if it is not one we advertised.
    fn respond(&self, request: Request) -> Option<Response> {
        let (id, params) = cast::<CodeLensRequest>(request)?;
        Some(self.code_lens(id, params))
    }

    fn code_lens(&self, id: RequestId, params: CodeLensParams) -> Response {
        let lenses: Vec<_> = self
            .documents
            .get(&params.text_document.uri)
            .filter(|_| self.prefs.code_lens)
            .and_then(|document| self.measure(document))
            .map(|stats| {
                vec![surface::lens(
                    &stats,
                    &self.key_of(&params.text_document.uri),
                )]
            })
            .unwrap_or_default();

        Response::new_ok(id, lenses)
    }

    /// Take in a change to the open set, and say what the client should now be
    /// told about the affected file.
    fn absorb(&mut self, notification: Notification) -> Vec<Notification> {
        if let Some(params) = cast_notification::<DidOpenTextDocument>(&notification) {
            let uri = params.text_document.uri;
            self.documents
                .insert(uri.clone(), Document::new(&uri, params.text_document.text));
            return self.diagnose(uri).into_iter().collect();
        }

        if let Some(params) = cast_notification::<DidChangeTextDocument>(&notification) {
            let uri = params.text_document.uri;
            // Full sync, so the last change carries the entire buffer.
            if let Some(change) = params.content_changes.into_iter().next_back() {
                match self.documents.get_mut(&uri) {
                    Some(document) => document.text = change.text,
                    None => {
                        self.documents
                            .insert(uri.clone(), Document::new(&uri, change.text));
                    }
                }
            }
            return self.diagnose(uri).into_iter().collect();
        }

        if let Some(params) = cast_notification::<DidCloseTextDocument>(&notification) {
            let uri = params.text_document.uri;
            self.documents.remove(&uri);
            // Diagnostics outlive the buffer unless they are explicitly
            // cleared, and a closed file's warnings sitting in the problems
            // panel are indistinguishable from real ones.
            return vec![publish(uri, Vec::new())];
        }

        Vec::new()
    }

    /// The classifier key for a URI, whether or not it is currently open.
    fn key_of(&self, uri: &Uri) -> String {
        self.documents
            .get(uri)
            .map(|document| document.key.clone())
            .unwrap_or_else(|| Document::new(uri, String::new()).key)
    }

    fn measure(&self, document: &Document) -> Option<crate::core::types::FileStats> {
        self.counter
            .count_bytes(document.text.as_bytes(), &document.key)
            .ok()
    }

    fn diagnose(&self, uri: Uri) -> Option<Notification> {
        let stats = self.measure(self.documents.get(&uri)?)?;
        Some(publish(uri, surface::diagnostics(&stats, &self.prefs)))
    }
}

fn publish(uri: Uri, diagnostics: Vec<lsp_types::Diagnostic>) -> Notification {
    Notification::new(
        PublishDiagnostics::METHOD.to_string(),
        PublishDiagnosticsParams {
            uri,
            diagnostics,
            version: None,
        },
    )
}

/// Interpret a request as `R`, or decline it.
///
/// A request for something we never advertised, and a request whose parameters
/// do not parse, are both the client's business rather than grounds for taking
/// the server down.
fn cast<R>(request: Request) -> Option<(RequestId, R::Params)>
where
    R: lsp_types::request::Request,
{
    match request.extract::<R::Params>(R::METHOD) {
        Ok(pair) => Some(pair),
        Err(ExtractError::JsonError { method, error }) => {
            eprintln!("howmany: could not read {method}: {error}");
            None
        }
        Err(ExtractError::MethodMismatch(_)) => None,
    }
}

fn cast_notification<N>(notification: &Notification) -> Option<N::Params>
where
    N: lsp_types::notification::Notification,
{
    (notification.method == N::METHOD)
        .then(|| serde_json::from_value(notification.params.clone()).ok())
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
        VersionedTextDocumentIdentifier,
    };
    use std::str::FromStr;

    fn uri(text: &str) -> Uri {
        Uri::from_str(text).expect("test URIs must parse")
    }

    fn server() -> Server {
        Server {
            documents: HashMap::new(),
            counter: CodeCounter::new(),
            prefs: LspPreferences::default(),
        }
    }

    fn opened(uri: &Uri, text: &str) -> Notification {
        Notification::new(
            DidOpenTextDocument::METHOD.to_string(),
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "rust".to_string(),
                    version: 1,
                    text: text.to_string(),
                },
            },
        )
    }

    fn changed(uri: &Uri, text: &str) -> Notification {
        Notification::new(
            DidChangeTextDocument::METHOD.to_string(),
            DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version: 2,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: text.to_string(),
                }],
            },
        )
    }

    fn lens_titles(server: &Server, uri: &Uri) -> Vec<String> {
        let response = server.code_lens(
            RequestId::from(1),
            CodeLensParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            },
        );
        let lenses: Vec<lsp_types::CodeLens> =
            serde_json::from_value(response.response_result.expect("code lens must succeed"))
                .unwrap();
        lenses
            .into_iter()
            .filter_map(|lens| lens.command.map(|c| c.title))
            .collect()
    }

    #[test]
    fn opening_a_file_makes_its_lens_available() {
        let mut server = server();
        let file = uri("file:///p/main.rs");
        server.absorb(opened(&file, "// a\nfn main() {}\n"));

        let titles = lens_titles(&server, &file);
        assert_eq!(titles.len(), 1, "one file, one lens");
        assert!(titles[0].starts_with("2 Rust"), "{}", titles[0]);
    }

    /// The buffer is the source of truth, so a lens must follow an edit that
    /// has not been saved anywhere.
    #[test]
    fn the_lens_follows_unsaved_edits() {
        let mut server = server();
        let file = uri("file:///p/main.rs");
        server.absorb(opened(&file, "fn main() {}\n"));
        assert!(lens_titles(&server, &file)[0].starts_with("1 Rust"));

        server.absorb(changed(
            &file,
            "fn main() {}\nfn other() {}\nfn third() {}\n",
        ));
        assert!(lens_titles(&server, &file)[0].starts_with("3 Rust"));
    }

    #[test]
    fn a_file_nobody_opened_has_no_lens() {
        assert!(lens_titles(&server(), &uri("file:///p/ghost.rs")).is_empty());
    }

    #[test]
    fn closing_a_file_forgets_it_and_clears_its_warnings() {
        let mut server = server();
        let file = uri("file:///p/main.rs");
        server.absorb(opened(&file, "fn main() {}\n"));

        let published = server.absorb(Notification::new(
            DidCloseTextDocument::METHOD.to_string(),
            DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier { uri: file.clone() },
            },
        ));

        assert!(server.documents.is_empty(), "the buffer should be dropped");
        let params: PublishDiagnosticsParams =
            serde_json::from_value(published[0].params.clone()).unwrap();
        assert!(
            params.diagnostics.is_empty(),
            "a closed file must leave nothing in the problems panel"
        );
    }

    #[test]
    fn crossing_a_threshold_publishes_a_warning() {
        let mut server = server();
        server.prefs.max_file_lines = 3;
        let file = uri("file:///p/long.rs");

        let published = server.absorb(opened(&file, "fn a() {}\n".repeat(10).as_str()));
        let params: PublishDiagnosticsParams =
            serde_json::from_value(published[0].params.clone()).unwrap();

        assert_eq!(params.uri, file);
        assert_eq!(params.diagnostics.len(), 1);
        assert_eq!(params.diagnostics[0].source.as_deref(), Some("howmany"));
    }

    /// Editing a file back under the limit must retract the warning, not merely
    /// stop adding new ones.
    #[test]
    fn falling_back_under_a_threshold_retracts_the_warning() {
        let mut server = server();
        server.prefs.max_file_lines = 3;
        let file = uri("file:///p/long.rs");
        server.absorb(opened(&file, "fn a() {}\n".repeat(10).as_str()));

        let published = server.absorb(changed(&file, "fn a() {}\n"));
        let params: PublishDiagnosticsParams =
            serde_json::from_value(published[0].params.clone()).unwrap();
        assert!(params.diagnostics.is_empty());
    }

    /// A file's syntax comes from its name, so the same text must count
    /// differently under a different extension.
    #[test]
    fn comment_syntax_comes_from_the_file_name() {
        let mut server = server();
        let rust = uri("file:///p/a.rs");
        let python = uri("file:///p/a.py");
        server.absorb(opened(&rust, "# not a rust comment\n"));
        server.absorb(opened(&python, "# a python comment\n"));

        assert!(lens_titles(&server, &rust)[0].contains("code"));
        assert!(lens_titles(&server, &python)[0].contains("comment"));
    }

    /// The client is not to be trusted to send well-formed parameters, and a
    /// bad one must not end the session.
    #[test]
    fn a_malformed_notification_is_ignored() {
        let mut server = server();
        let published = server.absorb(Notification::new(
            DidOpenTextDocument::METHOD.to_string(),
            serde_json::json!({ "nonsense": true }),
        ));
        assert!(published.is_empty());
        assert!(server.documents.is_empty());
    }

    #[test]
    fn an_unknown_request_is_declined_rather_than_answered() {
        let request = Request::new(
            RequestId::from(1),
            "textDocument/hover".to_string(),
            serde_json::json!({}),
        );
        assert!(server().respond(request).is_none());
    }

    /// Turning the lens off in the config must actually turn it off.
    #[test]
    fn the_lens_can_be_switched_off() {
        let mut server = server();
        server.prefs.code_lens = false;
        let file = uri("file:///p/main.rs");
        server.absorb(opened(&file, "fn main() {}\n"));

        assert!(lens_titles(&server, &file).is_empty());
    }
}
