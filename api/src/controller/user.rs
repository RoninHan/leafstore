use crate::{
    middleware::auth::Auth,
    tools::{AppState, Params, ResponseData, ResponseStatus},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use service::{
    sea_orm::sqlx::types::uuid,
    user::{LoginModel, UserModel, UserServices},
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::to_value;

pub struct UserController;

impl UserController {
    pub async fn list_users(
        state: State<AppState>,
        Query(params): Query<Params>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        let page = params.page.unwrap_or(1);
        let posts_per_page = params.posts_per_page.unwrap_or(5);

        let (users, num_pages) = UserServices::find_user(&state.conn, page, posts_per_page)
            .await
            .expect("Cannot find posts in page");

        let data = ResponseData {
            code: 200,
            status: ResponseStatus::Success,
            data: Some(json!({
                "rows": users,
                "num_pages": num_pages,
            })),
            message: Some("Users retrieved successfully".to_string()),
        };

        let json_data = to_value(data).unwrap();
        println!("Json data: {:?}", json_data);
        Ok(Json(json!(json_data)))
    }

    pub async fn create_user(
        state: State<AppState>,
        Json(payload): Json<UserModel>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        println!("Payload: {:?}", payload);
        // password md5
        // let payload = UserModel {
        //     password: Auth::hash_password(&payload.password)
        //         .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password"))?,
        //     ..payload
        // };
        UserServices::create_user(&state.conn, payload)
            .await
            .map_err(|e| {
                println!("Failed to create user: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user")
            })?;

        let data = ResponseData::<Option<serde_json::Value>> {
            code: 201,
            status: ResponseStatus::Success,
            data: None,
            message: Some("User created successfully".to_string()),
        };
        let json_data = to_value(data).unwrap();
        println!("Json data: {:?}", json_data);
        Ok(Json(json!(json_data)))
    }

    pub async fn update_user(
        state: State<AppState>,
        Path(id): Path<uuid::Uuid>,
        Json(payload): Json<UserModel>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        println!("Payload: {:?}", payload);
        UserServices::update_user_by_id(&state.conn, id, payload)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update user"))?;

        let data = ResponseData::<Option<serde_json::Value>> {
            code: 200,
            status: ResponseStatus::Success,
            data: None,
            message: Some("User updated successfully".to_string()),
        };
        let json_data = to_value(data).unwrap();
        println!("Json data: {:?}", json_data);
        Ok(Json(json!(json_data)))
    }

    pub async fn delete_user(
        state: State<AppState>,
        Path(id): Path<uuid::Uuid>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        UserServices::delete_user(&state.conn, id)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete user"))?;

        let data = ResponseData::<Option<serde_json::Value>> {
            code: 200,
            status: ResponseStatus::Success,
            data: None,
            message: Some("User deleted successfully".to_string()),
        };
        let json_data = to_value(data).unwrap();
        println!("Json data: {:?}", json_data);
        Ok(Json(json!(json_data)))
    }

    pub async fn get_user_by_id(
        state: State<AppState>,
        Path(id): Path<uuid::Uuid>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        let user = UserServices::find_user_by_id(&state.conn, id)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to find user"))?;

        let data = match user {
            Some(user) => ResponseData {
                code: 200,
                status: ResponseStatus::Success,
                data: Some(json!(user)),
                message: Some("User retrieved successfully".to_string()),
            },
            None => ResponseData {
                code: 404,
                status: ResponseStatus::Error,
                data: None,
                message: Some("User not found".to_string()),
            },
        };
        let json_data = to_value(data).unwrap();
        println!("Json data: {:?}", json_data);
        Ok(Json(json!(json_data)))
    }

    pub async fn login(
        state: State<AppState>,
        Json(payload): Json<LoginModel>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
        let js_code = &payload.js_code;
        // Call WeChat jscode2session to exchange js_code for openid
        let url = "https://api.weixin.qq.com/sns/jscode2session";
        let req = Jscode2SessionRequest {
            appid: "wxf17c75a716ce6ede".to_string(),
            secret: "2c605b107f99f435b91963d67776f1d6".to_string(),
            js_code: js_code.clone(),
            grant_type: "authorization_code".to_string(),
        };

        println!("Request to WeChat jscode2session: {:?}", req);

        let client = reqwest::Client::new();
        let resp = client
            .get(url)
            .query(&req)
            .send()
            .await
            .map_err(|_| (StatusCode::BAD_GATEWAY, "Failed to call jscode2session"))?
            .json::<Jscode2SessionResponse>()
            .await
            .map_err(|_| (StatusCode::BAD_GATEWAY, "Invalid jscode2session response"))?;
        println!("WeChat jscode2session response: {:?}", resp);
        // If WeChat returns an error code
        if resp.errcode.unwrap_or(0) != 0 {
            return Err((StatusCode::BAD_GATEWAY, "WeChat jscode2session error"));
        }

        let openid = resp
            .openid
            .clone()
            .ok_or((StatusCode::BAD_REQUEST, "Missing openid"))?;

        // Find user by appid (openid); if not found, create a new user
        let user = match UserServices::find_user_by_appid(&state.conn, &openid)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to find user"))?
        {
            Some(u) => u,
            None => {
                println!("Creating new user with openid: {}", openid);
                // create minimal user record using the appid as app_id
                let _active_model = UserServices::create_user_with_appid(&state.conn, &openid)
                    .await
                    .map_err(|e| {
                        println!("Failed to create user: {:?}", e);
                        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user")
                    })?;
                // Fetch the newly created user as Model
                UserServices::find_user_by_appid(&state.conn, &openid)
                    .await
                    .map_err(|_| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to fetch created user",
                        )
                    })?
                    .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Created user not found"))?
            }
        };

        let token = Auth::encode_jwt(user.app_id.clone()).map_err(|_| {
                        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode token")
                    })?;
        let data = ResponseData {
            code: 200,
            status: ResponseStatus::Success,
            data: Some(json!({
                "user": user,
                    "token": token,
                "session_key": resp.session_key,
            })),
            message: Some("Login successful".to_string()),
        };

        let json_data = to_value(data).unwrap();
        Ok(Json(json!(json_data)))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Jscode2SessionRequest {
    appid: String,
    secret: String,
    js_code: String,
    grant_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Jscode2SessionResponse {
    openid: Option<String>,
    session_key: Option<String>,
    unionid: Option<String>,
    errcode: Option<i32>,
    errmsg: Option<String>,
}
