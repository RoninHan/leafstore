use crate::{
    tools::{AppState, Params, ResponseData, ResponseStatus},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json, Extension,
};
use entity::users::Model as UserEntity;
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