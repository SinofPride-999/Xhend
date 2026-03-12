#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use crate::models::{_entities::users, transfer_history, transfers};
use loco_rs::prelude::*;

pub async fn profile(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    // Get user's transfer stats
    let total_transfers = transfers::Model::find_by_sender(&ctx.db, user.id)
        .await?
        .len();

    let completed_transfers = transfer_history::Model::find_by_user(&ctx.db, user.id)
        .await?
        .len();

    format::json(serde_json::json!({
        "user": {
            "pid": user.pid,
            "name": user.name,
            "email": user.email,
            "profile_picture": user.profile_picture,
            "verified": user.email_verified_at.is_some(),
        },
        "stats": {
            "total_transfers": total_transfers,
            "completed_transfers": completed_transfers,
        }
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/user")
        .add("/profile", get(profile))
}
