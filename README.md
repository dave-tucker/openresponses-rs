# openresponses-rs

Rust implementation of the [OpenResponses API](https://www.openresponses.org/specification) — a library crate for inference engines and proxy servers that need to expose a conformant OpenResponses endpoint.

## Workspace

| Crate | Purpose |
|---|---|
| `openresponses` | Core types, `ResponsesHandler` trait, WebSocket session logic — no web-framework dependency |
| `openresponses-axum` | Axum router integration wiring `ResponsesHandler` to HTTP and WebSocket endpoints |
| `openresponses-server` | Reference binary (`openresponses-server`) and `MockHandler` library used for compliance testing |

## Quick start

Add `openresponses` and `openresponses-axum` to your `Cargo.toml`:

```toml
[dependencies]
openresponses = { git = "https://github.com/dave-tucker/openresponses-rs" }
openresponses-axum = { git = "https://github.com/dave-tucker/openresponses-rs" }
```

Implement the trait and mount the router:

```rust
use std::sync::Arc;
use async_trait::async_trait;
use openresponses::{
    handler::{ResponseOrStream, ResponsesHandler},
    types::{CompactResource, CompactResponseMethodPublicBody, CreateResponseBody, UsageResource},
    ResponseBuilder,
};

struct MyBackend;

#[async_trait]
impl ResponsesHandler for MyBackend {
    type Error = anyhow::Error;

    async fn create_response(
        &self,
        req: CreateResponseBody,
        _auth: Option<String>,
    ) -> Result<ResponseOrStream, Self::Error> {
        let resp = ResponseBuilder::new(&req.model)
            .build_text("Hello!", UsageResource::new(10, 5));
        Ok(ResponseOrStream::Response(Box::new(resp)))
    }

    async fn compact_response(
        &self,
        _req: CompactResponseMethodPublicBody,
        _auth: Option<String>,
    ) -> Result<CompactResource, Self::Error> {
        unimplemented!("compaction not supported")
    }
}

#[tokio::main]
async fn main() {
    let handler = Arc::new(MyBackend);
    let app = axum::Router::new()
        .nest("/v1", openresponses_axum::router(handler));
    // ...
}
```

See [`crates/openresponses-axum/examples/echo_server.rs`](crates/openresponses-axum/examples/echo_server.rs) for a complete working example including streaming and WebSocket support.

## Key types

### `ResponseBuilder`

Builds a `ResponseResource` with sensible defaults — only set what your backend cares about.

```rust
use openresponses::{ResponseBuilder, builder::new_response_id, types::UsageResource};

// Non-streaming text reply
let resp = ResponseBuilder::new("my-model")
    .id(new_response_id())
    .store(false)
    .temperature(0.7)
    .build_text("Hello!", UsageResource::new(10, 5));

// Or supply arbitrary output items (function calls, reasoning, etc.)
let resp = ResponseBuilder::new("my-model")
    .build_with_output(output_items, usage);
```

### `UsageResource::new`

```rust
let usage = UsageResource::new(input_tokens, output_tokens);
// total_tokens, cached_tokens, reasoning_tokens all computed / defaulted to zero
```

### Streaming

Convert a completed `ResponseResource` into the full SSE event sequence:

```rust
use openresponses::websocket::events_for_response;
use futures::stream;

let events = events_for_response(resp);
Ok(ResponseOrStream::Stream(Box::pin(stream::iter(events))))
```

### ID helpers

```rust
use openresponses::{new_response_id, new_message_id, new_function_call_id, new_call_id};

let resp_id = new_response_id();   // "resp_<uuid>"
let msg_id  = new_message_id();    // "msg_<uuid>"
let fc_id   = new_function_call_id(); // "fc_<uuid>"
let call_id = new_call_id();       // "call_<uuid>"
```

## `ResponsesHandler` trait

```rust
#[async_trait]
pub trait ResponsesHandler: Send + Sync + 'static {
    type Error: Display + Send + Sync + 'static;

    async fn create_response(
        &self,
        req: CreateResponseBody,
        auth: Option<String>,
    ) -> Result<ResponseOrStream, Self::Error>;

    async fn compact_response(
        &self,
        req: CompactResponseMethodPublicBody,
        auth: Option<String>,
    ) -> Result<CompactResource, Self::Error>;

    // Default: returns None (no global persistence)
    async fn get_response(&self, id: &str) -> Option<ResponseResource> { None }
}
```

`get_response` has a default implementation returning `None`. Override it only if your backend persists responses and needs WebSocket multi-turn continuation across reconnects.

## Mock server

The `openresponses-server` binary is a fully compliant mock backend useful for local development and compliance testing.

```
cargo run --bin openresponses-server -- --port 3000 --api-key secret
```

| Flag | Env | Default | Description |
|---|---|---|---|
| `--port` | `PORT` | `3000` | Listen port |
| `--api-key` | `API_KEY` | _(none)_ | Require `Authorization: Bearer <key>` |

Or with Docker:

```
docker build -t openresponses-server .
docker run -p 3000:3000 openresponses-server
```

## Development

```bash
just check       # fmt + clippy + build + unit tests
just fmt         # auto-format
just run         # start mock server on :3000
just compliance  # run 17 compliance tests in containers (requires podman + ../openresponses)
just ci          # check + compliance
```

Compliance tests require a checkout of [openresponses](https://github.com/openresponses/openresponses) at `../openresponses` and [Podman](https://podman.io).

## Compliance

17/17 OpenResponses compliance tests pass. Tests cover: basic response, streaming, WebSocket (sequential, multi-turn, reconnect, compaction, failed continuation, previous-response-not-found), tool calling, image input, system prompt, assistant phases, and compact response.

## License

MIT OR Apache-2.0
