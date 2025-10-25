use axum::{Router, routing::post};
use sqlx::postgres::PgPoolOptions;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, prelude::*};

mod auth;
mod handlers;
mod models;
mod state;

fn init_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    let file_appender = tracing_appender::rolling::daily("logs", "server");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_filter(LevelFilter::DEBUG);
    let stdout_layer = fmt::layer()
        .with_level(true)
        .with_writer(std::io::stdout)
        .with_filter(LevelFilter::INFO);

    let collector = tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer);
    tracing::subscriber::set_global_default(collector).expect("Tracing collect error");
    guard
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().unwrap();
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

    let state = state::AppState::new(pool).await;

    let public_routes = Router::new().route("/login", post(handlers::auth::login));
    let protected_routes = Router::new()
        .route("/logout", post(handlers::auth::logout))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::token_auth,
        ));
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("服务器运行在：http://localhost:3000");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
