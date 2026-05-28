use std::fmt;
use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;

use crate::types::{
    CompactResource, CompactResponseMethodPublicBody, CreateResponseBody, ResponseResource,
    StreamEvent,
};

/// A streaming response: a pinned stream of `StreamEvent`.
pub type EventStream = Pin<Box<dyn Stream<Item = StreamEvent> + Send + 'static>>;

/// The result of calling `create_response` — either a single response or a stream.
#[allow(dead_code)]
pub enum ResponseOrStream {
    Response(Box<ResponseResource>),
    Stream(EventStream),
}

/// Trait that must be implemented by any backend.
#[async_trait]
pub trait ResponsesHandler: Send + Sync + 'static {
    type Error: fmt::Display + Send + Sync + 'static;

    /// Create a response (streaming or non-streaming).
    async fn create_response(
        &self,
        req: CreateResponseBody,
        auth: Option<String>,
    ) -> Result<ResponseOrStream, Self::Error>;

    /// Compact a response chain.
    async fn compact_response(
        &self,
        req: CompactResponseMethodPublicBody,
        auth: Option<String>,
    ) -> Result<CompactResource, Self::Error>;

    /// Look up a stored response by ID. Used for WebSocket continuation validation.
    /// Returns `None` if not found (or not stored globally).
    async fn get_response(&self, _id: &str) -> Option<ResponseResource> {
        None
    }
}
