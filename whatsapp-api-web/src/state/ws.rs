use dioxus::prelude::*;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use gloo_timers::future::TimeoutFuture;
use tracing::{error, warn};
use whatsapp_api_types::domain::chat::{Chat, ChatKind, MessageRef};
use whatsapp_api_types::domain::ws_event::WsEvent;

use super::app_state::{API_BASE_URL, CHATS, CONNECTION, LAST_ERROR, MESSAGES, QR_IMAGE, UiConnection};
use crate::api;

const RECONNECT_DELAY_MS: u32 = 1500;

/// Open the WebSocket and keep it alive forever, reconnecting on failure.
pub fn spawn_ws() {
    wasm_bindgen_futures::spawn_local(async move {
        loop {
            match connect().await {
                Ok((mut sink, mut stream)) => {
                    while let Some(msg) = stream.next().await {
                        match msg {
                            Ok(Message::Text(text)) => dispatch(&text),
                            Ok(Message::Bytes(bytes)) => {
                                if let Ok(text) = String::from_utf8(bytes) {
                                    dispatch(&text);
                                }
                            }
                            Err(e) => {
                                warn!("ws stream error: {e}");
                                break;
                            }
                        }
                    }
                    let _ = sink.close().await;
                }
                Err(e) => {
                    warn!("ws connect failed: {e:?}");
                    CONNECTION.with_mut(|c| *c = UiConnection::Disconnected);
                }
            }
            CONNECTION.with_mut(|c| *c = UiConnection::Reconnecting);
            TimeoutFuture::new(RECONNECT_DELAY_MS).await;
        }
    });
}

fn refresh_chats() {
    wasm_bindgen_futures::spawn_local(async move {
        match api::client::fetch_chats().await {
            Ok(chats) => CHATS.with_mut(|c| *c = chats),
            Err(e) => warn!("failed to fetch chats: {e}"),
        }
    });
}

async fn connect() -> Result<(SplitSink<WebSocket, Message>, SplitStream<WebSocket>), wasm_bindgen::JsError> {
    let ws = WebSocket::open(&ws_url())?;
    Ok(ws.split())
}

fn ws_url() -> String {
    let base = API_BASE_URL();
    let host = if base.is_empty() {
        web_sys::window()
            .map(|w| w.location())
            .map(|l| l.host().unwrap_or_else(|_| "localhost:8080".to_string()))
            .unwrap_or_else(|| "localhost:8080".to_string())
    } else {
        base.trim_end_matches('/')
            .strip_prefix("http://")
            .or_else(|| base.trim_end_matches('/').strip_prefix("https://"))
            .unwrap_or(base.trim_end_matches('/'))
            .to_string()
    };
    format!("ws://{host}/ws")
}

fn dispatch(text: &str) {
    let Ok(event) = serde_json::from_str::<WsEvent>(text) else {
        return;
    };
    match event {
        WsEvent::QrCode { image, .. } => {
            QR_IMAGE.with_mut(|q| *q = Some(image));
            CONNECTION.with_mut(|c| *c = UiConnection::WaitingForQr);
        }
        WsEvent::Connected => {
            CONNECTION.with_mut(|c| *c = UiConnection::Connected);
            refresh_chats();
        }
        WsEvent::ChatsUpdated => {
            refresh_chats();
        }
        WsEvent::Disconnected => {
            CONNECTION.with_mut(|c| *c = UiConnection::Disconnected);
        }
        WsEvent::LoggedOut { .. } => {
            QR_IMAGE.with_mut(|q| *q = None);
            CONNECTION.with_mut(|c| *c = UiConnection::Disconnected);
        }
        WsEvent::Message(msg) => {
            MESSAGES.with_mut(|m| {
                let entry = m.entry(msg.chat.clone()).or_default();
                if let Some(existing) = entry.iter_mut().find(|existing| existing.id == msg.id) {
                    *existing = msg.clone();
                } else {
                    entry.push(msg.clone());
                }
            });
            CHATS.with_mut(|chats| {
                let preview = MessageRef {
                    id: msg.id.clone(),
                    text: msg.text.clone(),
                    from_me: msg.from_me,
                    timestamp_ms: msg.timestamp_ms,
                };
                let kind = ChatKind::from_jid(&msg.chat);
                match chats.iter_mut().find(|c| c.jid == msg.chat) {
                    Some(chat) => {
                        chat.last_message = Some(preview);
                        chat.kind = kind;
                        if !msg.from_me && kind != ChatKind::Group && !msg.sender_name.is_empty() {
                            chat.name = msg.sender_name.clone();
                        }
                    }
                    None => chats.insert(
                        0,
                        Chat {
                            jid: msg.chat.clone(),
                            name: msg.sender_name.clone(),
                            is_group: kind == ChatKind::Group,
                            kind,
                            last_message: Some(preview),
                        },
                    ),
                }
            });
        }
        WsEvent::MessageStatusUpdate { id, chat, status, error } => {
            MESSAGES.with_mut(|m| {
                if let Some(entry) = m.get_mut(&chat)
                    && let Some(msg) = entry.iter_mut().find(|msg| msg.id == id)
                {
                    msg.status = status;
                    msg.error = error;
                }
            });
        }
        WsEvent::Error { message } => {
            error!("server error: {message}");
            LAST_ERROR.with_mut(|e| *e = Some(message));
        }
    }
}