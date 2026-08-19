use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::message::{Message, MessageStatus};

/// Server → client frames pushed over the WebSocket.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    QrCode {
        code: String,
        image: String,
        expires_in_secs: u64,
    },
    Connected,
    /// History sync has been folded into the engine's registry; clients
    /// should re-fetch the chat list.
    ChatsUpdated,
    Disconnected,
    LoggedOut { reason: String },
    Message(Message),
    /// A previously sent message changed status (sent / delivered / read / failed).
    MessageStatusUpdate {
        id: String,
        chat: String,
        status: MessageStatus,
        error: Option<String>,
    },
    Error { message: String },
}

/// Client → server frames accepted over the WebSocket.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsRequest {
    SendMessage { chat: String, text: String },
}