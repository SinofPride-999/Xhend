#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::extract::{
    ws::{Message, WebSocket},
    State, WebSocketUpgrade,
};
use futures::{SinkExt, StreamExt};
use loco_rs::prelude::*;
use sea_orm::DatabaseConnection;
use serde_json::Value;
use tokio::{
    sync::{broadcast, Mutex},
    time::interval,
};

use crate::models::{
    transfers::{self, CreateTransferParams},
    users::{self, Model as UserModel},
};

#[derive(Clone)]
pub struct WsState {
    connections: Arc<Mutex<HashMap<String, broadcast::Sender<Message>>>>,
    db: DatabaseConnection,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    // Verify user exists
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    let ws_state = WsState {
        connections: Arc::new(Mutex::new(HashMap::new())),
        db: ctx.db.clone(),
    };

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, ws_state, user)))
}

async fn handle_socket(socket: WebSocket, state: WsState, user: UserModel) {
    let conn_id = uuid::Uuid::new_v4().to_string();
    let conn_id_clone = conn_id.clone();
    let user_id = user.id;
    let user_email = user.email.clone();

    println!("User {} connected with socket: {}", user_email, conn_id);

    let (tx, mut rx) = broadcast::channel(100);
    {
        let mut connections = state.connections.lock().await;
        connections.insert(conn_id.clone(), tx.clone());
    }

    let (mut sender, mut receiver) = socket.split();

    // Create a task to send messages to the WebSocket
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Ping task to keep connection alive
    let ping_task = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            // We don't have a direct way to send ping here without sender
            // This is simplified - in production you'd want to send actual pings
        }
    });

    // Store mapping between this connection and its target
    let mut target_id: Option<String> = None;
    let mut current_transfer_id: Option<String> = None;

    let receive_task = tokio::spawn({
        let state = state.clone();
        let current_conn_id = conn_id.clone();

        async move {
            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
                    Message::Text(text) => {
                        println!("Received text from {}: {}", current_conn_id, text);

                        if let Ok(data) = serde_json::from_str::<Value>(&text) {
                            // Handle registration
                            if data["type"] == "register" {
                                if let Some(id) = data["connectionId"].as_str() {
                                    println!("Registering connection with custom ID: {}", id);
                                    let mut connections = state.connections.lock().await;
                                    connections.remove(&current_conn_id);
                                    connections.insert(id.to_string(), tx.clone());
                                }
                                continue;
                            }

                            // Handle file info - save to database
                            if data["type"] == "file-info" {
                                if let Some(transfer_id) = data["target_id"].as_str() {
                                    current_transfer_id = Some(transfer_id.to_string());

                                    // Save transfer to database
                                    let params = CreateTransferParams {
                                        transfer_id: transfer_id.to_string(),
                                        file_name: data["name"]
                                            .as_str()
                                            .unwrap_or("unknown")
                                            .to_string(),
                                        file_size: data["size"].as_i64().unwrap_or(0),
                                        file_type: data["mimeType"].as_str().map(|s| s.to_string()),
                                    };

                                    match transfers::Model::create_transfer(
                                        &state.db, user_id, params,
                                    )
                                    .await
                                    {
                                        Ok(transfer) => {
                                            println!(
                                                "Created transfer record: {}",
                                                transfer.transfer_id
                                            );
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to save transfer: {}", e);
                                        }
                                    }
                                }
                            }

                            // Handle file end - update status
                            if data["type"] == "file-end" {
                                if let Some(transfer_id) = &current_transfer_id {
                                    if let Ok(transfer) = transfers::Model::find_by_transfer_id(
                                        &state.db,
                                        transfer_id,
                                    )
                                    .await
                                    {
                                        let _ =
                                            transfer.update_status(&state.db, "completed").await;
                                    }
                                }
                            }

                            // Extract target_id from message
                            if let Some(target) = data["target_id"].as_str() {
                                target_id = Some(target.to_string());
                                println!("Setting target for {} to: {}", current_conn_id, target);

                                // Forward the message to target
                                let connections = state.connections.lock().await;
                                if let Some(target_tx) = connections.get(target) {
                                    let _ = target_tx.send(Message::Text(text.clone()));
                                }
                            } else {
                                // If no target_id in message but we have a stored target, use it
                                if let Some(target) = &target_id {
                                    let connections = state.connections.lock().await;
                                    if let Some(target_tx) = connections.get(target) {
                                        let _ = target_tx.send(Message::Text(text.clone()));
                                    }
                                }
                            }
                        }
                    }
                    Message::Binary(bin_data) => {
                        println!(
                            "Received binary from {} ({} bytes)",
                            current_conn_id,
                            bin_data.len()
                        );

                        // Forward binary data to target if we have one
                        if let Some(target) = &target_id {
                            let connections = state.connections.lock().await;
                            if let Some(target_tx) = connections.get(target) {
                                let _ = target_tx.send(Message::Binary(bin_data));
                                println!("Forwarded binary to {}", target);
                            }
                        }
                    }
                    Message::Close(_) => {
                        println!("Close message received from {}", current_conn_id);
                        break;
                    }
                    _ => continue,
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = ping_task => {},
        _ = receive_task => {},
    }

    // Clean up connection
    let mut connections = state.connections.lock().await;
    connections.remove(&conn_id_clone);
    println!("Connection closed: {}", conn_id_clone);
}

pub fn routes() -> Routes {
    Routes::new().prefix("api/ws").add("/", get(ws_handler))
}
