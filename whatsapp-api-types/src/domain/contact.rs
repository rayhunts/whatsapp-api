use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A saved contact in the local address book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub email: Option<String>,
    pub notes: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// A local group or label used to organize contacts for broadcasting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ContactGroup {
    pub id: String,
    pub name: String,
    pub contact_ids: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Request body for creating a contact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CreateContactRequest {
    pub name: String,
    pub phone: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Request body for updating a contact.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateContactRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Request body for creating a contact group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CreateContactGroupRequest {
    pub name: String,
    #[serde(default)]
    pub contact_ids: Vec<String>,
}

/// Request body for updating a contact group.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateContactGroupRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact_ids: Option<Vec<String>>,
}

/// Request body for broadcasting a message to contact groups.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BroadcastRequest {
    #[serde(default)]
    pub group_ids: Vec<String>,
    #[serde(default)]
    pub to: Vec<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay_ms: Option<BroadcastDelay>,
}

/// Optional random delay between broadcast messages.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BroadcastDelay {
    pub min: u64,
    pub max: u64,
}

/// Response returned after a broadcast has been queued.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BroadcastResponse {
    pub broadcast_id: String,
    pub total: usize,
    pub status: String,
}
