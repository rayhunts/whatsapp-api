use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Lifecycle status of a message. Only meaningful for outgoing messages;
/// incoming messages are defaulted to [`MessageStatus::Delivered`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MessageStatus {
    /// Accepted by the API but not yet handed to WhatsApp.
    #[default]
    Pending,
    /// Accepted by the WhatsApp server (one tick).
    Sent,
    /// Delivered to the recipient (two ticks).
    Delivered,
    /// Read by the recipient (blue ticks).
    Read,
    /// Send failed; see [`Message::error`] for details.
    Failed,
}

/// Direction of a message relative to the linked WhatsApp account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MessageDirection {
    /// Received from a contact or group.
    #[default]
    Incoming,
    /// Sent from this account.
    Outgoing,
}

/// A text message in wire format, identical on the server and the web client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Message {
    pub id: String,
    pub chat: String,
    pub sender: String,
    pub sender_name: String,
    pub from_me: bool,
    pub text: String,
    pub timestamp_ms: i64,
    #[serde(default)]
    pub status: MessageStatus,
    #[serde(default)]
    pub direction: MessageDirection,
    #[serde(default)]
    pub error: Option<String>,
}
