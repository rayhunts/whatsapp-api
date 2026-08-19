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

use crate::domain::{ConnectionState, QrPayload, WaEngine};

/// Application-layer façade around a `WaEngine`. Owns the engine instance,
/// spawns its session loop, and is the single entry point used by the server.
pub struct EngineService {
    engine: Arc<dyn WaEngine>,
}

impl EngineService {
    pub fn new(engine: Arc<dyn WaEngine>) -> Self {
        Self { engine }
    }

    pub async fn start(&self) {
        self.engine.clone().start().await;
    }

    pub async fn send_text(&self, chat: &str, text: &str) -> AppResult<Message> {
        self.engine.send_text(chat, text).await
    }

    pub async fn chats(&self) -> Vec<Chat> {
        self.engine.chats().await
    }

    pub async fn messages(&self, chat: &str) -> Vec<Message> {
        self.engine.messages(chat).await
    }

    pub fn state(&self) -> ConnectionState {
        self.engine.state()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WsEvent> {
        self.engine.subscribe()
    }

    pub fn last_qr(&self) -> Option<QrPayload> {
        self.engine.last_qr()
    }

    pub async fn create_automation(&self, req: CreateAutomationRequest) -> AppResult<Automation> {
        self.engine.create_automation(req).await
    }

    pub async fn list_automations(&self) -> AppResult<Vec<Automation>> {
        self.engine.list_automations().await
    }

    pub async fn get_automation(&self, id: &str) -> AppResult<Automation> {
        self.engine.get_automation(id).await
    }

    pub async fn update_automation(&self, id: &str, req: UpdateAutomationRequest) -> AppResult<Automation> {
        self.engine.update_automation(id, req).await
    }

    pub async fn delete_automation(&self, id: &str) -> AppResult<()> {
        self.engine.delete_automation(id).await
    }

    pub async fn trigger_automation(&self, id: &str) -> AppResult<()> {
        self.engine.trigger_automation(id).await
    }

    pub async fn ensure_chat(&self, jid: &str, name: Option<String>) {
        self.engine.ensure_chat(jid, name).await;
    }

    pub async fn create_contact(&self, req: CreateContactRequest) -> AppResult<Contact> {
        self.engine.create_contact(req).await
    }

    pub async fn list_contacts(&self) -> AppResult<Vec<Contact>> {
        self.engine.list_contacts().await
    }

    pub async fn get_contact(&self, id: &str) -> AppResult<Contact> {
        self.engine.get_contact(id).await
    }

    pub async fn update_contact(&self, id: &str, req: UpdateContactRequest) -> AppResult<Contact> {
        self.engine.update_contact(id, req).await
    }

    pub async fn delete_contact(&self, id: &str) -> AppResult<()> {
        self.engine.delete_contact(id).await
    }

    pub async fn create_contact_group(
        &self,
        req: CreateContactGroupRequest,
    ) -> AppResult<ContactGroup> {
        self.engine.create_contact_group(req).await
    }

    pub async fn list_contact_groups(&self) -> AppResult<Vec<ContactGroup>> {
        self.engine.list_contact_groups().await
    }

    pub async fn get_contact_group(&self, id: &str) -> AppResult<ContactGroup> {
        self.engine.get_contact_group(id).await
    }

    pub async fn update_contact_group(
        &self,
        id: &str,
        req: UpdateContactGroupRequest,
    ) -> AppResult<ContactGroup> {
        self.engine.update_contact_group(id, req).await
    }

    pub async fn delete_contact_group(&self, id: &str) -> AppResult<()> {
        self.engine.delete_contact_group(id).await
    }

    pub async fn broadcast(
        &self,
        group_ids: &[String],
        to: &[String],
        message: &str,
    ) -> AppResult<usize> {
        self.engine.broadcast(group_ids, to, message).await
    }
}