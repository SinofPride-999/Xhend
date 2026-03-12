#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::{_entities::users, transfer_history, transfers};

#[derive(Debug, Deserialize, Serialize)]
pub struct TransferResponse {
    pub id: String,
    pub file_name: String,
    pub file_size: i64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HistoryResponse {
    pub id: String,
    pub file_name: String,
    pub file_size: i64,
    pub recipient_name: Option<String>,
    pub completed_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SenderInfoResponse {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateHistoryParams {
    pub transfer_id: String,
    pub recipient_name: Option<String>,
    pub recipient_ip: Option<String>,
}

pub async fn get_active_transfers(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    let transfers = transfers::Model::find_by_sender(&ctx.db, user.id).await?;

    let active: Vec<TransferResponse> = transfers
        .into_iter()
        .filter(|t| t.status == "pending" || t.status == "active")
        .map(|t| TransferResponse {
            id: t.transfer_id,
            file_name: t.file_name,
            file_size: t.file_size,
            status: t.status,
            created_at: t.created_at.to_rfc3339(),
        })
        .collect();

    format::json(active)
}

pub async fn get_transfer_history(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    let history = transfer_history::Model::find_by_user(&ctx.db, user.id).await?;

    let response: Vec<HistoryResponse> = history
        .into_iter()
        .map(|h| HistoryResponse {
            id: h.transfer_id,
            file_name: h.file_name,
            file_size: h.file_size,
            recipient_name: h.recipient_name,
            completed_at: h.completed_at.to_rfc3339(),
        })
        .collect();

    format::json(response)
}

pub async fn get_sender_info(
    Path(transfer_id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let transfer = transfers::Model::find_by_transfer_id(&ctx.db, &transfer_id).await?;

    let user = users::Model::find_by_id(&ctx.db, transfer.sender_id).await?;

    format::json(SenderInfoResponse {
        name: user.name,
        email: user.email,
    })
}

pub async fn create_history(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateHistoryParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    let transfer = transfers::Model::find_by_transfer_id(&ctx.db, &params.transfer_id).await?;

    let history = transfer_history::Model::create_history(
        &ctx.db,
        &params.transfer_id,
        user.id,
        &transfer.file_name,
        transfer.file_size,
        transfer.file_type,
        params.recipient_name,
        params.recipient_ip,
    )
    .await?;

    format::json(history)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/transfers/active", get(get_active_transfers))
        .add("/transfers/history", get(get_transfer_history))
        .add("/transfers/history", post(create_history))
        .add("/sender/{transfer_id}", get(get_sender_info))
}
