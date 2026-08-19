use gloo_net::http::Request;
use whatsapp_api_types::domain::automation::{
    Automation, CreateAutomationRequest, UpdateAutomationRequest,
};
use whatsapp_api_types::domain::chat::Chat;
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

pub async fn send_text(jid: &str, text: &str) -> Result<Message, String> {
    let resp = Request::post(&api_url(&format!("/api/chats/{jid}/messages")))
        .json(&serde_json::json!({ "body": text, "type": "text" }))
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(format!("HTTP {status}: {body}"))
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
