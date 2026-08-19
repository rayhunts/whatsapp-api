use thiserror::Error;

/// Application-level error type shared by every crate in the workspace.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid chat jid: {0}")]
    InvalidJid(String),

    #[error("not connected to whatsapp")]
    NotConnected,

    #[error("failed to send message: {0}")]
    Send(String),

    #[error("engine error: {0}")]
    Engine(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("automation error: {0}")]
    Automation(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}

pub type AppResult<T> = Result<T, AppError>;
