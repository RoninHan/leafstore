mod controller;
mod flash;
mod middleware;
mod tools;

use axum::{
    http::{Method, StatusCode},
    middleware as axum_middleware,
    routing::{delete, get, get_service, post},
    Router,
};

use middleware::auth::Auth;
use migration::{Migrator, MigratorTrait};
use service::sea_orm::Database;

use std::{env, fmt::format};
use tera::Tera;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing;

use crate::controller::block::BlockController;
use crate::controller::search_history::SearchHistoryController;
use crate::controller::user::UserController;

use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::{response::BucketExistsResponse, types::S3Api, Client, ClientBuilder};

use tools::AppState;

/// 应用程序入口函数
/// 负责初始化数据库连接、模板引擎和路由配置
#[tokio::main]
async fn start() -> anyhow::Result<()> {
    // 设置日志级别
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();
    let cors = CorsLayer::new()
        .allow_origin(Any) // 允许所有来源，生产环境建议指定具体来源
        .allow_methods([Method::GET, Method::POST, Method::DELETE]) // 允许的 HTTP 方法
        .allow_headers(Any); // 允许所有请求头

    // 加载环境变量
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    // 初始化 MinIO 客戶端 (minio crate v0.3)
    let base_url = "http://localhost:9000/".parse::<BaseUrl>()?;
    let bucket = std::env::var("MINIO_BUCKET").unwrap_or_else(|_| "collection".to_string());

    let static_provider = StaticProvider::new("minioadmin", "minioadmin", None);
    let client = ClientBuilder::new(base_url.clone())
        .provider(Some(Box::new(static_provider)))
        .build()?;

    let resp: BucketExistsResponse = client.bucket_exists(bucket.clone()).send().await?;
    if !resp.exists {
        client.create_bucket(bucket.clone()).send().await.unwrap();
    };

    // 连接数据库并执行迁移
    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    Migrator::up(&conn, None).await.unwrap();

    // 初始化模板引擎
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"))
        .expect("Tera initialization failed");

    let base_url = format!("http://localhost:9000/{}", bucket.clone());

    // 创建应用状态
    let state = AppState {
        templates,
        conn,
        client,
        bucket,
        base_url,
    };

    // 配置路由
    let app = Router::new()
        // 用户认证相关路由
        .route("/api/login", post(UserController::login))
        // 用户管理相关路由
        .route(
            "/api/user",
            get(UserController::list_users).layer(axum_middleware::from_fn_with_state(
                state.clone(),
                Auth::authorization_middleware,
            )),
        )
        .route(
            "/api/user/:id",
            get(UserController::get_user_by_id).layer(axum_middleware::from_fn_with_state(
                state.clone(),
                Auth::authorization_middleware,
            )),
        )
        .route(
            "/api/user/new",
            post(UserController::create_user).layer(axum_middleware::from_fn_with_state(
                state.clone(),
                Auth::authorization_middleware,
            )),
        )
        .route(
            "/api/user/update/:id",
            post(UserController::update_user).layer(axum_middleware::from_fn_with_state(
                state.clone(),
                Auth::authorization_middleware,
            )),
        )
        .route(
            "/api/user/delete/:id",
            delete(UserController::delete_user).layer(axum_middleware::from_fn_with_state(
                state.clone(),
                Auth::authorization_middleware,
            )),
        )
        // block 相关路由
        .route(
            "/api/block",
            get(controller::block::BlockController::block_list).layer(
                axum_middleware::from_fn_with_state(state.clone(), Auth::authorization_middleware),
            ),
        )
        .route(
            "/api/block/:id",
            get(controller::block::BlockController::get_block).layer(
                axum_middleware::from_fn_with_state(state.clone(), Auth::authorization_middleware),
            ),
        )
        .route(
            "/api/block/new",
            post(controller::block::BlockController::create_block).layer(
                axum_middleware::from_fn_with_state(state.clone(), Auth::authorization_middleware),
            ),
        )
        .route(
            "/api/block/update/:id",
            post(controller::block::BlockController::update_block).layer(
                axum_middleware::from_fn_with_state(state.clone(), Auth::authorization_middleware),
            ),
        )
        .route(
            "/api/block/delete/:id",
            delete(controller::block::BlockController::delete_block).layer(
                axum_middleware::from_fn_with_state(state.clone(), Auth::authorization_middleware),
            ),
        )
        .route(
            "/api/search_history",
            get(SearchHistoryController::get_search_history_by_uid).layer(
                axum_middleware::from_fn_with_state(state.clone(), Auth::authorization_middleware),
            ),
        )
        .route(
            "/api/search_history/new",
            post(SearchHistoryController::create_search_history).layer(
                axum_middleware::from_fn_with_state(state.clone(), Auth::authorization_middleware),
            ),
        )
        .route(
            "/api/search_history/delete/:id",
            delete(SearchHistoryController::delete_all_search_history).layer(
                axum_middleware::from_fn_with_state(state.clone(), Auth::authorization_middleware),
            ),
        )
        .route("/api/upload_pic", post(controller::block::BlockController::upload_pic).layer(
            axum_middleware::from_fn_with_state(state.clone(), Auth::authorization_middleware),
        ))  
        .route("/api/delete_pic", post(controller::block::BlockController::delete_pic).layer(
            axum_middleware::from_fn_with_state(state.clone(), Auth::authorization_middleware),
        ))
        // 静态文件服务
        .nest_service(
            "/static",
            get_service(ServeDir::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            )))
            .handle_error(|error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {error}"),
                )
            }),
        )
        .nest_service(
            "/uploads",
            get_service(ServeDir::new("./uploads")).handle_error(|error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {error}"),
                )
            }),
        )
        .layer(cors) // 添加 CORS 中间件
        .layer(CookieManagerLayer::new())
        .with_state(state);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(&server_url).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

/// 程序入口点
pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
