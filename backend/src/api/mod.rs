//! HTTP layer. Handlers only extract input, call a service and wrap the
//! result; all rules live in `services` and `domain`.

mod handlers;

use crate::state::AppState;
use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, set_header::SetResponseHeaderLayer, trace::TraceLayer};

pub fn router(state: AppState) -> Router {
    let origins: Vec<HeaderValue> = state.config.cors_origins.iter().filter_map(|o| o.parse().ok()).collect();
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        .allow_headers([header::CONTENT_TYPE])
        .allow_credentials(true);

    let v1 = Router::new()
        .route("/health", get(handlers::health))
        .route("/overview", get(handlers::overview))
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/auth/logout", post(handlers::logout))
        .route("/me", get(handlers::me).put(handlers::update_profile))
        .route("/me/password", post(handlers::change_password))
        .route("/dashboard", get(handlers::dashboard))
        .route("/levels", get(handlers::levels))
        .route("/learning-path", get(handlers::learning_path))
        .route("/skill-tree", get(handlers::skill_tree))
        .route("/departments", get(handlers::departments))
        .route("/departments/{slug}", get(handlers::department))
        .route("/courses", get(handlers::courses))
        .route("/courses/{slug}", get(handlers::course))
        .route("/lessons/{id}", get(handlers::lesson))
        .route("/lessons/{id}/complete", post(handlers::complete_lesson))
        .route("/dictionary", get(handlers::dictionary))
        .route("/dictionary/{slug}", get(handlers::term))
        .route("/achievements", get(handlers::achievements))
        .route("/stats", get(handlers::stats))
        .route("/transcript", get(handlers::transcript))
        .route("/passport", get(handlers::passport))
        .route("/leaderboard", get(handlers::leaderboard))
        .route("/missions/today", get(handlers::today_mission))
        .route("/check-in", post(handlers::check_in))
        .route("/activity", post(handlers::log_activity))
        .fallback(handlers::not_found);

    Router::new()
        .nest("/api/v1", v1)
        .layer(DefaultBodyLimit::max(64 * 1024))
        .layer(SetResponseHeaderLayer::overriding(header::CACHE_CONTROL, HeaderValue::from_static("no-store")))
        .layer(SetResponseHeaderLayer::overriding(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")))
        .layer(SetResponseHeaderLayer::overriding(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY")))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
