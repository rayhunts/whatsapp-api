# AGENTS.md

Rust workspace (edition 2024, resolver 3) implementing a WhatsApp Web API: Axum server + Dioxus 0.7 WASM web client on [`whatsapp-rust`](https://crates.io/crates/whatsapp-rust). Six `whatsapp-api-*` crates, each following Clean Architecture (`domain/`, `application/`, `infrastructure/`). `PLAN.md` is the design doc. Git repo has no commits yet; no tests, no CI exist.

## Commands

- `cargo dev` — run the dev orchestrator (`whatsapp-api-dev`), which builds and starts the backend on port 8081 and `dx serve` on port 8080 with hot reload. `dx serve` proxies `/api/*` and `/ws` to the backend, so open http://localhost:8080. Output is prefixed `[rust]` (backend) and `[dx]` (frontend).
- `cargo check -p whatsapp-api-web --target wasm32-unknown-unknown` — required after changing web code. Plain `cargo check --workspace` typechecks the web crate for the host too, which can hide wasm-only errors.
- `cargo build --workspace` and `cargo clippy --workspace` must pass (verified).
- `cargo clippy -p whatsapp-api-web --target wasm32-unknown-unknown` should also pass.
- Requires `rustup target add wasm32-unknown-unknown` and `dioxus-cli` (`dx`, `cargo install dioxus-cli --version "~0.7"`).

## Frontend build (whatsapp-api-server/build.rs)

- The frontend is compiled by `dx build` inside `build.rs` and embedded via `include_dir!("$OUT_DIR/web")` (`application/mod.rs`). `cargo:rerun-if-changed` on `whatsapp-api-web/src` and `whatsapp-api-web/Cargo.toml` means web edits trigger a server rebuild.
- `dx build` must use a separate `CARGO_TARGET_DIR=target/dx-build` — nested cargo on the workspace target dir would deadlock on the build lock. Preserve this if you touch build.rs. `whatsapp-api-web` must stay `crate-type = ["cdylib", "rlib"]` (WASM requirement); build.rs also panics if the dx output is missing, with no fallback.
- `SKIP_WEB_BUILD=1` skips the frontend build; if `dx` is missing the server still builds but serves a placeholder page. `cargo dev` sets this automatically so the embedded frontend is not built in dev.

## Dev server proxy (`whatsapp-api-web/Dioxus.toml`)

`cargo dev` relies on `dx serve`'s built-in proxy to forward frontend-relative API/WebSocket calls to the backend. `whatsapp-api-web/Dioxus.toml` configures `[[web.proxy]]` entries for `/api` and `/ws` pointing at `http://localhost:8081`. Do not rename or remove this file unless you also change the dev orchestrator.

## Architecture rules

- `whatsapp-api-engine` is the ONLY crate that depends on `whatsapp-rust`; server and web must never import it. The server goes through `EngineService` (`engine/src/application/mod.rs`) behind the `WaEngine` trait (`engine/src/domain/mod.rs`); the concrete `WhatsappEngine` is `engine/src/infrastructure/mod.rs`.
- `whatsapp-api-types` is pure serde DTOs shared by server and WASM client — no platform-specific deps allowed. `WsEvent`/`WsRequest` (serde `tag = "type"`, snake_case variants) are the WS wire format; changes must stay compatible across both sides.
- `whatsapp-rust` is pinned with `default-features = false` because its default `simd` feature needs nightly. Keep the explicit feature list (`sqlite-storage`, `tokio-transport`, `tokio-runtime`, `ureq-client`, `tokio-native`).
- Chat/message history is an in-memory registry only (capped at 1000 msgs/chat); only the WhatsApp session store persists (SQLite at `DB_PATH`). The engine keeps history sync enabled: `HistorySync` events fold chats/messages into the registry and emit `WsEvent::ChatsUpdated` (client re-fetches `/api/chats`); live messages stream in via `WsEvent::Message`. The bot auto-restarts every 2s.
- Chat JIDs are raw strings (`1234@s.whatsapp.net`, groups `@g.us`) used verbatim in `/api/chats/{jid}/messages`; groups are detected by `@g.us` in the JID. The chat window header has an info button that shows the raw JID.
- New chats can be started from the sidebar: private phone numbers, broadcast to multiple numbers, or an existing group JID. `POST /api/chats/ensure` creates a registry entry; `POST /api/chats/resolve-phone` normalizes a phone number to a JID.
- Automation rules are persisted in a separate SQLite file (`automations.db`, next to `DB_PATH`) and managed by `whatsapp-api-engine/src/infrastructure/automation.rs`. A background scheduler ticks every minute to run scheduled messages; incoming messages trigger auto-replies, forwarders, and webhooks. API surface: `GET/POST /api/automations`, `GET/PUT/DELETE /api/automations/{id}`, `POST /api/automations/{id}/trigger`. The GUI is in the Settings panel.

## Config

Env vars: `PORT` (8080), `DB_PATH` (whatsapp.db), `RUST_LOG` (default `info,whatsapp_api_server=debug`) are read by the server and loaded via `dotenvy` from `.env` (gitignored; see `.env.example`). `SKIP_WEB_BUILD=1` is read by `build.rs` only, not dotenvy.