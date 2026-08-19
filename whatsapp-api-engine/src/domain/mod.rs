use std::sync::Arc;

use tokio::sync::broadcast;
use whatsapp_api_errors::AppResult;
use whatsapp_api_types::domain::automation::{
    Automation, CreateAutomationRequest, UpdateAutomationRequest,
};
use whatsapp_api_types::domain::chat::Chat;
use whatsapp_api_types::domain::contact::{
    Contact, ContactGroup, CreateContactGroupRequest, CreateContactRequest,
    UpdateContactGroupRequest, UpdateContactRequest,
};
use whatsapp_api_types::domain::message::Message;
use whatsapp_api_types::domain::ws_event::WsEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

/// A pairing QR code: the raw payload and a `data:` URI of its rendered image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrPayload {
    pub code: String,
    pub image: String,
}

/// Port the server talks to. The concrete implementation lives in
/// `infrastructure` and is the only place that knows about `whatsapp-rust`.
#[async_trait::async_trait]
pub trait WaEngine: Send + Sync + 'static {
    /// Spawn the bot session loop in the background. Returns immediately.
    async fn start(self: Arc<Self>);
    async fn send_text(&self, chat: &str, text: &str) -> AppResult<Message>;
    async fn chats(&self) -> Vec<Chat>;
    async fn messages(&self, chat: &str) -> Vec<Message>;
    fn state(&self) -> ConnectionState;
    fn subscribe(&self) -> broadcast::Receiver<WsEvent>;
    fn last_qr(&self) -> Option<QrPayload>;

    async fn create_automation(&self, req: CreateAutomationRequest) -> AppResult<Automation>;
    async fn list_automations(&self) -> AppResult<Vec<Automation>>;
    async fn get_automation(&self, id: &str) -> AppResult<Automation>;
    async fn update_automation(&self, id: &str, req: UpdateAutomationRequest) -> AppResult<Automation>;
    async fn delete_automation(&self, id: &str) -> AppResult<()>;
    async fn trigger_automation(&self, id: &str) -> AppResult<()>;

    /// Make sure a chat entry exists in the registry, creating it if necessary.
    async fn ensure_chat(&self, jid: &str, name: Option<String>);

    async fn create_contact(&self, req: CreateContactRequest) -> AppResult<Contact>;
    async fn list_contacts(&self) -> AppResult<Vec<Contact>>;
    async fn get_contact(&self, id: &str) -> AppResult<Contact>;
    async fn update_contact(&self, id: &str, req: UpdateContactRequest) -> AppResult<Contact>;
    async fn delete_contact(&self, id: &str) -> AppResult<()>;

    async fn create_contact_group(&self, req: CreateContactGroupRequest) -> AppResult<ContactGroup>;
    async fn list_contact_groups(&self) -> AppResult<Vec<ContactGroup>>;
    async fn get_contact_group(&self, id: &str) -> AppResult<ContactGroup>;
    async fn update_contact_group(
        &self,
        id: &str,
        req: UpdateContactGroupRequest,
    ) -> AppResult<ContactGroup>;
    async fn delete_contact_group(&self, id: &str) -> AppResult<()>;

    /// Broadcast a message to the resolved targets. Returns the number of queued messages.
    async fn broadcast(&self, group_ids: &[String], to: &[String], message: &str) -> AppResult<usize>;
}