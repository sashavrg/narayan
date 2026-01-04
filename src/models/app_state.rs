use crate::services::job_manager::JobManager;
use axum::extract::ws::WebSocket;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

pub type WsClients = Arc<DashMap<String, WsClient>>;

#[derive(Clone)]
pub struct AppState {
    pub job_manager: Arc<JobManager>,
    pub ws_clients: WsClients,
}

impl AppState {
    pub fn new(job_manager: Arc<JobManager>) -> Self {
        Self {
            job_manager,
            ws_clients: Arc::new(DashMap::new()),
        }
    }
}

pub struct WsClient {
    pub id: String,
    pub sender: mpsc::UnboundedSender<String>,
}

impl WsClient {
    pub fn new(id: String, sender: mpsc::UnboundedSender<String>) -> Self {
        Self { id, sender }
    }

    pub fn send(&self, message: String) {
        let _ = self.sender.send(message);
    }
}
