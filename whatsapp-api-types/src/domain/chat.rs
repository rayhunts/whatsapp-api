use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Classification of a WhatsApp conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChatKind {
    /// One-to-one conversation with a contact.
    Private,
    /// Multi-participant WhatsApp group (`@g.us`).
    Group,
    /// Broadcast list (`@broadcast`).
    Broadcast,
    /// Status updates (`@status` or `@broadcast` status JIDs).
    Status,
    /// WhatsApp newsletter/channel (`@newsletter`).
    Newsletter,
    /// Any JID that does not match the known patterns.
    #[default]
    Unknown,
}

impl ChatKind {
    /// Derive a chat kind from a raw WhatsApp JID.
    pub fn from_jid(jid: &str) -> Self {
        if jid.ends_with("@g.us") {
            Self::Group
        } else if jid.ends_with("@newsletter") {
            Self::Newsletter
        } else if jid == "status@broadcast" || jid.ends_with("@status") {
            Self::Status
        } else if jid.ends_with("@broadcast") {
            Self::Broadcast
        } else if jid.ends_with("@s.whatsapp.net")
            || jid.contains("@")
                && !jid.ends_with("@g.us")
                && !jid.ends_with("@broadcast")
                && !jid.ends_with("@newsletter")
        {
            Self::Private
        } else {
            Self::Unknown
        }
    }
}

/// A chat (1:1 or group) tracked by the engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Chat {
    pub jid: String,
    pub name: String,
    pub is_group: bool,
    #[serde(default)]
    pub kind: ChatKind,
    pub last_message: Option<MessageRef>,
}

/// Short preview of the most recent message in a chat.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct MessageRef {
    pub id: String,
    pub text: String,
    pub from_me: bool,
    pub timestamp_ms: i64,
}