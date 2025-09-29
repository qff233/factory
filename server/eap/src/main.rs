
use sqlx::postgres::PgPoolOptions;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, prelude::*};

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
    let _guard = init_tracing();
    info!("Starting server...");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    info!("Connecting to database: {}", database_url);
    let _pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");
    info!("Database pool created successfully");

    Ok(())
}
