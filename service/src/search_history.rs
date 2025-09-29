use ::entity::{search_history, search_history::Entity as SearchHistory};
use chrono::Utc;
use prelude::DateTimeWithTimeZone;
use sea_orm::{sqlx::types::uuid, *};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct SearchHistoryModel {
    pub history: Option<sea_orm::prelude::Json>,
}

pub struct SearchHistoryServices;

impl SearchHistoryServices {
    pub async fn create_search_history(db: &DbConn, form_data: SearchHistoryModel, uid: uuid::Uuid) -> Result<search_history::ActiveModel, DbErr> {
        let now = DateTimeWithTimeZone::from(Utc::now());
        search_history::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            uid: Set(Some(uid)),
            history: Set(form_data.history),
            create_time: Set(now),
            update_time: Set(now),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn get_search_history_by_id(db: &DbConn, id: uuid::Uuid) -> Result<Option<search_history::Model>, DbErr> {
        SearchHistory::find_by_id(id).one(db).await
    }

    pub async fn get_search_history_by_uid(db: &DbConn, uid: uuid::Uuid) -> Result<Vec<search_history::Model>, DbErr> {
        SearchHistory::find()
            .filter(search_history::Column::Uid.eq(uid))
            .all(db)
            .await
    }

    pub async fn update_search_history_by_id(db: &DbConn, uid: uuid::Uuid, form_data: SearchHistoryModel) -> Result<search_history::Model, DbErr> {
        let history: search_history::ActiveModel = SearchHistory::find()
            .filter(search_history::Column::Uid.eq(uid))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find search_history.".to_owned()))
            .map(Into::into)?;
        let now = DateTimeWithTimeZone::from(Utc::now());
        search_history::ActiveModel {
            id: history.id,
            uid: history.uid,
            history: Set(form_data.history),
            update_time: Set(now),
            ..history
        }
        .update(db)
        .await
    }

    pub async fn delete_search_history_by_id(db: &DbConn, id: uuid::Uuid) -> Result<DeleteResult, DbErr> {
        SearchHistory::delete_by_id(id).exec(db).await
    }

    pub async fn delete_all_search_history_by_uid(db: &DbConn, uid: uuid::Uuid) -> Result<DeleteResult, DbErr> {
        SearchHistory::delete_many()
            .filter(search_history::Column::Uid.eq(uid))
            .exec(db)
            .await
    }
}


