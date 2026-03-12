use loco_rs::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;

pub use super::_entities::transfers::{ActiveModel, Entity, Model};
pub type Transfers = Entity;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CreateTransferParams {
    pub transfer_id: String,
    pub file_name: String,
    pub file_size: i64,
    pub file_type: Option<String>,
}

// implement your read-oriented logic here
impl Model {
    pub async fn create_transfer(
        db: &DatabaseConnection,
        sender_id: i32,
        params: CreateTransferParams,
    ) -> ModelResult<Self> {
        let transfer = ActiveModel {
            pid: Set(uuid::Uuid::new_v4()),
            sender_id: Set(sender_id),
            transfer_id: Set(params.transfer_id),
            file_name: Set(params.file_name),
            file_size: Set(params.file_size),
            file_type: Set(params.file_type),
            status: Set("pending".to_string()),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(transfer)
    }

    pub async fn find_by_transfer_id(
        db: &DatabaseConnection,
        transfer_id: &str,
    ) -> ModelResult<Self> {
        use super::_entities::transfers::Column;

        let transfer = Entity::find()
            .filter(Column::TransferId.eq(transfer_id))
            .one(db)
            .await?;
        transfer.ok_or_else(|| ModelError::EntityNotFound)
    }

    pub async fn find_by_sender(db: &DatabaseConnection, sender_id: i32) -> ModelResult<Vec<Self>> {
        use super::_entities::transfers::Column;

        let transfers = Entity::find()
            .filter(Column::SenderId.eq(sender_id))
            .order_by_desc(Column::CreatedAt)
            .all(db)
            .await?;
        Ok(transfers)
    }

    pub async fn update_status(&self, db: &DatabaseConnection, status: &str) -> ModelResult<Self> {
        let mut active: ActiveModel = self.clone().into();
        active.status = Set(status.to_string());
        active.updated_at = Set(chrono::Utc::now().into());
        active.update(db).await.map_err(ModelError::from)
    }
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
