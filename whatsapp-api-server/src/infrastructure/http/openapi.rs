use utoipa::openapi::path::{HttpMethod, OperationBuilder};
use utoipa::openapi::{Response, ResponsesBuilder};
use utoipa::{Modify, OpenApi};
use whatsapp_api_types::domain::automation::{
    Automation, AutomationConfig, AutomationKind, CreateAutomationRequest, MatchType,
    UpdateAutomationRequest,
};
use whatsapp_api_types::domain::chat::{Chat, ChatKind, MessageRef};
use whatsapp_api_types::domain::message::{Message, MessageStatus};
use whatsapp_api_types::domain::ws_event::{WsEvent, WsRequest};

use super::{
    __path_chat_messages, __path_chats, __path_create_automation, __path_delete_automation,
    __path_ensure_chat, __path_get_automation, __path_health, __path_list_automations,
    __path_resolve_phone, __path_send_message, __path_trigger_automation, __path_update_automation,
    EnsureChatRequest, ErrorBody, Health, ResolvePhoneRequest, ResolvePhoneResponse,
    SendMessageRequest, SendMessageResponse,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "whatsapp-api",
        version = env!("CARGO_PKG_VERSION"),
        description = "WhatsApp Web API: REST endpoints for chats and messages, plus a WebSocket event stream."
    ),
    paths(
        health,
        chats,
        chat_messages,
        send_message,
        ensure_chat,
        resolve_phone,
        list_automations,
        create_automation,
        get_automation,
        update_automation,
        delete_automation,
        trigger_automation
    ),
    components(schemas(
        Health,
        SendMessageRequest,
        SendMessageResponse,
        EnsureChatRequest,
        ResolvePhoneRequest,
        ResolvePhoneResponse,
        ErrorBody,
        Chat,
        ChatKind,
        MessageRef,
        Message,
        MessageStatus,
        WsEvent,
        WsRequest,
        Automation,
        AutomationConfig,
        AutomationKind,
        CreateAutomationRequest,
        UpdateAutomationRequest,
        MatchType
    )),
    modifiers(&AddWebSocketPath)
)]
pub struct ApiDoc;

/// OpenAPI has no native WebSocket support, so `/ws` is documented manually.
struct AddWebSocketPath;

impl Modify for AddWebSocketPath {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let operation = OperationBuilder::new()
            .summary(Some("WebSocket event stream"))
            .description(Some(
                "Upgrades to a WebSocket connection. On connect the server sends a snapshot \
                 (`QrCode` while waiting for a scan, `Connected` once paired), then streams \
                 live frames. The client may send `WsRequest` frames (e.g. `send_message`).",
            ))
            .responses(ResponsesBuilder::new().response(
                "101",
                Response::new("Upgraded to a WebSocket connection"),
            ))
            .tag("realtime")
            .build();
        openapi
            .paths
            .add_path_operation("/ws", vec![HttpMethod::Get], operation);
    }
}
