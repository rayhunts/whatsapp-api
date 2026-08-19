use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::{broadcast, watch};
use tracing::{error, info, warn};
use whatsapp_api_errors::{AppError, AppResult};
use whatsapp_api_types::domain::automation::{
    Automation, AutomationConfig, CreateAutomationRequest, UpdateAutomationRequest,
};
use whatsapp_api_types::domain::chat::{Chat, ChatKind, MessageRef};
use whatsapp_api_types::domain::contact::{
    Contact, ContactGroup, CreateContactGroupRequest, CreateContactRequest,
    UpdateContactGroupRequest, UpdateContactRequest,
};
use whatsapp_api_types::domain::message::{Message, MessageDirection, MessageStatus};
use whatsapp_api_types::domain::ws_event::WsEvent;
use whatsapp_rust::prelude::*;
use whatsapp_rust::send::SendOptions;
use whatsapp_rust::types::events::{EventKind, LazyHistorySync, Receipt};
use whatsapp_rust::wacore_binary::JidExt;

use crate::domain::{ConnectionState, QrPayload, WaEngine};

pub mod automation;
pub mod contacts;

const EVENT_BUFFER: usize = 256;
const MAX_MESSAGES_PER_CHAT: usize = 1000;
const RESTART_DELAY: Duration = Duration::from_secs(2);

/// Render a pairing QR payload as an SVG image and base64-encode it into a
/// `data:` URI ready to drop into an `<img>` tag.
fn qr_image_data_uri(code: &str) -> Option<String> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use qrcode::render::svg;
    let qr = qrcode::QrCode::new(code.as_bytes()).ok()?;
    let svg = qr.render::<svg::Color>().min_dimensions(21, 21).build();
    Some(format!("data:image/svg+xml;base64,{}", STANDARD.encode(svg)))
}

#[derive(Default)]
pub struct Registry {
    chats: BTreeMap<String, Chat>,
    messages: BTreeMap<String, VecDeque<Message>>,
}

/// Concrete `WaEngine` backed by a `whatsapp-rust` `Bot` with a SQLite store.
pub struct WhatsappEngine {
    db_path: String,
    registry: Arc<RwLock<Registry>>,
    events: broadcast::Sender<WsEvent>,
    state: watch::Sender<ConnectionState>,
    last_qr: watch::Sender<Option<QrPayload>>,
    client: Arc<Mutex<Option<Arc<Client>>>>,
    automations: Arc<automation::AutomationStore>,
    contacts: Arc<contacts::ContactStore>,
}

impl WhatsappEngine {
    pub fn new(db_path: impl Into<String>) -> Self {
        let (events, _) = broadcast::channel(EVENT_BUFFER);
        let (state, _) = watch::channel(ConnectionState::Disconnected);
        let (last_qr, _) = watch::channel(None);
        let db_path = db_path.into();
        let automations = Arc::new(
            automation::AutomationStore::new(&db_path).unwrap_or_else(|e| {
                panic!("failed to open automation store: {e}")
            }),
        );
        let contacts = Arc::new(
            contacts::ContactStore::new(&db_path).unwrap_or_else(|e| {
                panic!("failed to open contact store: {e}")
            }),
        );
        Self {
            db_path,
            registry: Arc::new(RwLock::new(Registry::default())),
            events,
            state,
            last_qr,
            client: Arc::new(Mutex::new(None)),
            automations,
            contacts,
        }
    }

    async fn run_loop(self: Arc<Self>) {
        loop {
            self.state.send_replace(ConnectionState::Connecting);
            if let Err(e) = self.build_and_run_bot().await {
                error!("bot session failed: {e}");
            } else {
                info!("bot session ended");
            }
            self.state.send_replace(ConnectionState::Disconnected);
            *self.client.lock().unwrap() = None;
            tokio::time::sleep(RESTART_DELAY).await;
        }
    }

    async fn build_and_run_bot(self: &Arc<Self>) -> AppResult<()> {
        let backend = SqliteStore::new(&self.db_path)
            .await
            .map_err(|e| AppError::Engine(e.to_string()))?;

        let bot = Bot::builder()
            .with_backend(backend)
            .on_qr_code({
                let events = self.events.clone();
                let last_qr = self.last_qr.clone();
                move |code, timeout| {
                    let events = events.clone();
                    let last_qr = last_qr.clone();
                    async move {
                        let image = qr_image_data_uri(&code).unwrap_or_default();
                        let payload = QrPayload { code: code.clone(), image: image.clone() };
                        last_qr.send_replace(Some(payload));
                        let _ = events.send(WsEvent::QrCode {
                            code,
                            image,
                            expires_in_secs: timeout.as_secs(),
                        });
                    }
                }
            })
            .on_message({
                let events = self.events.clone();
                let registry = self.registry.clone();
                let contacts = self.contacts.clone();
                move |ctx| {
                    let events = events.clone();
                    let registry = registry.clone();
                    let contacts = contacts.clone();
                    async move {
                        let Some(text) = ctx.message.text_content() else {
                            return;
                        };
                        let sender_name = if ctx.info.push_name.is_empty() {
                            ctx.info.source.sender.user().to_string()
                        } else {
                            ctx.info.push_name.clone()
                        };
                        let sender_name = contacts
                            .contact_name_for_jid(&ctx.info.source.sender.to_string())
                            .unwrap_or(sender_name);
                        let message = Message {
                            id: ctx.info.id.clone(),
                            chat: ctx.info.source.chat.to_string(),
                            sender: ctx.info.source.sender.to_string(),
                            sender_name,
                            from_me: ctx.info.source.is_from_me,
                            text: text.to_string(),
                            timestamp_ms: ctx.info.timestamp.timestamp_millis(),
                            status: MessageStatus::Delivered,
                            direction: if ctx.info.source.is_from_me {
                                MessageDirection::Outgoing
                            } else {
                                MessageDirection::Incoming
                            },
                            error: None,
                        };
                        record_message(&registry, message.clone());
                        let _ = events.send(WsEvent::Message(message));
                    }
                }
            })
            .on_event_for(&[EventKind::HistorySync], {
                let events = self.events.clone();
                let registry = self.registry.clone();
                let contacts = self.contacts.clone();
                move |event, _client| {
                    let events = events.clone();
                    let registry = registry.clone();
                    let contacts = contacts.clone();
                    async move {
                        let Event::HistorySync(lazy) = &*event else {
                            return;
                        };
                        if seed_history(&registry, &contacts, lazy).await {
                            let _ = events.send(WsEvent::ChatsUpdated);
                        }
                    }
                }
            })
            .on_event_for(&[EventKind::Receipt], {
                let events = self.events.clone();
                let registry = self.registry.clone();
                move |event, _client| {
                    let events = events.clone();
                    let registry = registry.clone();
                    async move {
                        let Event::Receipt(receipt) = &*event else {
                            return;
                        };
                        handle_receipt(&registry, &events, receipt);
                    }
                }
            })
            .on_connected({
                let events = self.events.clone();
                let state = self.state.clone();
                move |client| {
                    let events = events.clone();
                    let state = state.clone();
                    async move {
                        state.send_replace(ConnectionState::Connected);
                        let _ = client.presence().set_available().await;
                        let _ = events.send(WsEvent::Connected);
                    }
                }
            })
            .on_logged_out({
                let events = self.events.clone();
                let state = self.state.clone();
                move |logged_out| {
                    let events = events.clone();
                    let state = state.clone();
                    async move {
                        state.send_replace(ConnectionState::Disconnected);
                        let _ = events.send(WsEvent::LoggedOut {
                            reason: format!("{:?}", logged_out.reason),
                        });
                    }
                }
            })
            .build()
            .await
            .map_err(|e| AppError::Engine(e.to_string()))?;

        *self.client.lock().unwrap() = Some(bot.client());
        bot.run().await;
        Ok(())
    }
}

#[async_trait::async_trait]
impl WaEngine for WhatsappEngine {
    async fn start(self: Arc<Self>) {
        let scheduler = Arc::new(automation::AutomationScheduler::new(
            self.automations.clone(),
            self.client.clone(),
            self.registry.clone(),
            self.events.clone(),
        ));
        scheduler.start();
        tokio::spawn(self.run_loop());
    }

    async fn send_text(&self, chat: &str, text: &str) -> AppResult<Message> {
        let jid: Jid = chat.parse().map_err(|_| AppError::InvalidJid(chat.to_string()))?;
        let client = self
            .client
            .lock()
            .unwrap()
            .clone()
            .ok_or(AppError::NotConnected)?;

        let id = uuid::Uuid::new_v4().to_string();
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let sender = client
            .pn()
            .map(|j| j.to_string())
            .unwrap_or_else(|| "self".into());
        let message = Message {
            id: id.clone(),
            chat: chat.to_string(),
            sender,
            sender_name: "Me".into(),
            from_me: true,
            text: text.to_string(),
            timestamp_ms: now_ms,
            status: MessageStatus::Pending,
            direction: MessageDirection::Outgoing,
            error: None,
        };
        record_message(&self.registry, message.clone());
        let _ = self.events.send(WsEvent::Message(message.clone()));

        // Complete the send in the background so the API can return the pending
        // message immediately. Success/failure is delivered over the WebSocket.
        let registry = self.registry.clone();
        let events = self.events.clone();
        let text = text.to_string();
        let chat = chat.to_string();
        tokio::spawn(async move {
            let options = SendOptions::default().with_message_id(&id);
            match client
                .send_message_with_options(jid, wa::Message::text(&text), options)
                .await
            {
                Ok(_) => {
                    update_outgoing_status(&registry, &id, &chat, MessageStatus::Sent, None);
                    let _ = events.send(WsEvent::MessageStatusUpdate {
                        id,
                        chat,
                        status: MessageStatus::Sent,
                        error: None,
                    });
                }
                Err(e) => {
                    let err = e.to_string();
                    update_outgoing_status(
                        &registry,
                        &id,
                        &chat,
                        MessageStatus::Failed,
                        Some(err.clone()),
                    );
                    let _ = events.send(WsEvent::MessageStatusUpdate {
                        id,
                        chat,
                        status: MessageStatus::Failed,
                        error: Some(err),
                    });
                }
            }
        });

        Ok(message)
    }

    async fn chats(&self) -> Vec<Chat> {
        let reg = self.registry.read().unwrap();
        let mut chats: Vec<Chat> = reg.chats.values().cloned().collect();
        chats.sort_by_key(|c| c.last_message.as_ref().map(|m| m.timestamp_ms).unwrap_or(0));
        chats.reverse();
        chats
    }

    async fn messages(&self, chat: &str) -> Vec<Message> {
        self.registry
            .read()
            .unwrap()
            .messages
            .get(chat)
            .map(|m| m.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn state(&self) -> ConnectionState {
        *self.state.borrow()
    }

    fn subscribe(&self) -> broadcast::Receiver<WsEvent> {
        self.events.subscribe()
    }

    fn last_qr(&self) -> Option<QrPayload> {
        self.last_qr.borrow().clone()
    }

    async fn create_automation(&self, req: CreateAutomationRequest) -> AppResult<Automation> {
        self.automations.create(req)
    }

    async fn list_automations(&self) -> AppResult<Vec<Automation>> {
        self.automations.list()
    }

    async fn get_automation(&self, id: &str) -> AppResult<Automation> {
        self.automations.get(id)
    }

    async fn update_automation(&self, id: &str, req: UpdateAutomationRequest) -> AppResult<Automation> {
        self.automations.update(id, req)
    }

    async fn delete_automation(&self, id: &str) -> AppResult<()> {
        self.automations.delete(id)
    }

    async fn trigger_automation(&self, id: &str) -> AppResult<()> {
        let automation = self.automations.get(id)?;
        if let AutomationConfig::ScheduledMessage { to, text, .. } = automation.config {
            for jid in to {
                self.send_text(&jid, &text).await?;
            }
            self.automations.record_run(&automation.id)?;
        } else {
            return Err(AppError::Automation(
                "manual trigger is only supported for scheduled messages".into(),
            ));
        }
        Ok(())
    }

    async fn ensure_chat(&self, jid: &str, name: Option<String>) {
        let kind = ChatKind::from_jid(jid);
        let is_group = kind == ChatKind::Group;
        let default_name = name.clone().unwrap_or_default();
        let mut reg = self.registry.write().unwrap();
        let chat = reg.chats.entry(jid.to_string()).or_insert_with(|| Chat {
            jid: jid.to_string(),
            name: default_name,
            is_group,
            kind,
            last_message: None,
        });
        chat.kind = kind;
        if chat.name.is_empty()
            && let Some(n) = name
        {
            chat.name = n;
        }
    }

    async fn create_contact(&self, req: CreateContactRequest) -> AppResult<Contact> {
        self.contacts.create_contact(req)
    }

    async fn list_contacts(&self) -> AppResult<Vec<Contact>> {
        self.contacts.list_contacts()
    }

    async fn get_contact(&self, id: &str) -> AppResult<Contact> {
        self.contacts.get_contact(id)
    }

    async fn update_contact(&self, id: &str, req: UpdateContactRequest) -> AppResult<Contact> {
        self.contacts.update_contact(id, req)
    }

    async fn delete_contact(&self, id: &str) -> AppResult<()> {
        self.contacts.delete_contact(id)
    }

    async fn create_contact_group(
        &self,
        req: CreateContactGroupRequest,
    ) -> AppResult<ContactGroup> {
        self.contacts.create_group(req)
    }

    async fn list_contact_groups(&self) -> AppResult<Vec<ContactGroup>> {
        self.contacts.list_groups()
    }

    async fn get_contact_group(&self, id: &str) -> AppResult<ContactGroup> {
        self.contacts.get_group(id)
    }

    async fn update_contact_group(
        &self,
        id: &str,
        req: UpdateContactGroupRequest,
    ) -> AppResult<ContactGroup> {
        self.contacts.update_group(id, req)
    }

    async fn delete_contact_group(&self, id: &str) -> AppResult<()> {
        self.contacts.delete_group(id)
    }

    async fn broadcast(
        &self,
        group_ids: &[String],
        to: &[String],
        message: &str,
    ) -> AppResult<usize> {
        if message.trim().is_empty() {
            return Err(AppError::InvalidInput("message is empty".into()));
        }
        let targets = self.contacts.resolve_broadcast_targets(group_ids, to)?;
        if targets.is_empty() {
            return Err(AppError::InvalidInput("no valid broadcast targets".into()));
        }
        for jid in &targets {
            self.ensure_chat(jid, None).await;
        }
        let total = targets.len();
        let text = message.to_string();
        let engine = self.clone();
        tokio::spawn(async move {
            for jid in targets {
                let _ = engine.send_text(&jid, &text).await;
            }
        });
        Ok(total)
    }
}

impl Clone for WhatsappEngine {
    fn clone(&self) -> Self {
        Self {
            db_path: self.db_path.clone(),
            registry: self.registry.clone(),
            events: self.events.clone(),
            state: self.state.clone(),
            last_qr: self.last_qr.clone(),
            client: self.client.clone(),
            automations: self.automations.clone(),
            contacts: self.contacts.clone(),
        }
    }
}

/// Send plain text to a JID, tracking the message in the registry from
/// `pending` through `sent`/`failed` and broadcasting status updates.
async fn send_text_tracked(
    client: &Client,
    registry: &Arc<RwLock<Registry>>,
    events: &broadcast::Sender<WsEvent>,
    jid: &Jid,
    chat: &str,
    text: &str,
) -> AppResult<Message> {
    let id = uuid::Uuid::new_v4().to_string();
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let sender = client
        .pn()
        .map(|j| j.to_string())
        .unwrap_or_else(|| "self".into());
    let mut message = Message {
        id: id.clone(),
        chat: chat.to_string(),
        sender,
        sender_name: "Me".into(),
        from_me: true,
        text: text.to_string(),
        timestamp_ms: now_ms,
        status: MessageStatus::Pending,
        direction: MessageDirection::Outgoing,
        error: None,
    };
    record_message(registry, message.clone());
    let _ = events.send(WsEvent::Message(message.clone()));

    let options = SendOptions::default().with_message_id(&id);
    match client
        .send_message_with_options(jid.clone(), wa::Message::text(text), options)
        .await
    {
        Ok(_) => {
            update_outgoing_status(registry, &id, chat, MessageStatus::Sent, None);
            message.status = MessageStatus::Sent;
            let _ = events.send(WsEvent::MessageStatusUpdate {
                id,
                chat: chat.to_string(),
                status: MessageStatus::Sent,
                error: None,
            });
            Ok(message)
        }
        Err(e) => {
            let err = e.to_string();
            update_outgoing_status(registry, &id, chat, MessageStatus::Failed, Some(err.clone()));
            message.status = MessageStatus::Failed;
            message.error = Some(err.clone());
            let _ = events.send(WsEvent::MessageStatusUpdate {
                id,
                chat: chat.to_string(),
                status: MessageStatus::Failed,
                error: Some(err.clone()),
            });
            Err(AppError::Send(err))
        }
    }
}

/// Update the status of an outgoing message in the registry by id.
fn update_outgoing_status(
    registry: &Arc<RwLock<Registry>>,
    id: &str,
    chat: &str,
    status: MessageStatus,
    error: Option<String>,
) -> bool {
    let mut reg = registry.write().unwrap();
    let Some(queue) = reg.messages.get_mut(chat) else {
        return false;
    };
    let Some(msg) = queue.iter_mut().find(|m| m.id == id) else {
        return false;
    };
    if msg.direction != MessageDirection::Outgoing {
        return false;
    }
    msg.status = status;
    msg.error = error;
    true
}

/// Process a receipt event and promote outgoing message statuses.
fn handle_receipt(
    registry: &Arc<RwLock<Registry>>,
    events: &broadcast::Sender<WsEvent>,
    receipt: &Receipt,
) {
    use whatsapp_rust::types::presence::ReceiptType;
    let status = match receipt.r#type {
        ReceiptType::Delivered => Some(MessageStatus::Delivered),
        ReceiptType::Read => Some(MessageStatus::Read),
        _ => None,
    };
    let Some(status) = status else {
        return;
    };
    let chat = receipt.source.chat.to_string();
    for mid in &receipt.message_ids {
        if update_outgoing_status(registry, mid, &chat, status, None) {
            let _ = events.send(WsEvent::MessageStatusUpdate {
                id: mid.clone(),
                chat: chat.clone(),
                status,
                error: None,
            });
        }
    }
}

/// Fold a WhatsApp history-sync chunk into the registry, then return whether
/// anything was recorded (so the caller can announce `ChatsUpdated`).
async fn seed_history(
    registry: &Arc<RwLock<Registry>>,
    contacts: &Arc<contacts::ContactStore>,
    lazy: &LazyHistorySync,
) -> bool {
    let registry = registry.clone();
    let contacts = contacts.clone();
    let lazy = lazy.clone();
    tokio::task::spawn_blocking(move || {
        let mut stream = lazy.stream();
        let mut seeded = false;
        loop {
        match stream.next_conversation() {
            Ok(Some(conversation)) => {
                seed_conversation(&registry, &contacts, conversation);
                seeded = true;
            }
                Ok(None) => break,
                Err(e) => {
                    warn!("history sync chunk failed to decode: {e}");
                    break;
                }
            }
        }
        seeded
    })
    .await
    .unwrap_or(false)
}

fn seed_conversation(
    registry: &Arc<RwLock<Registry>>,
    contacts: &Arc<contacts::ContactStore>,
    conversation: wa::Conversation,
) {
    let chat_jid = conversation.id;
    let kind = ChatKind::from_jid(&chat_jid);
    let is_group = kind == ChatKind::Group;
    let chat_name = conversation.name.unwrap_or_default();
    {
        let mut reg = registry.write().unwrap();
        let chat = reg
            .chats
            .entry(chat_jid.clone())
            .or_insert_with(|| Chat {
                jid: chat_jid.clone(),
                name: chat_name.clone(),
                is_group,
                kind,
                last_message: None,
            });
        if chat.name.is_empty() {
            chat.name = chat_name;
        }
        chat.kind = kind;
    }

    let mut messages = Vec::new();
    for history_msg in conversation.messages {
        let Some(info) = history_msg.message.as_option() else {
            continue;
        };
        let Some(key) = info.key.as_option() else {
            continue;
        };
        let Some(text) = info.message.as_option().and_then(MessageExt::text_content) else {
            continue;
        };
        let sender = if is_group {
            key.participant.as_deref()
        } else {
            key.remote_jid.as_deref()
        };
        let Some(sender) = sender else {
            continue;
        };
        let sender_name = contacts
            .contact_name_for_jid(sender)
            .or_else(|| {
                info.push_name
                    .as_deref()
                    .filter(|n| !n.is_empty())
                    .map(|n| n.to_string())
            })
            .unwrap_or_else(|| sender.split('@').next().unwrap_or(sender).to_string());
        let from_me = key.from_me.unwrap_or(false);
        messages.push(Message {
            id: key.id.clone().unwrap_or_default(),
            chat: chat_jid.clone(),
            sender: sender.to_string(),
            sender_name,
            from_me,
            text: text.to_string(),
            timestamp_ms: info.message_timestamp.unwrap_or(0).saturating_mul(1000) as i64,
            status: if from_me {
                MessageStatus::Sent
            } else {
                MessageStatus::Delivered
            },
            direction: if from_me {
                MessageDirection::Outgoing
            } else {
                MessageDirection::Incoming
            },
            error: None,
        });
    }
    messages.sort_by_key(|m| m.timestamp_ms);
    for message in messages {
        record_message(registry, message);
    }
}

fn record_message(registry: &Arc<RwLock<Registry>>, msg: Message) {
    let mut reg = registry.write().unwrap();

    let queue = reg.messages.entry(msg.chat.clone()).or_default();
    queue.push_back(msg.clone());
    while queue.len() > MAX_MESSAGES_PER_CHAT {
        queue.pop_front();
    }

    let kind = ChatKind::from_jid(&msg.chat);
    let is_group = kind == ChatKind::Group;
    let chat = reg
        .chats
        .entry(msg.chat.clone())
        .or_insert_with(|| Chat {
            jid: msg.chat.clone(),
            name: msg.sender_name.clone(),
            is_group,
            kind,
            last_message: None,
        });
    chat.kind = kind;
    if !msg.from_me && !msg.sender_name.is_empty() && !is_group && chat.name.is_empty() {
        chat.name = msg.sender_name.clone();
    }
    chat.last_message = Some(MessageRef {
        id: msg.id,
        text: msg.text,
        from_me: msg.from_me,
        timestamp_ms: msg.timestamp_ms,
    });
}