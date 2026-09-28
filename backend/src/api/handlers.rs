use crate::{
    auth::{session_token, CurrentUser},
    error::{AppError, AppResult},
    services::{academics, auth, dashboard, dictionary, engagement, lessons, records, stats},
    state::AppState,
};
use axum::{
    extract::{rejection::JsonRejection, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Value};

/// JSON body extractor that reports malformed input as a validation error.
pub struct Body<T>(pub T);

impl<T: DeserializeOwned, S: Send + Sync> axum::extract::FromRequest<S> for Body<T> {
    type Rejection = AppError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(v)) => Ok(Body(v)),
            Err(e) => Err(AppError::Validation(match e {
                JsonRejection::MissingJsonContentType(_) => "بدنه‌ی درخواست باید JSON باشد.".into(),
                _ => "داده‌ی ارسالی نامعتبر است.".into(),
            })),
        }
    }
}

pub async fn health(State(state): State<AppState>) -> AppResult<Json<Value>> {
    sqlx::query("SELECT 1").execute(&state.pool).await?;
    Ok(Json(json!({ "status": "ok" })))
}

pub async fn not_found() -> AppError {
    AppError::NotFound("مسیر")
}

pub async fn overview(State(s): State<AppState>) -> AppResult<Json<academics::Overview>> {
    Ok(Json(academics::overview(&s).await?))
}

fn session_cookie(state: &AppState, token: &str, max_age: i64) -> HeaderValue {
    let secure = if state.config.cookie_secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}",
        auth::SESSION_COOKIE
    ))
    .expect("cookie is ASCII")
}

async fn with_session(state: &AppState, session: auth::IssuedSession, status: StatusCode) -> AppResult<Response> {
    let me = dashboard::me(state, session.user_id).await?;
    let cookie = session_cookie(state, &session.token, auth::SESSION_DAYS * 86_400);
    Ok((status, [(header::SET_COOKIE, cookie)], Json(me)).into_response())
}

pub async fn register(State(s): State<AppState>, Body(req): Body<auth::RegisterRequest>) -> AppResult<Response> {
    let session = auth::register(&s, req).await?;
    with_session(&s, session, StatusCode::CREATED).await
}

pub async fn login(State(s): State<AppState>, Body(req): Body<auth::LoginRequest>) -> AppResult<Response> {
    let session = auth::login(&s, req).await?;
    with_session(&s, session, StatusCode::OK).await
}

pub async fn logout(State(s): State<AppState>, headers: HeaderMap) -> AppResult<Response> {
    if let Some(token) = session_token(&headers) {
        auth::logout(&s, &token).await?;
    }
    Ok((StatusCode::NO_CONTENT, [(header::SET_COOKIE, session_cookie(&s, "", 0))]).into_response())
}

pub async fn me(State(s): State<AppState>, u: CurrentUser) -> AppResult<Json<dashboard::Me>> {
    Ok(Json(dashboard::me(&s, u.id).await?))
}

pub async fn update_profile(State(s): State<AppState>, u: CurrentUser, Body(req): Body<auth::ProfileRequest>) -> AppResult<Json<dashboard::Me>> {
    auth::update_profile(&s, u.id, req).await?;
    Ok(Json(dashboard::me(&s, u.id).await?))
}

pub async fn change_password(State(s): State<AppState>, u: CurrentUser, Body(req): Body<auth::PasswordRequest>) -> AppResult<StatusCode> {
    auth::change_password(&s, u.id, &u.token, req).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn dashboard(State(s): State<AppState>, u: CurrentUser) -> AppResult<Json<dashboard::Dashboard>> {
    Ok(Json(dashboard::dashboard(&s, u.id).await?))
}

pub async fn levels(State(s): State<AppState>, u: CurrentUser) -> AppResult<Json<academics::LevelsResponse>> {
    Ok(Json(academics::levels(&s, u.id).await?))
}

pub async fn learning_path(State(s): State<AppState>, u: CurrentUser) -> AppResult<Json<academics::LearningPathResponse>> {
    Ok(Json(academics::learning_path(&s, u.id).await?))
}

pub async fn skill_tree(State(s): State<AppState>, u: CurrentUser) -> AppResult<Json<Vec<academics::SkillBranch>>> {
    Ok(Json(academics::skill_tree(&s, u.id).await?))
}

pub async fn departments(State(s): State<AppState>, u: CurrentUser) -> AppResult<Json<Vec<academics::DepartmentDto>>> {
    Ok(Json(academics::departments(&s, u.id).await?))
}

pub async fn department(State(s): State<AppState>, u: CurrentUser, Path(slug): Path<String>) -> AppResult<Json<academics::DepartmentDetail>> {
    Ok(Json(academics::department(&s, u.id, &slug).await?))
}

#[derive(Deserialize)]
pub struct CoursesQuery {
    department: Option<String>,
}

pub async fn courses(State(s): State<AppState>, u: CurrentUser, Query(q): Query<CoursesQuery>) -> AppResult<Json<Vec<academics::CourseCard>>> {
    Ok(Json(academics::courses(&s, u.id, q.department.as_deref()).await?))
}

pub async fn course(State(s): State<AppState>, u: CurrentUser, Path(slug): Path<String>) -> AppResult<Json<academics::CourseDetail>> {
    Ok(Json(academics::course(&s, u.id, &slug).await?))
}

fn lesson_id(raw: &str) -> AppResult<i32> {
    raw.parse::<i32>().ok().filter(|id| *id > 0).ok_or(AppError::NotFound("درس"))
}

pub async fn lesson(State(s): State<AppState>, u: CurrentUser, Path(id): Path<String>) -> AppResult<Json<lessons::LessonDto>> {
    Ok(Json(lessons::get(&s, u.id, lesson_id(&id)?).await?))
}

pub async fn complete_lesson(
    State(s): State<AppState>,
    u: CurrentUser,
    Path(id): Path<String>,
    Body(req): Body<lessons::CompleteRequest>,
) -> AppResult<Json<lessons::CompleteResponse>> {
    Ok(Json(lessons::complete(&s, u.id, lesson_id(&id)?, req).await?))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

pub async fn dictionary(State(s): State<AppState>, Query(q): Query<SearchQuery>) -> AppResult<Json<Vec<dictionary::TermDto>>> {
    Ok(Json(dictionary::search(&s, q.q.as_deref()).await?))
}

pub async fn term(State(s): State<AppState>, Path(slug): Path<String>) -> AppResult<Json<dictionary::TermDto>> {
    Ok(Json(dictionary::term(&s, &slug).await?))
}

pub async fn achievements(State(s): State<AppState>, u: CurrentUser) -> AppResult<Json<records::AchievementsResponse>> {
    Ok(Json(records::achievements(&s, u.id).await?))
}

pub async fn stats(State(s): State<AppState>, u: CurrentUser) -> AppResult<Json<stats::Stats>> {
    Ok(Json(stats::stats(&s, u.id).await?))
}

pub async fn transcript(State(s): State<AppState>, u: CurrentUser) -> AppResult<Json<records::Transcript>> {
    Ok(Json(records::transcript(&s, u.id).await?))
}

pub async fn passport(State(s): State<AppState>, u: CurrentUser) -> AppResult<Json<records::Passport>> {
    Ok(Json(records::passport(&s, u.id).await?))
}

pub async fn leaderboard(State(s): State<AppState>, u: CurrentUser) -> AppResult<Json<records::Leaderboard>> {
    Ok(Json(records::leaderboard(&s, u.id).await?))
}

pub async fn today_mission(State(s): State<AppState>, u: CurrentUser) -> AppResult<Json<engagement::MissionDto>> {
    Ok(Json(engagement::today_mission(&s, u.id).await?))
}

pub async fn check_in(State(s): State<AppState>, u: CurrentUser, Body(req): Body<engagement::CheckInRequest>) -> AppResult<Json<engagement::CheckInResponse>> {
    Ok(Json(engagement::check_in(&s, u.id, req).await?))
}

pub async fn log_activity(State(s): State<AppState>, u: CurrentUser, Body(req): Body<engagement::StudyRequest>) -> AppResult<Json<engagement::SyncOutcome>> {
    Ok(Json(engagement::log_study(&s, u.id, req).await?))
}
