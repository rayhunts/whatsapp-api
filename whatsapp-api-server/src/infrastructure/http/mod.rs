use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use whatsapp_api_engine::domain::ConnectionState;
use whatsapp_api_errors::AppError;
use whatsapp_api_types::domain::automation::{
    Automation, CreateAutomationRequest, UpdateAutomationRequest,
};
use whatsapp_api_types::domain::chat::{Chat, ChatKind};
use whatsapp_api_types::domain::contact::{
    BroadcastRequest, BroadcastResponse, Contact, ContactGroup, CreateContactGroupRequest,
    CreateContactRequest, UpdateContactGroupRequest, UpdateContactRequest,
};
use whatsapp_api_types::domain::message::{Message, SendMessageRequest, SendMessageResponse};

use crate::application::AppState;

pub mod openapi;

#[derive(Serialize, ToSchema)]
pub struct Health {
    #[schema(example = "ok")]
    status: String,
    connected: bool,
    qr_pending: bool,
}

#[derive(Serialize, ToSchema)]
struct ErrorBody {
    error: String,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct EnsureChatRequest {
    #[schema(example = "6281234567890@s.whatsapp.net")]
    jid: String,
    #[schema(example = "Acme Support")]
    name: Option<String>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct ResolvePhoneRequest {
    #[schema(example = "+62 812-3456-7890")]
    phone: String,
}

#[derive(serde::Serialize, ToSchema)]
pub struct ResolvePhoneResponse {
    jid: String,
    kind: ChatKind,
}

/// Newtype so the foreign `AppError` can become an axum `Response`.
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            AppError::InvalidJid(_) | AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            AppError::NotConnected => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Send(_) | AppError::Engine(_) | AppError::Storage(_) | AppError::Automation(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
        };
        (status, Json(ErrorBody { error: self.0.to_string() })).into_response()
    }
}

#[utoipa::path(
    get,
    path = "/api/health",
    tag = "health",
    responses(
        (status = 200, description = "Server + connection state", body = Health)
    )
)]
pub async fn health(State(state): State<Arc<AppState>>) -> Json<Health> {
    Json(Health {
        status: "ok".into(),
        connected: state.service.state() == ConnectionState::Connected,
        qr_pending: state.service.last_qr().is_some(),
    })
}

#[utoipa::path(
    get,
    path = "/api/chats",
    tag = "chats",
    responses(
        (status = 200, description = "Chat list", body = Vec<Chat>)
    )
)]
pub async fn chats(State(state): State<Arc<AppState>>) -> Json<Vec<whatsapp_api_types::domain::chat::Chat>> {
    Json(state.service.chats().await)
}

#[utoipa::path(
    get,
    path = "/api/chats/{jid}/messages",
    tag = "messages",
    params(
        ("jid" = String, Path, description = "Chat JID, e.g. `123456789@s.whatsapp.net` (groups end in `@g.us`)")
    ),
    responses(
        (status = 200, description = "Message history for the chat", body = Vec<Message>),
        (status = 400, description = "Invalid chat JID", body = ErrorBody),
        (status = 404, description = "Chat not found", body = ErrorBody),
        (status = 503, description = "Not connected to WhatsApp", body = ErrorBody)
    )
)]
pub async fn chat_messages(
    State(state): State<Arc<AppState>>,
    Path(jid): Path<String>,
) -> Result<Json<Vec<whatsapp_api_types::domain::message::Message>>, ApiError> {
    Ok(Json(state.service.messages(&jid).await))
}

#[utoipa::path(
    post,
    path = "/api/chats/{jid}/messages",
    tag = "messages",
    params(
        ("jid" = String, Path, description = "Chat JID, e.g. `123456789@s.whatsapp.net` (groups end in `@g.us`)")
    ),
    request_body = SendMessageRequest,
    responses(
        (status = 201, description = "Message accepted", body = SendMessageResponse),
        (status = 400, description = "Invalid chat JID or message type", body = ErrorBody),
        (status = 404, description = "Chat not found", body = ErrorBody),
        (status = 503, description = "Not connected to WhatsApp", body = ErrorBody),
        (status = 500, description = "Failed to send message", body = ErrorBody)
    )
)]
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Path(jid): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<SendMessageResponse>), ApiError> {
    if req.r#type != "text" {
        Err(AppError::InvalidInput(format!(
            "unsupported message type: {}",
            req.r#type
        )))?;
    }
    state.service.ensure_chat(&jid, None).await;
    let message = state.service.send_text(&jid, &req.body).await?;
    let created_at = chrono::DateTime::from_timestamp_millis(message.timestamp_ms)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default();
    Ok((
        StatusCode::CREATED,
        Json(SendMessageResponse {
            id: message.id,
            jid: message.chat,
            body: message.text,
            status: message.status,
            created_at,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/chats/ensure",
    tag = "chats",
    request_body = EnsureChatRequest,
    responses(
        (status = 204, description = "Chat ensured"),
        (status = 400, description = "Invalid chat JID", body = ErrorBody)
    )
)]
pub async fn ensure_chat(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EnsureChatRequest>,
) -> Result<StatusCode, ApiError> {
    state.service.ensure_chat(&req.jid, req.name).await;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/chats/resolve-phone",
    tag = "chats",
    request_body = ResolvePhoneRequest,
    responses(
        (status = 200, description = "Resolved JID", body = ResolvePhoneResponse),
        (status = 400, description = "Invalid phone number", body = ErrorBody)
    )
)]
pub async fn resolve_phone(
    Json(req): Json<ResolvePhoneRequest>,
) -> Result<Json<ResolvePhoneResponse>, ApiError> {
    let jid = whatsapp_api_types::domain::automation::phone_to_jid(&req.phone)
        .ok_or_else(|| AppError::InvalidInput(format!("invalid phone number: {}", req.phone)))?;
    Ok(Json(ResolvePhoneResponse {
        jid: jid.clone(),
        kind: ChatKind::from_jid(&jid),
    }))
}

#[utoipa::path(
    get,
    path = "/api/automations",
    tag = "automations",
    responses(
        (status = 200, description = "List of automations", body = Vec<Automation>)
    )
)]
pub async fn list_automations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Automation>>, ApiError> {
    Ok(Json(state.service.list_automations().await?))
}

#[utoipa::path(
    post,
    path = "/api/automations",
    tag = "automations",
    request_body = CreateAutomationRequest,
    responses(
        (status = 201, description = "Automation created", body = Automation)
    )
)]
pub async fn create_automation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAutomationRequest>,
) -> Result<(StatusCode, Json<Automation>), ApiError> {
    let created = state.service.create_automation(req).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    get,
    path = "/api/automations/{id}",
    tag = "automations",
    responses(
        (status = 200, description = "Automation details", body = Automation),
        (status = 404, description = "Automation not found", body = ErrorBody)
    )
)]
pub async fn get_automation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Automation>, ApiError> {
    Ok(Json(state.service.get_automation(&id).await?))
}

#[utoipa::path(
    put,
    path = "/api/automations/{id}",
    tag = "automations",
    request_body = UpdateAutomationRequest,
    responses(
        (status = 200, description = "Automation updated", body = Automation),
        (status = 404, description = "Automation not found", body = ErrorBody)
    )
)]
pub async fn update_automation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAutomationRequest>,
) -> Result<Json<Automation>, ApiError> {
    Ok(Json(state.service.update_automation(&id, req).await?))
}

#[utoipa::path(
    delete,
    path = "/api/automations/{id}",
    tag = "automations",
    responses(
        (status = 204, description = "Automation deleted"),
        (status = 404, description = "Automation not found", body = ErrorBody)
    )
)]
pub async fn delete_automation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.service.delete_automation(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/automations/{id}/trigger",
    tag = "automations",
    responses(
        (status = 204, description = "Automation triggered"),
        (status = 400, description = "Cannot trigger this automation type", body = ErrorBody),
        (status = 404, description = "Automation not found", body = ErrorBody)
    )
)]
pub async fn trigger_automation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.service.trigger_automation(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/contacts",
    tag = "contacts",
    responses(
        (status = 200, description = "Contact list", body = Vec<Contact>)
    )
)]
pub async fn list_contacts(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Contact>>, ApiError> {
    Ok(Json(state.service.list_contacts().await?))
}

#[utoipa::path(
    post,
    path = "/api/contacts",
    tag = "contacts",
    request_body = CreateContactRequest,
    responses(
        (status = 201, description = "Contact created", body = Contact)
    )
)]
pub async fn create_contact(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateContactRequest>,
) -> Result<(StatusCode, Json<Contact>), ApiError> {
    let created = state.service.create_contact(req).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    get,
    path = "/api/contacts/{id}",
    tag = "contacts",
    responses(
        (status = 200, description = "Contact details", body = Contact),
        (status = 404, description = "Contact not found", body = ErrorBody)
    )
)]
pub async fn get_contact(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Contact>, ApiError> {
    Ok(Json(state.service.get_contact(&id).await?))
}

#[utoipa::path(
    patch,
    path = "/api/contacts/{id}",
    tag = "contacts",
    request_body = UpdateContactRequest,
    responses(
        (status = 200, description = "Contact updated", body = Contact),
        (status = 404, description = "Contact not found", body = ErrorBody)
    )
)]
pub async fn update_contact(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateContactRequest>,
) -> Result<Json<Contact>, ApiError> {
    Ok(Json(state.service.update_contact(&id, req).await?))
}

#[utoipa::path(
    delete,
    path = "/api/contacts/{id}",
    tag = "contacts",
    responses(
        (status = 204, description = "Contact deleted"),
        (status = 404, description = "Contact not found", body = ErrorBody)
    )
)]
pub async fn delete_contact(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.service.delete_contact(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/contact-groups",
    tag = "contact groups",
    responses(
        (status = 200, description = "Contact group list", body = Vec<ContactGroup>)
    )
)]
pub async fn list_contact_groups(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ContactGroup>>, ApiError> {
    Ok(Json(state.service.list_contact_groups().await?))
}

#[utoipa::path(
    post,
    path = "/api/contact-groups",
    tag = "contact groups",
    request_body = CreateContactGroupRequest,
    responses(
        (status = 201, description = "Contact group created", body = ContactGroup)
    )
)]
pub async fn create_contact_group(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateContactGroupRequest>,
) -> Result<(StatusCode, Json<ContactGroup>), ApiError> {
    let created = state.service.create_contact_group(req).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    get,
    path = "/api/contact-groups/{id}",
    tag = "contact groups",
    responses(
        (status = 200, description = "Contact group details", body = ContactGroup),
        (status = 404, description = "Contact group not found", body = ErrorBody)
    )
)]
pub async fn get_contact_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ContactGroup>, ApiError> {
    Ok(Json(state.service.get_contact_group(&id).await?))
}

#[utoipa::path(
    put,
    path = "/api/contact-groups/{id}",
    tag = "contact groups",
    request_body = UpdateContactGroupRequest,
    responses(
        (status = 200, description = "Contact group updated", body = ContactGroup),
        (status = 404, description = "Contact group not found", body = ErrorBody)
    )
)]
pub async fn update_contact_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateContactGroupRequest>,
) -> Result<Json<ContactGroup>, ApiError> {
    Ok(Json(state.service.update_contact_group(&id, req).await?))
}

#[utoipa::path(
    delete,
    path = "/api/contact-groups/{id}",
    tag = "contact groups",
    responses(
        (status = 204, description = "Contact group deleted"),
        (status = 404, description = "Contact group not found", body = ErrorBody)
    )
)]
pub async fn delete_contact_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.service.delete_contact_group(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/broadcasts",
    tag = "broadcasts",
    request_body = BroadcastRequest,
    responses(
        (status = 202, description = "Broadcast queued", body = BroadcastResponse),
        (status = 400, description = "Invalid request", body = ErrorBody)
    )
)]
pub async fn broadcast(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BroadcastRequest>,
) -> Result<(StatusCode, Json<BroadcastResponse>), ApiError> {
    let total = state
        .service
        .broadcast(&req.group_ids, &req.to, &req.message)
        .await?;
    let broadcast_id = uuid::Uuid::new_v4().to_string();
    Ok((
        StatusCode::ACCEPTED,
        Json(BroadcastResponse {
            broadcast_id,
            total,
            status: "queued".into(),
        }),
    ))
}
