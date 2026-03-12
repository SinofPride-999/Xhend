use loco_rs::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;

pub use super::_entities::transfer_history::{ActiveModel, Entity, Model};
pub type TransferHistory = Entity;

// implement your read-oriented logic here
impl Model {
    pub async fn create_history(
        db: &DatabaseConnection,
        transfer_id: &str,
        user_id: i32,
        file_name: &str,
        file_size: i64,
        file_type: Option<String>,
        recipient_name: Option<String>,
        recipient_ip: Option<String>,
    ) -> ModelResult<Self> {
        let history = ActiveModel {
            pid: Set(uuid::Uuid::new_v4()),
            transfer_id: Set(transfer_id.to_string()),
            user_id: Set(user_id),
            file_name: Set(file_name.to_string()),
            file_size: Set(file_size),
            file_type: Set(file_type),
            recipient_name: Set(recipient_name),
            recipient_ip: Set(recipient_ip),
            completed_at: Set(chrono::Utc::now().into()),
            created_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(history)
    }

    pub async fn find_by_user(db: &DatabaseConnection, user_id: i32) -> ModelResult<Vec<Self>> {
        use super::_entities::transfer_history::Column;

        let history = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::CompletedAt)
            .all(db)
            .await?;
        Ok(history)
    }
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
