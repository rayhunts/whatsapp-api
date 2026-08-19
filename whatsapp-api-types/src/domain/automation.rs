use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Classification of an automation rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AutomationKind {
    /// Send a message on a cron schedule or one-shot time.
    ScheduledMessage,
    /// Reply automatically when an incoming message matches a pattern.
    AutoReply,
    /// Copy incoming messages from one chat to another.
    Forwarder,
    /// POST events to an external URL.
    Webhook,
}

/// How an auto-reply pattern should be matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    /// Message text contains the pattern (case-insensitive).
    #[default]
    Contains,
    /// Message text equals the pattern exactly.
    Exact,
    /// Pattern is a regular expression.
    Regex,
}

/// Automation configuration payload. The variant is selected by `kind`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AutomationConfig {
    ScheduledMessage {
        /// One or more target JIDs. For private chats they are built from a phone number.
        #[schema(example = json!(["6281234567890@s.whatsapp.net"]))]
        to: Vec<String>,
        #[schema(example = "Hello from the scheduler")]
        text: String,
        /// Cron expression in standard 5-field format (`* * * * *`) or an ISO-8601
        /// one-shot timestamp for a single run.
        #[schema(example = "0 8 * * *")]
        schedule: String,
    },
    AutoReply {
        /// Restrict rule to one chat JID, or `None` to apply to every chat.
        chat_filter: Option<String>,
        match_type: MatchType,
        #[schema(example = "price")]
        pattern: String,
        #[schema(example = "Our price list is available at ...")]
        response: String,
        /// Minimum seconds between replies from this rule.
        cooldown_secs: u64,
    },
    Forwarder {
        #[schema(example = "source-group@g.us")]
        from: String,
        #[schema(example = "target-group@g.us")]
        to: String,
        /// Optional text filter. When present, only messages containing this text are forwarded.
        filter: Option<String>,
    },
    Webhook {
        #[schema(example = "https://example.com/webhook")]
        url: String,
        #[serde(default = "default_true")]
        on_message: bool,
        #[serde(default = "default_true")]
        on_sent: bool,
        #[serde(default = "default_false")]
        on_connected: bool,
    },
}

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}

/// Automation rule stored in the database and exposed over the API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Automation {
    pub id: String,
    pub name: String,
    #[serde(default = "default_true_bool")]
    pub enabled: bool,
    #[serde(flatten)]
    pub config: AutomationConfig,
    #[serde(default)]
    pub last_run_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

fn default_true_bool() -> bool {
    true
}

/// Request body for creating a new automation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CreateAutomationRequest {
    pub name: String,
    #[serde(default = "default_true_bool")]
    pub enabled: bool,
    #[serde(flatten)]
    pub config: AutomationConfig,
}

/// Request body for updating an existing automation.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateAutomationRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub config: Option<AutomationConfig>,
}

/// Normalizes a phone number entered by a user into a WhatsApp private JID.
pub fn phone_to_jid(raw: &str) -> Option<String> {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 7 {
        return None;
    }
    Some(format!("{digits}@s.whatsapp.net"))
}

/// Normalizes a phone number list into a list of JIDs, silently dropping invalid entries.
pub fn phones_to_jids(phones: &[String]) -> Vec<String> {
    phones.iter().filter_map(|p| phone_to_jid(p)).collect()
}

/// Returns `true` if the given JID belongs to a WhatsApp group.
pub fn is_group_jid(jid: &str) -> bool {
    jid.ends_with("@g.us")
}

/// Returns `true` if the given JID looks like a broadcast list.
pub fn is_broadcast_jid(jid: &str) -> bool {
    jid.ends_with("@broadcast")
}
