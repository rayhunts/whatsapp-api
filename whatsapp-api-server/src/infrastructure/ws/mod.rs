use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tracing::{debug, warn};
use whatsapp_api_engine::domain::ConnectionState;
use whatsapp_api_types::domain::ws_event::{WsEvent, WsRequest};

use crate::application::AppState;

const HUB_BUFFER: usize = 256;

/// Fan-out hub for `WsEvent`s. Drains the engine's event stream and
/// re-broadcasts it to every connected WebSocket client.
pub struct WsHub {
    tx: broadcast::Sender<WsEvent>,
}

impl WsHub {
    pub fn new(mut engine_rx: broadcast::Receiver<WsEvent>) -> Self {
        let (tx, _) = broadcast::channel(HUB_BUFFER);
        let hub_tx = tx.clone();
        tokio::spawn(async move {
            loop {
                match engine_rx.recv().await {
                    Ok(event) => {
                        let _ = hub_tx.send(event);
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("hub lagged behind engine, skipped {skipped} events");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WsEvent> {
        self.tx.subscribe()
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut events = state.hub.subscribe();

    // Snapshot: tell the freshly connected client where things stand.
    if state.service.state() == ConnectionState::Connected {
        let _ = send_json(&mut sender, &WsEvent::Connected).await;
    } else if let Some(qr) = state.service.last_qr() {
        let _ = send_json(
            &mut sender,
            &WsEvent::QrCode {
                code: qr.code,
                image: qr.image,
                expires_in_secs: 60,
            },
        )
        .await;
    }

    loop {
        tokio::select! {
            event = events.recv() => {
                match event {
                    Ok(event) => {
                        if send_json(&mut sender, &event).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("ws client lagged, skipped {skipped} events");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<WsRequest>(&text) {
                            Ok(WsRequest::SendMessage { chat, text }) => {
                                if let Err(e) = state.service.send_text(&chat, &text).await {
                                    let _ = send_json(
                                        &mut sender,
                                        &WsEvent::Error { message: e.to_string() },
                                    )
                                    .await;
                                }
                            }
                            Err(e) => debug!("ignoring unparsable ws frame: {e}"),
                        }
                    }
                    Some(Err(e)) => {
                        debug!("ws client error: {e}");
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
        }
    }
}

async fn send_json<S, E>(sender: &mut S, event: &WsEvent) -> Result<(), E>
where
    S: futures::Sink<Message, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    let text = serde_json::to_string(event).unwrap_or_else(|e| {
        format!("{{\"type\":\"error\",\"message\":\"serialization failed: {e}\"}}")
    });
    sender.send(Message::Text(text.into())).await
}