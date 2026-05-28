//! Core types and traits for building [OpenResponses](https://www.openresponses.org)-compatible
//! servers in Rust.
//!
//! # Overview
//!
//! This crate provides everything needed to implement an OpenResponses API endpoint:
//!
//! - **[`ResponsesHandler`]** — the trait your inference backend implements
//! - **[`ResponseBuilder`]** — construct [`types::ResponseResource`] values without boilerplate
//! - **[`types`]** — all OpenResponses API types, fully serializable with serde
//! - **[`websocket`]** — [`WsSession`] for per-connection state and [`websocket::events_for_response`]
//!   for converting a completed response into the correct SSE event sequence
//!
//! # Quick start
//!
//! Implement [`ResponsesHandler`], then pass it to an integration crate such as
//! [`openresponses-axum`](https://docs.rs/openresponses-axum):
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use async_trait::async_trait;
//! use openresponses::{
//!     ResponseBuilder,
//!     handler::{ResponseOrStream, ResponsesHandler},
//!     types::{CompactResource, CompactResponseMethodPublicBody, CreateResponseBody, UsageResource},
//! };
//!
//! struct MyBackend;
//!
//! #[async_trait]
//! impl ResponsesHandler for MyBackend {
//!     type Error = std::io::Error;
//!
//!     async fn create_response(
//!         &self,
//!         req: CreateResponseBody,
//!         _auth: Option<String>,
//!     ) -> Result<ResponseOrStream, Self::Error> {
//!         let resp = ResponseBuilder::new(&req.model)
//!             .build_text("Hello!", UsageResource::new(10, 5));
//!         Ok(ResponseOrStream::Response(Box::new(resp)))
//!     }
//!
//!     async fn compact_response(
//!         &self,
//!         _req: CompactResponseMethodPublicBody,
//!         _auth: Option<String>,
//!     ) -> Result<CompactResource, Self::Error> {
//!         unimplemented!()
//!     }
//! }
//! ```

pub mod builder;
pub mod handler;
pub mod types;
pub mod websocket;

pub use builder::{
    ResponseBuilder, new_call_id, new_function_call_id, new_id, new_message_id, new_response_id,
};
pub use handler::{EventStream, ResponseOrStream, ResponsesHandler};
pub use types::*;
pub use websocket::{WsSession, events_for_response};
