use tauri::{AppHandle, Emitter};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct NotificationPayload {
    pub title: String,
    pub message: Option<String>,
}

/// Send an error notification to the frontend
pub fn notify_error(app: &AppHandle, title: impl Into<String>, message: Option<impl Into<String>>) {
    let payload = NotificationPayload {
        title: title.into(),
        message: message.map(|m| m.into()),
    };
    
    tracing::error!("Error notification: {} - {:?}", payload.title, payload.message);
    
    if let Err(e) = app.emit("error", &payload) {
        tracing::error!("Failed to emit error event: {}", e);
    }
}

/// Send a success notification to the frontend
pub fn notify_success(app: &AppHandle, title: impl Into<String>, message: Option<impl Into<String>>) {
    let payload = NotificationPayload {
        title: title.into(),
        message: message.map(|m| m.into()),
    };
    
    tracing::info!("Success notification: {} - {:?}", payload.title, payload.message);
    
    if let Err(e) = app.emit("success", &payload) {
        tracing::error!("Failed to emit success event: {}", e);
    }
}

/// Send a warning notification to the frontend
pub fn notify_warning(app: &AppHandle, title: impl Into<String>, message: Option<impl Into<String>>) {
    let payload = NotificationPayload {
        title: title.into(),
        message: message.map(|m| m.into()),
    };
    
    tracing::warn!("Warning notification: {} - {:?}", payload.title, payload.message);
    
    if let Err(e) = app.emit("warning", &payload) {
        tracing::error!("Failed to emit warning event: {}", e);
    }
}

/// Send an info notification to the frontend
pub fn notify_info(app: &AppHandle, title: impl Into<String>, message: Option<impl Into<String>>) {
    let payload = NotificationPayload {
        title: title.into(),
        message: message.map(|m| m.into()),
    };
    
    tracing::info!("Info notification: {} - {:?}", payload.title, payload.message);
    
    if let Err(e) = app.emit("info", &payload) {
        tracing::error!("Failed to emit info event: {}", e);
    }
}
