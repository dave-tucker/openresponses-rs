use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{State, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use futures::StreamExt;
use openresponses::{
    handler::{ResponseOrStream, ResponsesHandler},
    types::{
        CompactResponseMethodPublicBody, CreateResponseBody, WebSocketResponseCreateEvent, WsError,
        WsOutbound,
    },
    websocket::WsSession,
};
use serde_json::json;

// ---------------------------------------------------------------------------
// Helper: extract Bearer token from Authorization header
// ---------------------------------------------------------------------------
fn extract_auth(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

// ---------------------------------------------------------------------------
// POST /responses  (non-streaming or streaming)
// ---------------------------------------------------------------------------

pub async fn http_create_response<H: ResponsesHandler>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Json(body): Json<CreateResponseBody>,
) -> Response {
    let auth = extract_auth(&headers);

    match handler.create_response(body, auth).await {
        Err(e) => {
            let body = json!({
                "error": {
                    "code": "internal_error",
                    "message": e.to_string(),
                    "param": null
                }
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
        }
        Ok(ResponseOrStream::Response(resp)) => {
            Json(serde_json::to_value(&*resp).unwrap()).into_response()
        }
        Ok(ResponseOrStream::Stream(stream)) => {
            // Build an SSE body
            let sse_stream = stream.map(|event| {
                let json = serde_json::to_string(&event).unwrap_or_default();
                Ok::<_, std::convert::Infallible>(format!("data: {json}\n\n"))
            });
            // Add the [DONE] sentinel
            let done_stream = futures::stream::once(async {
                Ok::<_, std::convert::Infallible>("data: [DONE]\n\n".to_string())
            });
            let combined = sse_stream.chain(done_stream);
            let body = Body::from_stream(combined);
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/event-stream")
                .header("Cache-Control", "no-cache")
                .header("X-Accel-Buffering", "no")
                .body(body)
                .unwrap()
        }
    }
}

// ---------------------------------------------------------------------------
// POST /responses/compact
// ---------------------------------------------------------------------------

pub async fn http_compact<H: ResponsesHandler>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Json(body): Json<CompactResponseMethodPublicBody>,
) -> Response {
    let auth = extract_auth(&headers);
    match handler.compact_response(body, auth).await {
        Ok(compact) => Json(serde_json::to_value(&compact).unwrap()).into_response(),
        Err(e) => {
            let body = json!({
                "error": {
                    "code": "internal_error",
                    "message": e.to_string(),
                    "param": null
                }
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// GET /responses  (WebSocket upgrade)
// ---------------------------------------------------------------------------

pub async fn ws_upgrade<H: ResponsesHandler>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let auth = extract_auth(&headers);
    ws.on_upgrade(move |socket| ws_handler(socket, handler, auth))
}

async fn ws_handler<H: ResponsesHandler>(
    socket: axum::extract::ws::WebSocket,
    handler: Arc<H>,
    auth: Option<String>,
) {
    use axum::extract::ws::Message;
    use futures::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();
    let mut session = WsSession::new(handler, auth);

    while let Some(Ok(msg)) = receiver.next().await {
        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };

        // Parse the incoming event
        let event: WebSocketResponseCreateEvent = match serde_json::from_str(&text) {
            Ok(e) => e,
            Err(e) => {
                let err = WsError::new(400, "invalid_request", &e.to_string(), None);
                let json = serde_json::to_string(&err).unwrap_or_default();
                let _ = sender.send(Message::Text(json.into())).await;
                continue;
            }
        };

        // Handle
        let outbound = session.handle_message(event).await;

        for msg in outbound {
            match msg {
                WsOutbound::Event(event) => {
                    let json = serde_json::to_string(&*event).unwrap_or_default();
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        return;
                    }
                }
                WsOutbound::Done => {
                    // Send the [DONE] sentinel as a text message
                    if sender
                        .send(Message::Text("[DONE]".to_string().into()))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                WsOutbound::Error(err) => {
                    let json = serde_json::to_string(&err).unwrap_or_default();
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        return;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Fallback JSON extractor error handler
// ---------------------------------------------------------------------------
