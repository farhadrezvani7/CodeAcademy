use anyhow::Context;
use codeacademy_api::{config::Config, repo::accounts, router, seed, AppState, Clock, MIGRATOR};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "codeacademy_api=info,tower_http=info".into()))
        .init();

    let config = Config::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .context("connecting to PostgreSQL")?;
    MIGRATOR.run(&pool).await.context("running migrations")?;
    if seed::sync(&pool).await? {
        tracing::info!("academic content synchronized");
    }
    if std::env::args().any(|a| a == "--sync-only") {
        return Ok(());
    }

    let clock = Clock::new(config.timezone);
    let removed = accounts::delete_expired_sessions(&pool, clock.now()).await?;
    if removed > 0 {
        tracing::info!(removed, "expired sessions removed");
    }

    let bind = config.bind_addr.clone();
    let state = AppState { pool, config: Arc::new(config), clock };
    let listener = tokio::net::TcpListener::bind(&bind).await.with_context(|| format!("binding {bind}"))?;
    tracing::info!("کد آکادمی API listening on http://{bind}");
    axum::serve(listener, router(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
