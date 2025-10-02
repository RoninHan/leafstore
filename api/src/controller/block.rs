use crate::tools::{AppState, Params, ResponseData, ResponseStatus};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use entity::users::Model as UserEntity;
use futures_util::StreamExt;
use minio::s3::{builders::ObjectContent, types::S3Api};
use service::{block::BlockModel, sea_orm::sqlx::types::uuid, BlockServices};

use serde_json::json;
use serde_json::to_value;

pub struct BlockController;

impl BlockController {
    pub async fn block_list(
        state: State<AppState>,
        Query(params): Query<Params>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        let page = params.page.unwrap_or(1);
        let posts_per_page = params.posts_per_page.unwrap_or(5);

        let (blocks, num_pages) = BlockServices::find_blocks(&state.conn, page, posts_per_page)
            .await
            .expect("Cannot find blocks in page");

        let data = ResponseData {
            code: 200,
            status: ResponseStatus::Success,
            data: Some(json!({
                "rows": blocks,
                "num_pages": num_pages,
            })),
            message: Some("Blocks retrieved successfully".to_string()),
        };

        let json_data = to_value(data).unwrap();
        println!("Json data: {:?}", json_data);
        Ok(Json(json!(json_data)))
    }

    pub async fn upload_pic(
        Extension(user): Extension<UserEntity>,
        State(state): State<AppState>,
        mut multipart: Multipart,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        let mut image_url: Option<Vec<String>> = None;

        loop {
            match multipart.next_field().await {
                Ok(Some(field)) => {
                    let name = field.name().map(|s| s.to_string()).unwrap_or_default();

                    if name == "image" {
                        // 處理圖片上傳
                        if let Some(filename) = field.file_name().map(|f| f.to_string()) {
                            // Read the field as a stream and collect bytes
                            let mut data_bytes: Vec<u8> = Vec::new();
                            let mut stream = field;
                            while let Some(chunk_res) = stream.next().await {
                                match chunk_res {
                                    Ok(chunk) => data_bytes.extend_from_slice(&chunk),
                                    Err(e) => {
                                        eprintln!("failed to read file chunk: {:?}", e);
                                        return Err((
                                            StatusCode::BAD_REQUEST,
                                            "failed to read file bytes",
                                        ));
                                    }
                                }
                            }

                            let object_key = format!("images/{}/{}", user.id, filename);
                            let content = ObjectContent::from(data_bytes.clone());
                            // 上傳到 MinIO
                            let res = state
                                .client
                                .put_object_content(&state.bucket, &object_key, content)
                                .send()
                                .await;

                            match res {
                                Ok(_) => {

                                    // 拼圖片訪問 URL
                                    let url = format!("{}/{}", state.base_url, object_key);
                                    image_url.get_or_insert_with(Vec::new).push(url);
                                }
                                Err(e) => {
                                    eprintln!("上傳圖片失敗: {:?}", e);
                                    return Err((
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        "upload image failed",
                                    ));
                                }
                            }
                        }
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    eprintln!("multipart read error: {:?}", e);
                    return Err((StatusCode::BAD_REQUEST, "failed to read multipart stream"));
                }
            }
        }

        let data = ResponseData {
            code: 200,
            status: ResponseStatus::Success,
            data: Some(json!({
                "image_url": image_url,
            })),
            message: Some("Image uploaded successfully".to_string()),
        };

        let json_data = to_value(data).unwrap();
        println!("Json data: {:?}", json_data);
        Ok(Json(json!(json_data)))
    }

    pub async fn delete_pic(
        Extension(user): Extension<UserEntity>,
        State(state): State<AppState>,
        mut multipart: Multipart,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        let mut image_url: Option<Vec<String>> = None;
        loop {
            match multipart.next_field().await {
                Ok(Some(field)) => {
                    let name = field.name().map(|s| s.to_string()).unwrap_or_default();

                    if name == "image" {
                        // 處理圖片刪除
                        if let Some(filename) = field.file_name() {
                            let object_key = format!("images/{}/{}", user.id, filename);

                            // 從 MinIO 刪除圖片
                            let res = state
                                .client
                                .delete_object(&state.bucket, &object_key)
                                .send()
                                .await;

                            match res {
                                Ok(_) => {
                                    // 拼圖片訪問 URL
                                    let url = format!("{}/{}", state.base_url, object_key);
                                    image_url.get_or_insert_with(Vec::new).push(url);
                                }
                                Err(e) => {
                                    eprintln!("刪除圖片失敗: {:?}", e);
                                    return Err((
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        "delete image failed",
                                    ));
                                }
                            }
                        }
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    eprintln!("multipart read error: {:?}", e);
                    return Err((StatusCode::BAD_REQUEST, "failed to read multipart stream"));
                }
            }
        }
        let data = ResponseData {
            code: 200,
            status: ResponseStatus::Success,
            data: Some(json!({
                "image_url": image_url,
            })),
            message: Some("Image deleted successfully".to_string()),
        };
        let json_data = to_value(data).unwrap();
        println!("Json data: {:?}", json_data);
        Ok(Json(json!(json_data)))
    }

    pub async fn create_block(
        Extension(user): Extension<UserEntity>,
        State(state): State<AppState>,
        Json(payload): Json<BlockModel>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        BlockServices::create_block(&state.conn, payload, user.id)
            .await
            .map_err(|e| {
                println!("Failed to create block: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create block")
            })?;

        let data = ResponseData::<Option<serde_json::Value>> {
            code: 201,
            status: ResponseStatus::Success,
            data: None,
            message: Some("Block created successfully".to_string()),
        };
        Ok(Json(json!(data)))
    }

    pub async fn get_block(
        State(state): State<AppState>,
        Path(id): Path<uuid::Uuid>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        let block = BlockServices::get_block_by_id(&state.conn, id)
            .await
            .map_err(|e| {
                println!("Failed to get block: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get block")
            })?;
        let data = ResponseData {
            code: 200,
            status: ResponseStatus::Success,
            data: Some(json!({
                "block": block,
            })),
            message: Some("Block retrieved successfully".to_string()),
        };
        Ok(Json(json!(data)))
    }

    pub async fn update_block(
        State(state): State<AppState>,
        Path(id): Path<uuid::Uuid>,
        Json(payload): Json<BlockModel>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        BlockServices::update_block_by_id(&state.conn, id, payload)
            .await
            .map_err(|e| {
                println!("Failed to update block: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update block")
            })?;
        let data = ResponseData::<Option<serde_json::Value>> {
            code: 200,
            status: ResponseStatus::Success,
            data: None,
            message: Some("Block updated successfully".to_string()),
        };
        Ok(Json(json!(data)))
    }

    pub async fn delete_block(
        State(state): State<AppState>,
        Path(id): Path<uuid::Uuid>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        BlockServices::delete_block_by_id(&state.conn, id)
            .await
            .map_err(|e| {
                println!("Failed to delete block: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete block")
            })?;
        let data = ResponseData::<Option<serde_json::Value>> {
            code: 200,
            status: ResponseStatus::Success,
            data: None,
            message: Some("Block deleted successfully".to_string()),
        };
        Ok(Json(json!(data)))
    }
}
