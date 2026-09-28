use anyhow::{Context, Result};
use chrono_tz::Tz;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub cors_origins: Vec<String>,
    /// Adds `Secure` to the session cookie. Enable when served over HTTPS.
    pub cookie_secure: bool,
    pub timezone: Tz,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
        let tz_name = std::env::var("APP_TIMEZONE").unwrap_or_else(|_| "Asia/Tehran".into());
        Ok(Self {
            database_url,
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into()),
            cors_origins: std::env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173".into())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            cookie_secure: std::env::var("COOKIE_SECURE").is_ok_and(|v| v == "true" || v == "1"),
            timezone: tz_name.parse().map_err(|_| anyhow::anyhow!("invalid APP_TIMEZONE: {tz_name}"))?,
        })
    }
}
