//! Axum integration for the [`openresponses`] crate.
//!
//! Provides a single [`router`] function that mounts HTTP and WebSocket handlers
//! for the OpenResponses API onto an Axum [`Router`].
//!
//! # Usage
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use axum::{routing::get, Router};
//!
//! // Bring your own handler that implements openresponses::ResponsesHandler
//! # use openresponses_server::MockHandler as MyBackend;
//! let handler = Arc::new(MyBackend::new());
//!
//! let app = Router::new()
//!     .nest("/v1", openresponses_axum::router(handler))
//!     .route("/health", get(|| async { "ok" }));
//! ```

mod handlers;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use openresponses::handler::ResponsesHandler;

pub use handlers::{http_compact, http_create_response, ws_upgrade};

/// Build an Axum router for the OpenResponses API.
///
/// Mount this under `/v1` in your application alongside any additional routes.
pub fn router<H: ResponsesHandler>(handler: Arc<H>) -> Router {
    Router::new()
        .route("/responses", post(http_create_response::<H>))
        .route("/responses", get(ws_upgrade::<H>))
        .route("/responses/compact", post(http_compact::<H>))
        .with_state(handler)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use openresponses_server::MockHandler;
    use tower::ServiceExt;

    fn make_app() -> axum::Router {
        super::router(Arc::new(MockHandler::new()))
    }

    async fn body_json(body: Body) -> serde_json::Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn test_basic_response_http() {
        let app = make_app();
        let req = Request::builder()
            .method("POST")
            .uri("/responses")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"model":"mock","input":[{"type":"message","role":"user","content":"hello"}]}"#,
            ))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        assert_eq!(json["status"], "completed");
        assert_eq!(json["object"], "response");
        assert!(!json["output"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_compact_endpoint() {
        let app = make_app();
        let req = Request::builder()
            .method("POST")
            .uri("/responses/compact")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"model":"mock","input":[{"type":"message","role":"user","content":"compact me"}]}"#,
            ))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        assert_eq!(json["object"], "response.compaction");
        assert_eq!(json["status"], "completed");
    }

    #[tokio::test]
    async fn test_compact_missing_model_returns_422() {
        let app = make_app();
        let req = Request::builder()
            .method("POST")
            .uri("/responses/compact")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"input":[{"type":"message","role":"user","content":"no model"}]}"#,
            ))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert!(
            resp.status() == StatusCode::UNPROCESSABLE_ENTITY
                || resp.status() == StatusCode::BAD_REQUEST,
            "expected 422 or 400, got {}",
            resp.status()
        );
    }

    #[tokio::test]
    async fn test_streaming_response_sse() {
        let app = make_app();
        let req = Request::builder()
            .method("POST")
            .uri("/responses")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"model":"mock","stream":true,"input":[{"type":"message","role":"user","content":"hello"}]}"#,
            ))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("Content-Type")
                .unwrap()
                .to_str()
                .unwrap(),
            "text/event-stream"
        );
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(text.contains("response.created"));
        assert!(text.contains("response.completed"));
        assert!(text.contains("[DONE]"));
    }

    #[tokio::test]
    async fn test_tool_calling_response() {
        let app = make_app();
        let body = serde_json::json!({
            "model": "mock",
            "input": [{"type": "message", "role": "user", "content": "weather?"}],
            "tools": [{"type": "function", "name": "get_weather", "description": "get weather"}]
        });
        let req = Request::builder()
            .method("POST")
            .uri("/responses")
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        let output = json["output"].as_array().unwrap();
        assert!(!output.is_empty());
        assert_eq!(output[0]["type"], "function_call");
    }
}
