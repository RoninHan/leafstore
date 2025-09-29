use crate::{
    tools::{AppState, ResponseData, ResponseStatus},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use entity::users::Model as UserEntity;
use service::{sea_orm::sqlx::types::uuid, search_history::SearchHistoryModel, SearchHistoryServices};

use serde_json::json;
use serde_json::to_value;

pub struct SearchHistoryController;

impl SearchHistoryController {

    pub async fn get_search_history_by_uid(
        Extension(user): Extension<UserEntity>,
        State(state): State<AppState>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {

        let history = SearchHistoryServices::get_search_history_by_uid(&state.conn, user.id)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Cannot find search history"))?;

        let data = ResponseData {
            code: 200,
            status: ResponseStatus::Success,
            data: Some(json!({
                "rows": history
            })),
            message: Some("Blocks retrieved successfully".to_string()),
        };

        let json_data = to_value(data).unwrap();
        println!("Json data: {:?}", json_data);
        Ok(Json(json!(json_data)))
    }
    
    pub async fn create_search_history(
        Extension(user): Extension<UserEntity>,
        State(state): State<AppState>,
        Json(payload): Json<SearchHistoryModel>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        SearchHistoryServices::create_search_history(&state.conn, payload, user.id)
            .await
            .map_err(|e| {
                println!("Failed to create search history: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create search history")
            })?;

        let data = ResponseData::<Option<serde_json::Value>> {
            code: 201,
            status: ResponseStatus::Success,
            data: None,
            message: Some("Search history created successfully".to_string()),
        };

        let json_data = to_value(data).unwrap();
        println!("Json data: {:?}", json_data);
        Ok(Json(json!(json_data)))
    }

    pub async fn delete_all_search_history(
        State(state): State<AppState>,
        Path(uid): Path<uuid::Uuid>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        SearchHistoryServices::delete_all_search_history_by_uid(&state.conn, uid)
            .await
            .map_err(|e| {
                println!("Failed to delete all search history: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete all search history")
            })?;

        let data = ResponseData::<Option<serde_json::Value>> {
            code: 200,
            status: ResponseStatus::Success,
            data: None,
            message: Some("All search history deleted successfully".to_string()),
        };

        let json_data = to_value(data).unwrap();
        println!("Json data: {:?}", json_data);
        Ok(Json(json!(json_data)))
    }
}