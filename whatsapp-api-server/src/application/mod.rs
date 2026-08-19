use std::sync::Arc;

use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use include_dir::{include_dir, Dir};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use whatsapp_api_engine::application::EngineService;

use crate::infrastructure::http::openapi::ApiDoc;
use crate::infrastructure::http::{
    chat_messages, chats, create_automation, delete_automation, ensure_chat, get_automation,
    health, list_automations, resolve_phone, send_message, trigger_automation, update_automation,
};
use crate::infrastructure::ws::WsHub;

pub struct AppState {
    pub service: EngineService,
    pub hub: WsHub,
}

/// Web frontend produced by `build.rs` (via `dx build`) and embedded at compile time.
static WEB: Dir<'_> = include_dir!("$OUT_DIR/web");

pub fn build_app(service: EngineService) -> Router {
    let hub = WsHub::new(service.subscribe());
    let state = Arc::new(AppState { service, hub });

    Router::new()
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
        .route("/api/health", get(health))
        .route("/api/chats", get(chats))
        .route("/api/chats/ensure", post(ensure_chat))
        .route("/api/chats/resolve-phone", post(resolve_phone))
        .route(
            "/api/chats/{jid}/messages",
            get(chat_messages).post(send_message),
        )
        .route("/api/automations", get(list_automations).post(create_automation))
        .route(
            "/api/automations/{id}",
            get(get_automation).put(update_automation).delete(delete_automation),
        )
        .route("/api/automations/{id}/trigger", post(trigger_automation))
        .route("/ws", get(crate::infrastructure::ws::ws_handler))
        .route("/openapi.json", get(openapi_json))
        .fallback(web_assets)
        .with_state(state)
}

/// Machine-readable OpenAPI spec, e.g. for code generators.
async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// Serves the embedded frontend; falls back to `index.html` so client-side
/// routes keep working, and to a placeholder page when no frontend was built.
async fn web_assets(request: Request) -> Response {
    let path = request.uri().path().trim_start_matches('/');
    if let Some(resp) = asset_response(path) {
        return resp;
    }
    asset_response("index.html").unwrap_or_else(web_not_built)
}

fn asset_response(path: &str) -> Option<Response> {
    let file = WEB.get_file(path)?;
    let mime = mime_for(path);
    ([(header::CONTENT_TYPE, mime)], file.contents()).into_response().into()
}

fn mime_for(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript",
        "css" => "text/css; charset=utf-8",
        "wasm" => "application/wasm",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn web_not_built() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        "<h1>whatsapp-api-server</h1><p>The web frontend is not built.</p>\
         <p>Install dioxus-cli and rebuild: <code>cargo install dioxus-cli</code></p>",
    )
        .into_response()
}