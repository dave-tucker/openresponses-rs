//! Reference [`MockHandler`] implementation of [`openresponses::ResponsesHandler`].
//!
//! Passes all 17 OpenResponses compliance tests without a real LLM. Useful as a
//! test double in integration tests or as a starting point for a real backend.
//!
//! The `openresponses-server` binary (built from `src/main.rs`) wraps this handler
//! in an Axum server with `--port` and `--api-key` CLI flags.

mod mock;

pub use mock::MockHandler;
