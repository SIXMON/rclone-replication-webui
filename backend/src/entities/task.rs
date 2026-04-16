use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub source_remote_id: Uuid,
    pub source_path: String,
    pub dest_remote_id: Uuid,
    pub dest_path: String,
    pub cron_expression: Option<String>,
    pub enabled: bool,
    #[sea_orm(column_type = "custom(\"TEXT[]\")")]
    pub rclone_flags: Vec<String>,
    pub notification_channel_id: Option<Uuid>,
    #[sea_orm(column_type = "custom(\"TEXT[]\")")]
    pub notify_on: Vec<String>,
    pub max_retries: i32,
    pub retry_delay_seconds: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::remote::Entity",
        from = "Column::SourceRemoteId",
        to = "super::remote::Column::Id"
    )]
    SourceRemote,
    #[sea_orm(
        belongs_to = "super::remote::Entity",
        from = "Column::DestRemoteId",
        to = "super::remote::Column::Id"
    )]
    DestRemote,
    #[sea_orm(
        belongs_to = "super::notification_channel::Entity",
        from = "Column::NotificationChannelId",
        to = "super::notification_channel::Column::Id"
    )]
    NotificationChannel,
    #[sea_orm(has_many = "super::task_run::Entity")]
    TaskRuns,
}

impl Related<super::notification_channel::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NotificationChannel.def()
    }
}

impl Related<super::task_run::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskRuns.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
