use serde::{Deserialize, Serialize};
use service::sea_orm::DatabaseConnection;
use tera::Tera;
use minio::s3::Client;
#[derive(Clone)]
pub struct AppState {
    pub templates: Tera,
    pub conn: DatabaseConnection,
    pub client: Client,
    pub bucket: String,
    pub base_url: String,
}

#[derive(Deserialize)]
pub struct Params {
    pub page: Option<u64>,
    pub posts_per_page: Option<u64>,
    pub q: Option<String>,
    pub categories_id: Option<i32>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlashData {
    pub kind: String,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub enum ResponseStatus {
    Success,
    Error,
}

#[derive(Deserialize, Serialize)]
pub struct ResponseData<T> {
    pub status: ResponseStatus,
    pub code: i32,
    pub message: Option<String>,
    pub data: Option<T>,
}