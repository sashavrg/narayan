use crate::models::{AppState, WsClient, WsMessage};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let client_id = Uuid::new_v4().to_string();
    info!("WebSocket client connected: {}", client_id);

    let (mut sender, mut receiver) = socket.split();

    // Create channel for this client
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Add client to state
    let client = WsClient::new(client_id.clone(), tx);
    state.ws_clients.insert(client_id.clone(), client);

    // Spawn task to send messages to client
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Spawn task to receive messages from client
    let client_id_clone = client_id.clone();
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Err(e) = handle_client_message(&text, &client_id_clone, &state_clone).await {
                    warn!("Error handling client message: {}", e);
                }
            } else if let Message::Close(_) = msg {
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        },
        _ = (&mut recv_task) => {
            send_task.abort();
        },
    }

    // Remove client from state
    state.ws_clients.remove(&client_id);
    info!("WebSocket client disconnected: {}", client_id);
}

async fn handle_client_message(
    text: &str,
    _client_id: &str,
    _state: &AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Received message: {}", text);

    let message: WsMessage = serde_json::from_str(text)?;

    match message {
        WsMessage::Subscribe { job_id } => {
            debug!("Client subscribed to job: {}", job_id);
            // Client subscriptions are implicit - they receive updates for jobs they're interested in
        }
        WsMessage::Unsubscribe { job_id } => {
            debug!("Client unsubscribed from job: {}", job_id);
        }
        WsMessage::Ping => {
            debug!("Received ping");
            // Pong is handled automatically
        }
        _ => {
            warn!("Unexpected message from client: {:?}", message);
        }
    }

    Ok(())
}

pub fn broadcast_progress(state: &AppState, job_id: String, file_index: usize, percent: f32) {
    let message = WsMessage::Progress {
        job_id: job_id.clone(),
        file: format!("File {}", file_index + 1),
        percent,
    };

    if let Ok(json) = message.to_json() {
        // Broadcast to all connected clients
        for client in state.ws_clients.iter() {
            client.value().send(json.clone());
        }
    } else {
        error!("Failed to serialize progress message");
    }
}

pub fn broadcast_complete(state: &AppState, job_id: String) {
    let message = WsMessage::Complete { job_id };

    if let Ok(json) = message.to_json() {
        for client in state.ws_clients.iter() {
            client.value().send(json.clone());
        }
    }
}

pub fn broadcast_error(state: &AppState, job_id: String, error_message: String) {
    let message = WsMessage::Error {
        job_id,
        message: error_message,
    };

    if let Ok(json) = message.to_json() {
        for client in state.ws_clients.iter() {
            client.value().send(json.clone());
        }
    }
}
