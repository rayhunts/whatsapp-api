use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use cron::Schedule as CronSchedule;
use regex::Regex;
use rusqlite::{params, Connection};
use tokio::sync::broadcast;
use tokio::time::interval;
use tracing::{error, info, warn};
use whatsapp_api_errors::{AppError, AppResult};
use whatsapp_api_types::domain::automation::{
    Automation, AutomationConfig, CreateAutomationRequest, MatchType, UpdateAutomationRequest,
};
use whatsapp_api_types::domain::message::Message;
use whatsapp_api_types::domain::ws_event::WsEvent;
use whatsapp_rust::prelude::*;

use super::Registry;

const AUTOMATION_DB_FILE: &str = "automations.db";
const TICK_INTERVAL_SECS: u64 = 60;

/// SQLite-backed store for automation rules.
pub struct AutomationStore {
    conn: Mutex<Connection>,
}

impl AutomationStore {
    pub fn new(db_path: impl Into<String>) -> AppResult<Self> {
        let base: std::path::PathBuf = db_path.into().into();
        let dir = base.parent().unwrap_or_else(|| std::path::Path::new("."));
        let db_file = dir.join(AUTOMATION_DB_FILE);
        let conn = Connection::open(&db_file)
            .map_err(|e| AppError::Storage(format!("failed to open automation db: {e}")))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS automations (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                kind TEXT NOT NULL,
                config TEXT NOT NULL,
                last_run_at INTEGER,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::Storage(format!("failed to create automations table: {e}")))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn create(&self, req: CreateAutomationRequest) -> AppResult<Automation> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_secs();
        let kind = kind_string(&req.config);
        let config_json =
            serde_json::to_string(&req.config).map_err(|e| AppError::Automation(e.to_string()))?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO automations (id, name, enabled, kind, config, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![&id, &req.name, req.enabled as i32, kind, config_json, now, now],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(Automation {
            id,
            name: req.name,
            enabled: req.enabled,
            config: req.config,
            last_run_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn list(&self) -> AppResult<Vec<Automation>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, enabled, kind, config, last_run_at, created_at, updated_at
                 FROM automations ORDER BY created_at DESC",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                let config_json: String = row.get(4)?;
                let config: AutomationConfig = serde_json::from_str(&config_json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?;
                Ok(Automation {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    enabled: row.get::<_, i32>(2)? != 0,
                    config,
                    last_run_at: row.get::<_, Option<i64>>(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| AppError::Storage(e.to_string()))?);
        }
        Ok(out)
    }

    pub fn get(&self, id: &str) -> AppResult<Automation> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, enabled, kind, config, last_run_at, created_at, updated_at
                 FROM automations WHERE id = ?1",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map([id], |row| {
                let config_json: String = row.get(4)?;
                let config: AutomationConfig = serde_json::from_str(&config_json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?;
                Ok(Automation {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    enabled: row.get::<_, i32>(2)? != 0,
                    config,
                    last_run_at: row.get::<_, Option<i64>>(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| AppError::Storage(e.to_string()))?;
        rows.next()
            .transpose()
            .map_err(|e| AppError::Storage(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("automation {id}")))
    }

    pub fn update(&self, id: &str, req: UpdateAutomationRequest) -> AppResult<Automation> {
        let mut automation = self.get(id)?;
        let now = now_secs();
        if let Some(name) = req.name {
            automation.name = name;
        }
        if let Some(enabled) = req.enabled {
            automation.enabled = enabled;
        }
        if let Some(config) = req.config {
            automation.config = config;
        }
        let kind = kind_string(&automation.config);
        let config_json = serde_json::to_string(&automation.config)
            .map_err(|e| AppError::Automation(e.to_string()))?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE automations
             SET name = ?1, enabled = ?2, kind = ?3, config = ?4, updated_at = ?5
             WHERE id = ?6",
            params![
                &automation.name,
                automation.enabled as i32,
                kind,
                config_json,
                now,
                id
            ],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        automation.updated_at = now;
        Ok(automation)
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        let affected = conn
            .execute("DELETE FROM automations WHERE id = ?1", [id])
            .map_err(|e| AppError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("automation {id}")));
        }
        Ok(())
    }

    pub fn record_run(&self, id: &str) -> AppResult<()> {
        let now = now_secs();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE automations SET last_run_at = ?1 WHERE id = ?2",
            params![now, id],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn disable(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE automations SET enabled = 0, updated_at = ?1 WHERE id = ?2",
            params![now_secs(), id],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    }
}

/// Runs automations: scheduled messages, auto-replies, forwarders and webhooks.
pub struct AutomationScheduler {
    store: Arc<AutomationStore>,
    client: Arc<Mutex<Option<Arc<Client>>>>,
    registry: Arc<std::sync::RwLock<Registry>>,
    events: broadcast::Sender<WsEvent>,
    reply_cooldowns: Mutex<HashMap<(String, String), i64>>,
}

impl AutomationScheduler {
    pub fn new(
        store: Arc<AutomationStore>,
        client: Arc<Mutex<Option<Arc<Client>>>>,
        registry: Arc<std::sync::RwLock<Registry>>,
        events: broadcast::Sender<WsEvent>,
    ) -> Self {
        Self {
            store,
            client,
            registry,
            events,
            reply_cooldowns: Mutex::new(HashMap::new()),
        }
    }

    pub fn start(self: Arc<Self>) {
        let ticker = self.clone();
        tokio::spawn(async move {
            let mut int = interval(Duration::from_secs(TICK_INTERVAL_SECS));
            loop {
                int.tick().await;
                ticker.run_scheduled_messages().await;
            }
        });

        let listener = self.clone();
        tokio::spawn(async move {
            let mut rx = listener.events.subscribe();
            loop {
                match rx.recv().await {
                    Ok(event) => listener.handle_event(&event).await,
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
        });
    }

    async fn run_scheduled_messages(&self) {
        let automations = match self.store.list() {
            Ok(a) => a,
            Err(e) => {
                error!("failed to list automations: {e}");
                return;
            }
        };
        let now = Utc::now();
        for automation in automations {
            if !automation.enabled {
                continue;
            }
            let AutomationConfig::ScheduledMessage { to, text, schedule } = &automation.config else {
                continue;
            };
            let due = match parse_schedule(schedule) {
                Ok(Schedule::OneShot(dt)) => now >= dt,
                Ok(Schedule::Cron(sch)) => {
                    let last_ts = automation
                        .last_run_at
                        .unwrap_or(automation.created_at)
                        .max(automation.created_at);
                    let last_run = DateTime::from_timestamp(last_ts, 0)
                        .unwrap_or(DateTime::UNIX_EPOCH);
                    let next_after_last = sch.after(&last_run).next();
                    match next_after_last {
                        Some(next) => now >= next,
                        None => false,
                    }
                }
                Err(e) => {
                    warn!("invalid schedule for automation {}: {e}", automation.id);
                    continue;
                }
            };
            if due {
                info!("running scheduled automation {}", automation.id);
                for jid in to {
                    if let Err(e) = self.send_text(jid, text).await {
                        error!("scheduled message failed to {jid}: {e}");
                    }
                }
                if parse_schedule(schedule).map(|s| matches!(s, Schedule::OneShot(_))).unwrap_or(false) {
                    let _ = self.store.disable(&automation.id);
                } else {
                    let _ = self.store.record_run(&automation.id);
                }
            }
        }
    }

    async fn handle_event(&self, event: &WsEvent) {
        match event {
            WsEvent::Message(msg) => {
                if !msg.from_me {
                    self.run_replies(msg).await;
                    self.run_forwarders(msg).await;
                    self.run_webhooks(&WebhookEvent::MessageReceived(msg.clone())).await;
                } else {
                    self.run_webhooks(&WebhookEvent::MessageSent(msg.clone())).await;
                }
            }
            WsEvent::Connected => {
                self.run_webhooks(&WebhookEvent::Connected).await;
            }
            _ => {}
        }
    }

    async fn run_replies(&self, msg: &Message) {
        let automations = match self.store.list() {
            Ok(a) => a,
            Err(_) => return,
        };
        for automation in automations {
            if !automation.enabled {
                continue;
            }
            let AutomationConfig::AutoReply {
                chat_filter,
                match_type,
                pattern,
                response,
                cooldown_secs,
            } = &automation.config
            else {
                continue;
            };
            if let Some(filter) = chat_filter
                && filter != &msg.chat
            {
                continue;
            }
            if !pattern_matches(&msg.text, pattern, *match_type) {
                continue;
            }
            let key = (automation.id.clone(), msg.chat.clone());
            let now = now_secs();
            {
                let cooldowns = self.reply_cooldowns.lock().unwrap();
                if let Some(last) = cooldowns.get(&key)
                    && now - last < *cooldown_secs as i64
                {
                    continue;
                }
            }
            if let Err(e) = self.send_text(&msg.chat, response).await {
                error!("auto-reply failed: {e}");
            } else {
                self.reply_cooldowns.lock().unwrap().insert(key, now);
            }
        }
    }

    async fn run_forwarders(&self, msg: &Message) {
        let automations = match self.store.list() {
            Ok(a) => a,
            Err(_) => return,
        };
        for automation in automations {
            if !automation.enabled {
                continue;
            }
            let AutomationConfig::Forwarder { from, to, filter } = &automation.config else {
                continue;
            };
            if from != &msg.chat {
                continue;
            }
            if let Some(f) = filter
                && !msg.text.to_lowercase().contains(&f.to_lowercase())
            {
                continue;
            }
            let text = format!("[Forwarded from {}] {}: {}", msg.chat, msg.sender_name, msg.text);
            if let Err(e) = self.send_text(to, &text).await {
                error!("forwarder failed: {e}");
            }
        }
    }

    async fn run_webhooks(&self, event: &WebhookEvent) {
        let automations = match self.store.list() {
            Ok(a) => a,
            Err(_) => return,
        };
        for automation in automations {
            if !automation.enabled {
                continue;
            }
            let AutomationConfig::Webhook {
                url,
                on_message,
                on_sent,
                on_connected,
            } = &automation.config
            else {
                continue;
            };
            let should_fire = match event {
                WebhookEvent::MessageReceived(_) => *on_message,
                WebhookEvent::MessageSent(_) => *on_sent,
                WebhookEvent::Connected => *on_connected,
            };
            if !should_fire {
                continue;
            }
            let payload = serde_json::json!({
                "event": event.name(),
                "data": event.payload(),
            });
            let url = url.clone();
            tokio::task::spawn_blocking(move || {
                let _ = ureq::post(&url)
                    .set("Content-Type", "application/json")
                    .send_json(payload);
            });
        }
    }

    async fn send_text(&self, jid: &str, text: &str) -> AppResult<Message> {
        let parsed: Jid = jid.parse().map_err(|_| AppError::InvalidJid(jid.to_string()))?;
        let client = self
            .client
            .lock()
            .unwrap()
            .clone()
            .ok_or(AppError::NotConnected)?;
        super::send_text_tracked(&client, &self.registry, &self.events, &parsed, jid, text).await
    }
}

enum Schedule {
    OneShot(DateTime<Utc>),
    Cron(Box<CronSchedule>),
}

fn parse_schedule(s: &str) -> AppResult<Schedule> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(Schedule::OneShot(dt.with_timezone(&Utc)));
    }
    let sch: CronSchedule = s.parse().map_err(|e| AppError::InvalidInput(format!("invalid cron: {e}")))?;
    Ok(Schedule::Cron(Box::new(sch)))
}

fn pattern_matches(text: &str, pattern: &str, match_type: MatchType) -> bool {
    match match_type {
        MatchType::Contains => text.to_lowercase().contains(&pattern.to_lowercase()),
        MatchType::Exact => text.eq_ignore_ascii_case(pattern),
        MatchType::Regex => Regex::new(pattern)
            .map(|re| re.is_match(text))
            .unwrap_or(false),
    }
}

fn kind_string(config: &AutomationConfig) -> &'static str {
    match config {
        AutomationConfig::ScheduledMessage { .. } => "scheduled_message",
        AutomationConfig::AutoReply { .. } => "auto_reply",
        AutomationConfig::Forwarder { .. } => "forwarder",
        AutomationConfig::Webhook { .. } => "webhook",
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

enum WebhookEvent {
    MessageReceived(Message),
    MessageSent(Message),
    Connected,
}

impl WebhookEvent {
    fn name(&self) -> &'static str {
        match self {
            WebhookEvent::MessageReceived(_) => "message_received",
            WebhookEvent::MessageSent(_) => "message_sent",
            WebhookEvent::Connected => "connected",
        }
    }

    fn payload(&self) -> serde_json::Value {
        match self {
            WebhookEvent::MessageReceived(m) | WebhookEvent::MessageSent(m) => {
                serde_json::to_value(m).unwrap_or(serde_json::Value::Null)
            }
            WebhookEvent::Connected => serde_json::Value::Object(Default::default()),
        }
    }
}
