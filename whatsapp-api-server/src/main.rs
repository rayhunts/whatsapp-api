mod application;
mod infrastructure;

use std::sync::Arc;

use tracing::info;
use tracing_subscriber::EnvFilter;
use whatsapp_api_engine::application::EngineService;
use whatsapp_api_engine::infrastructure::WhatsappEngine;

const DEFAULT_PORT: u16 = 8080;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,whatsapp_api_server=debug")),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "whatsapp.db".into());

    let engine: Arc<dyn whatsapp_api_engine::domain::WaEngine> =
        Arc::new(WhatsappEngine::new(db_path));
    let service = EngineService::new(engine);
    service.start().await;

    let app = application::build_app(service);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    info!("whatsapp-api-server listening on http://0.0.0.0:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}