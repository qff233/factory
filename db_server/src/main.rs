use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, Method},
    routing::post,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use sqlx::{Column, PgPool, Row, postgres::PgPoolOptions, postgres::PgRow};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{debug, error, info};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::time::FormatTime;

use std::fmt;

#[derive(Debug, Deserialize)]
struct SqlRequest {
    sql: String,
    params: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize)]
struct SqlResponse {
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

struct CustomTimeFormat;
impl FormatTime for CustomTimeFormat {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> fmt::Result {
        let now = Local::now();
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}

fn init_tracing() -> WorkerGuard {
    let log_filename = Local::now().format("logs/%Y-%m-%d.log").to_string();
    std::fs::create_dir_all("logs").expect("Failed to create logs directory");

    let file_appender = tracing_appender::rolling::never("logs", log_filename);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        // 包含时间戳
        .with_timer(CustomTimeFormat)
        // 包含日志级别
        .with_level(true)
        // 包含模块路径
        .with_target(true)
        // 设置日志级别过滤
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        // 初始化日志器
        .try_init()
        .expect("Failed to initialize tracing");
    guard
}

async fn execute_sql(
    pool: &PgPool,
    sql: &str,
    params: Option<Vec<serde_json::Value>>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut query = sqlx::query(sql);
    if let Some(params) = params {
        for param in params {
            query = query.bind(param);
        }
    };

    debug!("Executing SQL: {}", sql);

    let rows: Vec<PgRow> = query.fetch_all(pool).await?;
    let mut results = Vec::new();
    for row in rows {
        let mut map = serde_json::Map::new();
        for column in row.columns() {
            let value: serde_json::Value = row.try_get(column.name())?;
            map.insert(column.name().to_string(), value);
        }
        results.push(serde_json::Value::Object(map));
    }

    Ok(serde_json::Value::Array(results))
}

async fn handle_sql_request(
    State(pool): State<PgPool>,
    Json(request): Json<SqlRequest>,
) -> Json<SqlResponse> {
    debug!(
        "received SQL request: sql={}, params={:?}",
        request.sql, request.params
    );

    let result = execute_sql(&pool, &request.sql, request.params).await;
    match result {
        Ok(data) => Json(SqlResponse {
            success: true,
            data: Some(data),
            error: None,
        }),
        Err(e) => {
            error!("SQL request error, {}", e.to_string());
            Json(SqlResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            })
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_tracing();
    info!("Starting server...");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    info!("Connecting to database: {}", database_url);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");
    info!("Database pool created successfully");

    let cors_layer = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]);
    let trace_layer = TraceLayer::new_for_http();
    let app = Router::new()
        .route("/sql", post(handle_sql_request))
        .layer(trace_layer)
        .layer(cors_layer)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
