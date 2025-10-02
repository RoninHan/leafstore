use ::entity::{blocks, blocks::Entity as Block};
use chrono::Utc;
use prelude::DateTimeWithTimeZone;
use sea_orm::{sqlx::types::uuid, *};
use serde::{Deserialize, Serialize};
use serde_json;


#[derive(Deserialize, Serialize, Debug)]
pub struct BlockModel {
    pub context: Option<String>,
    pub imgs: Option<Vec<String>>,
    pub location: Option<String>,
    pub latitude_and_longitude: Option<String>,
    pub draft: Option<bool>,
}

pub struct BlockServices;

impl BlockServices {
    pub async fn create_block(db: &DbConn, form_data: BlockModel, user_id: uuid::Uuid) -> Result<blocks::Model, DbErr> {
        let now = DateTimeWithTimeZone::from(Utc::now());
        blocks::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            pid: Set(Some(user_id.to_string())),
            context: Set(form_data.context),
            imgs: Set(form_data.imgs.map(|imgs| serde_json::to_value(imgs).unwrap())),
            location: Set(form_data.location),
            latitude_and_longitude: Set(form_data.latitude_and_longitude),
            draft: Set(form_data.draft),
            create_time: Set(now),
            update_time: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await
    }

    pub async fn get_block_by_id(db: &DbConn, id: uuid::Uuid) -> Result<Option<blocks::Model>, DbErr> {
        Block::find_by_id(id).one(db).await
    }

    pub async fn find_blocks(db: &DbConn, page: u64, per_page: u64) -> Result<(Vec<blocks::Model>, u64), DbErr> {
        let paginator = Block::find()
            .order_by_asc(blocks::Column::CreateTime)
            .paginate(db, per_page);
        let num_pages = paginator.num_pages().await?;

        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    pub async fn find_blocks_by_pid(db: &DbConn, pid: uuid::Uuid, page: u64, per_page: u64) -> Result<(Vec<blocks::Model>, u64), DbErr> {
        let paginator = Block::find()
            .filter(blocks::Column::Pid.eq(pid))
            .order_by_asc(blocks::Column::CreateTime)
            .paginate(db, per_page);
        let num_pages = paginator.num_pages().await?;

        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    pub async fn update_block_by_id(db: &DbConn, id: uuid::Uuid, form_data: BlockModel) -> Result<blocks::Model, DbErr> {
        let block: blocks::ActiveModel = Block::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find block.".to_owned()))
            .map(Into::into)?;
        let now = DateTimeWithTimeZone::from(Utc::now());
        blocks::ActiveModel {
            id: block.id,
            context: Set(form_data.context),
            imgs: Set(form_data.imgs.map(|imgs| serde_json::to_value(imgs).unwrap())),
            location: Set(form_data.location),
            latitude_and_longitude: Set(form_data.latitude_and_longitude),
            draft: Set(form_data.draft),
            update_time: Set(now),
            ..block
        }
        .update(db)
        .await
    }

    pub async fn delete_block_by_id(db: &DbConn, id: uuid::Uuid) -> Result<DeleteResult, DbErr> {
        Block::delete_by_id(id).exec(db).await
    }
}
