mod config;
mod entities;
mod errors;
mod migration;
mod models;
mod routes;
mod services;
mod sse;
mod state;

use config::Config;
use migration::Migrator;
use sea_orm::{ConnectionTrait, Database, Statement};
use sea_orm_migration::MigratorTrait;
use state::AppState;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Healthcheck : vérifie la connexion DB et quitte
    if args.iter().any(|a| a == "--health") {
        return healthcheck().await;
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rclone_replication_ui=debug,sea_orm=warn,sqlx=warn,info".parse().unwrap()),
        )
        .init();

    let config = Config::from_env()?;
    tracing::info!("Connecting to database...");

    let db = Database::connect(&config.database_url).await?;

    tracing::info!("Running migrations...");
    Migrator::up(&db, None).await?;

    let state = AppState::new(db, config.clone());

    tracing::info!("Building cron scheduler...");
    services::scheduler::rebuild_scheduler(state.clone()).await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = routes::build_router(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("Listening on {}", config.bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}

/// Healthcheck léger : connecte à la BDD, exécute SELECT 1, et quitte.
async fn healthcheck() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?;

    let db = Database::connect(&url).await?;
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT 1".to_string(),
    ))
    .await?;

    println!("OK");
    Ok(())
}
