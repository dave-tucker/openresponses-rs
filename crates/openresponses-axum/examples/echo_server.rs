//! Minimal example: an echo backend that implements `ResponsesHandler`.
//!
//! Run with:
//!
//!   cargo run --example echo_server
//!
//! Then try it:
//!
//!   curl -s -X POST http://localhost:3000/v1/responses \
//!        -H 'Content-Type: application/json' \
//!        -d '{"model":"echo","input":[{"type":"message","role":"user","content":"hello"}]}' | jq .
//!
//!   curl -s -X POST http://localhost:3000/v1/responses \
//!        -H 'Content-Type: application/json' \
//!        -d '{"model":"echo","stream":true,"input":[{"type":"message","role":"user","content":"hello"}]}'

use std::sync::Arc;

use async_trait::async_trait;
use axum::{routing::get, Router};
use openresponses::{
    builder::ResponseBuilder,
    handler::{ResponseOrStream, ResponsesHandler},
    new_call_id,
    types::{
        CompactResource, CompactResponseMethodPublicBody, CompactionOutputItem, CreateResponseBody,
        OutputItem, StringOrItems, UsageResource,
    },
    websocket::events_for_response,
};

// ---------------------------------------------------------------------------
// Echo backend
// ---------------------------------------------------------------------------

struct EchoHandler;

#[derive(Debug)]
struct EchoError(String);

impl std::fmt::Display for EchoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait]
impl ResponsesHandler for EchoHandler {
    type Error = EchoError;

    async fn create_response(
        &self,
        req: CreateResponseBody,
        _auth: Option<String>,
    ) -> Result<ResponseOrStream, EchoError> {
        // Extract the last user message as the text to echo back.
        let echo_text = match &req.input {
            StringOrItems::String(s) => s.clone(),
            StringOrItems::Items(items) => items
                .iter()
                .rev()
                .find_map(|item| {
                    use openresponses::types::{ContentPart, ItemParam, MessageContent};
                    if let ItemParam::Message(msg) = item {
                        match &msg.content {
                            MessageContent::String(s) => Some(s.clone()),
                            MessageContent::Parts(parts) => parts.iter().find_map(|p| {
                                if let ContentPart::InputText { text } = p {
                                    Some(text.clone())
                                } else {
                                    None
                                }
                            }),
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "(nothing to echo)".to_string()),
        };

        let usage = UsageResource::new(1, 1);
        let resp = ResponseBuilder::new(&req.model)
            .store(req.store.unwrap_or(true))
            .build_text(format!("Echo: {echo_text}"), usage);

        if req.stream.unwrap_or(false) {
            let events = events_for_response(resp);
            Ok(ResponseOrStream::Stream(Box::pin(futures::stream::iter(
                events,
            ))))
        } else {
            Ok(ResponseOrStream::Response(Box::new(resp)))
        }
    }

    async fn compact_response(
        &self,
        _req: CompactResponseMethodPublicBody,
        _auth: Option<String>,
    ) -> Result<CompactResource, EchoError> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        Ok(CompactResource {
            id: format!("resp_compact_{now}"),
            object: "response.compaction".to_string(),
            created_at: now,
            status: "completed".to_string(),
            output: vec![OutputItem::Compaction(CompactionOutputItem {
                id: new_call_id(),
                status: "completed".to_string(),
                encrypted_content: Some("echo-compacted".to_string()),
            })],
            usage: UsageResource::new(1, 0),
        })
    }
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let handler = Arc::new(EchoHandler);

    let app = Router::new()
        // Mount the OpenResponses API under /v1
        .nest("/v1", openresponses_axum::router(handler))
        // Health endpoint for load-balancers / Docker healthchecks
        .route("/health", get(|| async { "ok" }));

    let addr = "0.0.0.0:3000";
    println!("echo server listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
