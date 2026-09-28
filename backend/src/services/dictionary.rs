use crate::error::{AppError, AppResult};
use crate::repo::content::{self, TermRow};
use crate::state::AppState;
use serde::Serialize;

pub const MAX_QUERY_CHARS: usize = 64;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedTerm {
    pub slug: String,
    pub name_en: String,
    pub name_fa: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TermDto {
    pub slug: String,
    pub name_en: String,
    pub name_fa: String,
    pub category: String,
    /// Persian name of the department the term belongs to.
    pub category_name: String,
    pub definition: String,
    pub example: String,
    pub related: Vec<RelatedTerm>,
}

/// Validates and normalizes a search query. Empty means "no filter".
pub fn normalize_query(q: Option<&str>) -> AppResult<Option<String>> {
    let Some(q) = q.map(str::trim).filter(|q| !q.is_empty()) else { return Ok(None) };
    if q.chars().count() > MAX_QUERY_CHARS {
        return Err(AppError::Validation("عبارت جست‌وجو بیش از حد طولانی است.".into()));
    }
    Ok(Some(q.to_string()))
}

fn to_dto(t: TermRow, all: &[TermRow], categories: &[(String, String)]) -> TermDto {
    let category_name = categories.iter().find(|(slug, _)| *slug == t.category).map(|(_, n)| n.clone()).unwrap_or_default();
    let related = t
        .related
        .iter()
        .filter_map(|slug| all.iter().find(|x| &x.slug == slug))
        .map(|x| RelatedTerm { slug: x.slug.clone(), name_en: x.name_en.clone(), name_fa: x.name_fa.clone() })
        .collect();
    TermDto { slug: t.slug, name_en: t.name_en, name_fa: t.name_fa, category: t.category, category_name, definition: t.definition, example: t.example, related }
}

async fn categories(conn: &mut sqlx::PgConnection) -> AppResult<Vec<(String, String)>> {
    Ok(crate::repo::catalog::departments(&mut *conn).await?.into_iter().map(|d| (d.slug, d.name_fa)).collect())
}

pub async fn search(state: &AppState, q: Option<&str>) -> AppResult<Vec<TermDto>> {
    let q = normalize_query(q)?;
    let mut conn = state.pool.acquire().await?;
    let all = content::search_terms(&mut *conn, None).await?;
    let categories = categories(&mut conn).await?;
    let hits = if q.is_some() { content::search_terms(&mut *conn, q.as_deref()).await? } else { all.clone() };
    Ok(hits.into_iter().map(|t| to_dto(t, &all, &categories)).collect())
}

pub async fn term(state: &AppState, slug: &str) -> AppResult<TermDto> {
    let mut conn = state.pool.acquire().await?;
    let t = content::term(&mut *conn, slug).await?.ok_or(AppError::NotFound("اصطلاح"))?;
    let all = content::search_terms(&mut *conn, None).await?;
    let categories = categories(&mut conn).await?;
    Ok(to_dto(t, &all, &categories))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_normalization() {
        assert_eq!(normalize_query(None).unwrap(), None);
        assert_eq!(normalize_query(Some("   ")).unwrap(), None);
        assert_eq!(normalize_query(Some(" Stream ")).unwrap().as_deref(), Some("Stream"));
        assert!(normalize_query(Some(&"x".repeat(65))).is_err());
    }
}
