# AGENTS.md — openresponses-rs

Guidance for AI coding agents working in this repository.

## Build and test

```bash
just check      # canonical: fmt check + clippy + build + unit tests (must pass before any commit)
just fmt        # auto-fix formatting
just run        # start mock server on :3000
just compliance # end-to-end compliance suite in containers (needs podman + ../openresponses)
just ci         # check + compliance (full pipeline)
```

Run `just check` after every non-trivial change. All clippy warnings are errors (`-D warnings`).

## Workspace layout

```
crates/
  openresponses/        # core library: types, ResponsesHandler trait, WsSession, builder
  openresponses-axum/   # Axum router integration (HTTP + WebSocket handlers)
  openresponses-server/ # binary (openresponses-server) + MockHandler lib target
scripts/
  regexp-fix.js         # Bun/JSC regex preload — do not delete, required by compliance tests
Dockerfile              # multi-stage: rust:1.85-slim → debian:bookworm-slim
docker-compose.test.yml # compliance test harness (server + oven/bun:1)
justfile
```

## Key invariants

**`FunctionTool.strict` must always serialize.** It is `Option<bool>` but has no `skip_serializing_if`, so absent values appear as `null`. The compliance test's zod schema expects `boolean | null`, not `undefined`. Do not add `skip_serializing_if` here.

**WebSocket handler must not send `[DONE]`.** The compliance test advances the turn index on `response.completed`, not on `[DONE]`. Sending `[DONE]` after a terminal event causes a turn-index race in multi-turn tests. `WsOutbound::Done` must not appear in `handle_message` output.

**`CompactResource` requires `created_at` and `usage`.** Both fields are mandatory in the spec. Always populate them.

**`events_for_response` is the canonical streaming helper.** Use it to convert a completed `ResponseResource` into the correct SSE event sequence. Don't generate events by hand.

## Adding a new route or handler

1. Add types to `crates/openresponses/src/types/` (keep types framework-agnostic).
2. Add or extend the trait method in `crates/openresponses/src/handler.rs`. Provide a default impl if it can be optional.
3. Wire the route in `crates/openresponses-axum/src/lib.rs` / `handlers.rs`.
4. Update `MockHandler` in `crates/openresponses-server/src/mock.rs` to implement the new method.
5. Run `just check`.

## Builder and ID utilities

Use these instead of constructing `ResponseResource` by hand:

```rust
use openresponses::{ResponseBuilder, new_response_id, new_message_id, types::UsageResource};

let resp = ResponseBuilder::new("model-name")
    .store(false)
    .build_text("reply text", UsageResource::new(10, 5));
```

ID helpers: `new_response_id()` → `resp_<uuid>`, `new_message_id()` → `msg_<uuid>`, `new_function_call_id()` → `fc_<uuid>`, `new_call_id()` → `call_<uuid>`.

## Compliance tests

17 tests, all must pass. Run with `just compliance` (requires podman and `../openresponses` checkout).

The compliance container uses `oven/bun:1` which runs JavaScriptCore. JSC rejects `{n,m}` quantifiers where `m > 65535` or `n > m`. `scripts/regexp-fix.js` is loaded as a Bun preload to patch `globalThis.RegExp`. The `docker-compose.test.yml` invokes bun directly (`bun --preload ...`) rather than via `bun run <script>` because the latter spawns a subprocess that doesn't inherit `--preload`.

## Mock backend behavior

- No tools in request → returns a text message output item.
- Tools present → returns a `function_call` output item using the first tool's name, `call_id: new_call_id()`, `arguments: "{}"`.
- `store: false` responses are kept in `WsSession.local_store` (per-connection) rather than the global store.
- Failed continuation (bad `call_id`) evicts the referenced `previous_response_id` from `local_store`.

## Code style

- No comments explaining what code does — names should be self-explanatory.
- Comments only for non-obvious WHY: hidden constraints, subtle invariants, workarounds.
- No `#[allow(dead_code)]` on new public items — use the existing ones only where serde-only fields aren't called directly.
- Keep `#[allow(dead_code)]` on types that exist purely for serde (de)serialization without being constructed in Rust.
