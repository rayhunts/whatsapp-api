# whatsapp-api

A WhatsApp Web API: Rust + Tokio backend built on [`whatsapp-rust`](https://crates.io/crates/whatsapp-rust) with a [Dioxus](https://dioxuslabs.com) (WASM) web client.

The project structure mimics [stynx-code](https://github.com/maulanasdqn/stynx-code): a multi-crate Cargo workspace with Clean Architecture (`domain/`, `application/`, `infrastructure/`) inside each crate.

## Workspace layout

| Crate | Role |
| --- | --- |
| `whatsapp-api-errors` | Shared `AppError` type (thiserror) |
| `whatsapp-api-types` | Serde DTOs (`Chat`, `Message`, `WsEvent`) shared by server and web |
| `whatsapp-api-engine` | Wraps the `whatsapp-rust` `Bot` behind a `WaEngine` trait + tokio event bus |
| `whatsapp-api-server` | Axum HTTP + WebSocket server (binary) |
| `whatsapp-api-web` | Dioxus 0.6 WASM web client |

```
Browser (Dioxus WASM) ── REST + WebSocket ──▶ whatsapp-api-server (Axum/tokio)
                                                  │
                                     whatsapp-api-engine (WhatsappEngine)
                                                  │
                                     whatsapp-rust Bot (SqliteStore, tokio transport)
                                                  │
                                            WhatsApp Web
```

## Quick start

Requirements: Rust (with the `wasm32-unknown-unknown` target) and
[dioxus-cli](https://github.com/DioxusLabs/dioxus) (`dx`):

```bash
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --version "~0.6"   # once
```

Run everything (WASM frontend + server) with a single command:

```bash
cp .env.example .env            # optional
cargo dev
```

Open `http://localhost:8080`, scan the QR code with
WhatsApp (Settings → Linked devices → Link a device), and start messaging.

The frontend is compiled by the server's `build.rs` (via `dx build`) and embedded
into the binary — no separate build step, and edits to `whatsapp-api-web/src`
are picked up on the next `cargo dev`.

## One-binary release

```bash
cargo build --release
```

This produces a single self-contained static binary with the frontend embedded:

```bash
./target/release/whatsapp-api-server            # → http://0.0.0.0:8080
```

Copy it to any machine and run it — it needs nothing but the env vars below.

## Configuration

Environment variables (see `.env.example`): `PORT`, `DB_PATH`, `RUST_LOG`.
The session store is a SQLite database — pairing survives restarts.

| Variable | Default | Description |
| --- | --- | --- |
| `PORT` | `8080` | HTTP listen port |
| `DB_PATH` | `whatsapp.db` | SQLite path for the WhatsApp session store |
| `RUST_LOG` | `info,whatsapp_api_server=debug` | Log level |
| `SKIP_WEB_BUILD` | unset | `1` to skip building/embedding the frontend (server-only builds) |

## API

| Method | Route | Description |
| --- | --- | --- |
| `GET` | `/api/health` | Server + connection state |
| `GET` | `/api/chats` | Chat list |
| `GET` | `/api/chats/{jid}/messages` | Message history for a chat |
| `POST` | `/api/chats/{jid}/messages` | `{ "text": "..." }` — send a text message |
| `WS` | `/ws` | Server pushes `QrCode` / `Connected` / `Disconnected` / `LoggedOut` / `Message` frames; client may send `SendMessage` frames |
| `GET` | `/docs` | Interactive API docs (Scalar UI) |
| `GET` | `/openapi.json` | Machine-readable OpenAPI 3.1 spec |

The docs are generated from the handlers at runtime — visit `/docs` for a
browser UI or consume `/openapi.json` directly.

## License

MIT