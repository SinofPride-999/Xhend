#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20260312_151008_transfers;
mod m20260312_151214_create_transfer_history;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260312_151008_transfers::Migration),
            Box::new(m20260312_151214_create_transfer_history::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}