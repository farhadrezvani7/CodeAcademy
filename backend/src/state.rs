use crate::config::Config;
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use std::sync::Arc;

/// Wall clock in the academy's timezone. Tests can pin "now".
#[derive(Debug, Clone)]
pub struct Clock {
    pub tz: Tz,
    fixed: Option<DateTime<Utc>>,
}

impl Clock {
    pub fn new(tz: Tz) -> Self {
        Self { tz, fixed: None }
    }

    pub fn fixed(tz: Tz, at: DateTime<Utc>) -> Self {
        Self { tz, fixed: Some(at) }
    }

    pub fn now(&self) -> DateTime<Utc> {
        self.fixed.unwrap_or_else(Utc::now)
    }

    pub fn today(&self) -> NaiveDate {
        self.now().with_timezone(&self.tz).date_naive()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub clock: Clock,
}
