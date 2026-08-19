use gloo_net::http::Request;
use whatsapp_api_types::domain::automation::{
    Automation, CreateAutomationRequest, UpdateAutomationRequest,
};
use whatsapp_api_types::domain::chat::Chat;
use whatsapp_api_types::domain::contact::{
    BroadcastRequest, BroadcastResponse, Contact, ContactGroup, CreateContactGroupRequest,
    CreateContactRequest, UpdateContactGroupRequest, UpdateContactRequest,
};
use whatsapp_api_types::domain::message::Message;

use crate::state::app_state::API_BASE_URL;

fn api_url(path: &str) -> String {
    let base = API_BASE_URL();
    if base.is_empty() {
        path.to_string()
    } else {
        format!("{}{}", base.trim_end_matches('/'), path)
    }
}

pub async fn fetch_chats() -> Result<Vec<Chat>, gloo_net::Error> {
    Request::get(&api_url("/api/chats")).send().await?.json().await
}

pub async fn fetch_messages(jid: &str) -> Result<Vec<Message>, gloo_net::Error> {
    Request::get(&api_url(&format!("/api/chats/{jid}/messages")))
        .send()
        .await?
        .json()
        .await
}

pub async fn send_text(jid: &str, text: &str) -> Result<(), String> {
    let resp = Request::post(&api_url(&format!("/api/chats/{jid}/messages")))
        .json(&serde_json::json!({ "body": text, "type": "text" }))
        .map_err(|_| "Failed to prepare message".to_string())?
        .send()
        .await
        .map_err(|_| "Network error".to_string())?;
    if resp.ok() {
        // The message will arrive via WebSocket; no need to parse the response body.
        let _ = resp.text().await;
        Ok(())
    } else {
        let body = resp.text().await.unwrap_or_default();
        let message = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| "Failed to send message".to_string());
        Err(message)
    }
}

pub async fn ensure_chat(jid: &str, name: Option<&str>) -> Result<(), gloo_net::Error> {
    let mut body = serde_json::json!({ "jid": jid });
    if let Some(n) = name {
        body["name"] = serde_json::json!(n);
    }
    Request::post(&api_url("/api/chats/ensure"))
        .json(&body)?
        .send()
        .await?;
    Ok(())
}

pub async fn resolve_phone(phone: &str) -> Result<String, gloo_net::Error> {
    let resp = Request::post(&api_url("/api/chats/resolve-phone"))
        .json(&serde_json::json!({ "phone": phone }))?
        .send()
        .await?;
    let json = resp.json::<serde_json::Value>().await?;
    Ok(json["jid"].as_str().unwrap_or_default().to_string())
}

pub async fn list_automations() -> Result<Vec<Automation>, gloo_net::Error> {
    Request::get(&api_url("/api/automations"))
        .send()
        .await?
        .json()
        .await
}

pub async fn create_automation(req: &CreateAutomationRequest) -> Result<Automation, gloo_net::Error> {
    Request::post(&api_url("/api/automations"))
        .json(req)?
        .send()
        .await?
        .json()
        .await
}

pub async fn update_automation(
    id: &str,
    req: &UpdateAutomationRequest,
) -> Result<Automation, gloo_net::Error> {
    Request::put(&api_url(&format!("/api/automations/{id}")))
        .json(req)?
        .send()
        .await?
        .json()
        .await
}

pub async fn delete_automation(id: &str) -> Result<(), gloo_net::Error> {
    Request::delete(&api_url(&format!("/api/automations/{id}")))
        .send()
        .await?;
    Ok(())
}

pub async fn trigger_automation(id: &str) -> Result<(), gloo_net::Error> {
    Request::post(&api_url(&format!("/api/automations/{id}/trigger")))
        .send()
        .await?;
    Ok(())
}

pub async fn list_contacts() -> Result<Vec<Contact>, gloo_net::Error> {
    Request::get(&api_url("/api/contacts")).send().await?.json().await
}

pub async fn create_contact(req: &CreateContactRequest) -> Result<Contact, gloo_net::Error> {
    Request::post(&api_url("/api/contacts"))
        .json(req)?
        .send()
        .await?
        .json()
        .await
}

pub async fn update_contact(id: &str, req: &UpdateContactRequest) -> Result<Contact, gloo_net::Error> {
    Request::patch(&api_url(&format!("/api/contacts/{id}")))
        .json(req)?
        .send()
        .await?
        .json()
        .await
}

pub async fn delete_contact(id: &str) -> Result<(), gloo_net::Error> {
    Request::delete(&api_url(&format!("/api/contacts/{id}")))
        .send()
        .await?;
    Ok(())
}

pub async fn list_contact_groups() -> Result<Vec<ContactGroup>, gloo_net::Error> {
    Request::get(&api_url("/api/contact-groups"))
        .send()
        .await?
        .json()
        .await
}

pub async fn create_contact_group(req: &CreateContactGroupRequest) -> Result<ContactGroup, gloo_net::Error> {
    Request::post(&api_url("/api/contact-groups"))
        .json(req)?
        .send()
        .await?
        .json()
        .await
}

pub async fn update_contact_group(
    id: &str,
    req: &UpdateContactGroupRequest,
) -> Result<ContactGroup, gloo_net::Error> {
    Request::put(&api_url(&format!("/api/contact-groups/{id}")))
        .json(req)?
        .send()
        .await?
        .json()
        .await
}

pub async fn delete_contact_group(id: &str) -> Result<(), gloo_net::Error> {
    Request::delete(&api_url(&format!("/api/contact-groups/{id}")))
        .send()
        .await?;
    Ok(())
}

pub async fn broadcast(req: &BroadcastRequest) -> Result<BroadcastResponse, gloo_net::Error> {
    Request::post(&api_url("/api/broadcasts"))
        .json(req)?
        .send()
        .await?
        .json()
        .await
}
