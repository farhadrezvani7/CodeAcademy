//! Accounts and sessions: registration, login (with throttling), logout,
//! profile and password changes.

use crate::domain::{fa, jalali};
use crate::error::{AppError, AppResult};
use crate::repo::accounts;
use crate::state::AppState;
use argon2::{
    password_hash::{rand_core::OsRng, rand_core::RngCore, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Duration;
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub const SESSION_COOKIE: &str = "ca_session";
pub const SESSION_DAYS: i64 = 30;
pub const MIN_PASSWORD: usize = 8;
pub const MAX_PASSWORD: usize = 128;
pub const MAX_FAILURES: i64 = 5;
pub const LOCKOUT_MINUTES: i64 = 15;

/// A real hash verified when the email is unknown, so response time does
/// not reveal which emails are registered.
fn dummy_hash() -> String {
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HASH.get_or_init(|| {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default().hash_password(b"not-a-real-password", &salt).map(|h| h.to_string()).unwrap_or_default()
    })
    .clone()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// A freshly issued session: the raw token goes into the cookie only.
pub struct IssuedSession {
    pub user_id: i32,
    pub token: String,
}

pub fn normalize_email(raw: &str) -> AppResult<String> {
    let email = raw.trim().to_lowercase();
    let valid = email.len() <= 254
        && !email.chars().any(char::is_whitespace)
        && email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty() && !domain.contains('@') && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
        });
    if valid {
        Ok(email)
    } else {
        Err(AppError::Validation("ایمیل معتبر نیست.".into()))
    }
}

pub fn normalize_name(raw: &str) -> AppResult<String> {
    let name = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let len = name.chars().count();
    if !(2..=60).contains(&len) {
        return Err(AppError::Validation("نام باید بین ۲ تا ۶۰ نویسه باشد.".into()));
    }
    if name.chars().any(|c| c.is_control() || "<>{}".contains(c)) {
        return Err(AppError::Validation("نام شامل نویسه‌های غیرمجاز است.".into()));
    }
    Ok(name)
}

pub fn check_password_strength(password: &str) -> AppResult<()> {
    let len = password.chars().count();
    if len < MIN_PASSWORD {
        return Err(AppError::Validation("رمز عبور باید حداقل ۸ نویسه باشد.".into()));
    }
    if len > MAX_PASSWORD {
        return Err(AppError::Validation("رمز عبور بیش از حد طولانی است.".into()));
    }
    let has_letter = password.chars().any(char::is_alphabetic);
    let has_digit = password.chars().any(|c| c.is_ascii_digit() || ('۰'..='۹').contains(&c));
    if !has_letter || !has_digit {
        return Err(AppError::Validation("رمز عبور باید هم حرف داشته باشد و هم عدد.".into()));
    }
    Ok(())
}

/// `CA-<jalali year>-<5-digit sequence>`, e.g. `CA-1405-00012`.
pub fn student_id(today: chrono::NaiveDate, number: i64) -> String {
    let (jy, _, _) = jalali::from_gregorian(today);
    format!("CA-{jy}-{number:05}")
}

pub fn hash_token(token: &str) -> String {
    Sha256::digest(token.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

fn new_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

async fn hash_password(password: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default().hash_password(password.as_bytes(), &salt).map(|h| h.to_string())
    })
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .map_err(|e| AppError::Internal(anyhow::anyhow!("hashing failed: {e}")))
}

async fn verify_password(password: String, hash: String) -> AppResult<bool> {
    tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash).map(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok()).unwrap_or(false)
    })
    .await
    .map_err(|e| AppError::Internal(e.into()))
}

async fn issue_session(state: &AppState, user_id: i32) -> AppResult<IssuedSession> {
    let token = new_token();
    let expires = state.clock.now() + Duration::days(SESSION_DAYS);
    accounts::create_session(&state.pool, user_id, &hash_token(&token), expires).await?;
    Ok(IssuedSession { user_id, token })
}

pub async fn register(state: &AppState, req: RegisterRequest) -> AppResult<IssuedSession> {
    let name = normalize_name(&req.name)?;
    let email = normalize_email(&req.email)?;
    check_password_strength(&req.password)?;
    if accounts::email_taken(&state.pool, &email).await? {
        return Err(AppError::Conflict("با این ایمیل قبلاً ثبت‌نام شده است. وارد حسابت شو.".into()));
    }
    let hash = hash_password(req.password).await?;
    let now = state.clock.now();

    let mut tx = state.pool.begin().await?;
    let number = accounts::next_student_number(&mut *tx).await?;
    let sid = student_id(state.clock.today(), number);
    let user_id = accounts::create_user(&mut *tx, &name, &email, &hash, &sid, now)
        .await?
        .ok_or_else(|| AppError::Conflict("با این ایمیل قبلاً ثبت‌نام شده است. وارد حسابت شو.".into()))?;
    // Enroll the first-year courses that have no prerequisites.
    super::lessons::advance(&mut tx, user_id, now).await?;
    tx.commit().await?;

    issue_session(state, user_id).await
}

pub async fn login(state: &AppState, req: LoginRequest) -> AppResult<IssuedSession> {
    let invalid = || AppError::Unauthenticated("ایمیل یا رمز عبور نادرست است.".into());
    let Ok(email) = normalize_email(&req.email) else { return Err(invalid()) };
    let now = state.clock.now();
    let failures = accounts::recent_failures(&state.pool, &email, now - Duration::minutes(LOCKOUT_MINUTES)).await?;
    if failures >= MAX_FAILURES {
        return Err(AppError::TooManyRequests(format!(
            "تلاش‌های ناموفق زیاد بود. {} دقیقه‌ی دیگر دوباره امتحان کن.",
            fa::digits(LOCKOUT_MINUTES.to_string())
        )));
    }
    let creds = accounts::credentials_by_email(&state.pool, &email).await?;
    let hash = match &creds {
        Some(c) => c.password_hash.clone(),
        None => tokio::task::spawn_blocking(dummy_hash).await.map_err(|e| AppError::Internal(e.into()))?,
    };
    let ok = verify_password(req.password, hash).await? && creds.is_some();
    accounts::record_login(&state.pool, &email, ok, now).await?;
    match creds {
        Some(c) if ok => issue_session(state, c.id).await,
        _ => Err(invalid()),
    }
}

pub async fn logout(state: &AppState, token: &str) -> AppResult<()> {
    accounts::delete_session(&state.pool, &hash_token(token)).await?;
    Ok(())
}

pub async fn update_profile(state: &AppState, user_id: i32, req: ProfileRequest) -> AppResult<()> {
    let name = normalize_name(&req.name)?;
    accounts::set_name(&state.pool, user_id, &name).await?;
    Ok(())
}

/// Changes the password and signs out every other device.
pub async fn change_password(state: &AppState, user_id: i32, session_token: &str, req: PasswordRequest) -> AppResult<()> {
    check_password_strength(&req.new_password)?;
    let creds = accounts::credentials_by_id(&state.pool, user_id).await?.ok_or(AppError::Unauthorized)?;
    if !verify_password(req.current_password, creds.password_hash).await? {
        return Err(AppError::Validation("رمز عبور فعلی درست نیست.".into()));
    }
    let hash = hash_password(req.new_password).await?;
    accounts::set_password(&state.pool, user_id, &hash).await?;
    accounts::delete_other_sessions(&state.pool, user_id, &hash_token(session_token)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_rules() {
        assert_eq!(normalize_email("  Ali.Rezaei@Example.IR ").unwrap(), "ali.rezaei@example.ir");
        for bad in ["", "ali", "ali@", "@x.ir", "ali@x", "a b@x.ir", "a@@x.ir", "a@x.ir."] {
            assert!(normalize_email(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn name_rules() {
        assert_eq!(normalize_name("  سارا   محمدی ").unwrap(), "سارا محمدی");
        assert!(normalize_name("س").is_err());
        assert!(normalize_name("<script>").is_err());
    }

    #[test]
    fn password_rules() {
        assert!(check_password_strength("short1").is_err());
        assert!(check_password_strength("onlyletters").is_err());
        assert!(check_password_strength("12345678").is_err());
        assert!(check_password_strength("tehran1405").is_ok());
        assert!(check_password_strength("رمزعبور۱۲۳").is_ok());
    }

    #[test]
    fn student_id_uses_jalali_year() {
        let d = chrono::NaiveDate::from_ymd_opt(2026, 9, 28).unwrap();
        assert_eq!(student_id(d, 12), "CA-1405-00012");
    }

    #[test]
    fn token_hash_is_stable_hex() {
        assert_eq!(hash_token("abc").len(), 64);
        assert_eq!(hash_token("abc"), hash_token("abc"));
        assert_ne!(new_token(), new_token());
    }
}
