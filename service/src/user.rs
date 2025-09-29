use ::entity::{users, users::Entity as User};
use chrono::{DateTime, Utc};
use prelude::DateTimeWithTimeZone;
use sea_orm::{sqlx::types::uuid, *};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct UserModel {
    pub id: uuid::Uuid,
    pub name: String,
    pub sex: String,
    pub email: String,
    pub phone: String,
    pub birthday: Option<DateTimeWithTimeZone>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginModel {
    pub js_code: String,
}

pub struct UserServices;

impl UserServices {
    pub async fn create_user(
        db: &DbConn,
        form_data: UserModel,
    ) -> Result<users::Model, DbErr> {
        let sex = form_data
            .sex
            .parse::<i32>()
            .map_err(|_| DbErr::Custom("性别必须是数字".to_string()))?;
        let userid = uuid::Uuid::new_v4();
        users::ActiveModel {
            id: Set(userid),
            name: Set(Some(form_data.name.to_owned())),
            sex: Set(Some(sex)),
            email: Set(Some(form_data.email)),
            phone: Set(Some(form_data.phone.to_owned())),
            birthday: Set(form_data.birthday),
            created_at: Set(DateTimeWithTimeZone::from(Utc::now())),
            updated_at: Set(DateTimeWithTimeZone::from(Utc::now())),
            ..Default::default()
        }
        .insert(db)
        .await?;

        User::find()
            .filter(users::Column::Id.eq(userid))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Created user not found".to_string()))
    }

    pub async fn update_user_by_id(
        db: &DbConn,
        id: uuid::Uuid,
        form_data: UserModel,
    ) -> Result<users::Model, DbErr> {
        let users: users::ActiveModel = User::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find users.".to_owned()))
            .map(Into::into)?;
        let sex: i32 = form_data.sex.parse().expect("msg");
        users::ActiveModel {
            id: users.id,
            name: Set(Some(form_data.name.to_owned())),
            email: Set(Some(form_data.email)),
            sex: Set(Some(sex)),
            phone: Set(Some(form_data.phone)),
            birthday: Set(form_data.birthday),
            updated_at: Set(DateTimeWithTimeZone::from(Utc::now())),
            ..Default::default()
        }
        .update(db)
        .await
    }

    pub async fn delete_user(db: &DbConn, id: uuid::Uuid) -> Result<DeleteResult, DbErr> {
        let users: users::ActiveModel = User::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find users.".to_owned()))
            .map(Into::into)?;

        users.delete(db).await
    }

    pub async fn delete_all_users(db: &DbConn) -> Result<DeleteResult, DbErr> {
        User::delete_many().exec(db).await
    }

    pub async fn find_user(
        db: &DbConn,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<users::Model>, u64), DbErr> {
        let paginator = User::find()
            .order_by_asc(users::Column::Id)
            .paginate(db, per_page);
        let num_pages = paginator.num_pages().await?;

        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    pub async fn find_user_by_id(
        db: &DbConn,
        id: uuid::Uuid,
    ) -> Result<Option<users::Model>, DbErr> {
        User::find_by_id(id).one(db).await
    }

    pub async fn find_user_by_appid(
        db: &DbConn,
        appid: &str,
    ) -> Result<Option<users::Model>, DbErr> {
        User::find()
            .filter(users::Column::AppId.eq(appid))
            .one(db)
            .await
    }

    pub async fn create_user_with_appid(db: &DbConn, appid: &str) -> Result<users::Model, DbErr> {
        // create a minimal user record with the provided appid (openid)
        let now = DateTimeWithTimeZone::from(Utc::now());

        let active = users::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            sex: Set(Some(0)),
            app_id: Set(appid.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        // insert the active model (force INSERT so we don't issue an UPDATE with zero rows)
        let _saved = active.insert(db).await?;

        // fetch and return the created model
        User::find()
            .filter(users::Column::AppId.eq(appid))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Created user not found".to_string()))
    }

    pub async fn find_user_by_email(
        db: &DbConn,
        email: &str,
    ) -> Result<Option<users::Model>, DbErr> {
        User::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await
    }
}
