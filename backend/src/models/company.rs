use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    pub domain: String,
    pub logo_url: Option<String>,
    pub description: Option<String>,
    pub industry: Option<String>,
    pub employee_count_range: Option<String>,
    pub founded_year: Option<i32>,
    pub headquarters: Option<String>,
    pub website_url: Option<String>,
    pub github_org: Option<String>,
    pub twitter_handle: Option<String>,
    pub linkedin_url: Option<String>,
    pub wellfound_slug: Option<String>,
    pub is_active: bool,
    pub last_scraped_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCompany {
    pub name: String,
    pub domain: String,
    pub logo_url: Option<String>,
    pub description: Option<String>,
    pub industry: Option<String>,
    pub github_org: Option<String>,
    pub twitter_handle: Option<String>,
    pub wellfound_slug: Option<String>,
}

impl Company {
    pub async fn find_by_domain(
        pool: &sqlx::PgPool,
        domain: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM companies WHERE domain = $1")
            .bind(domain)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_github_org(
        pool: &sqlx::PgPool,
        github_org: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM companies WHERE github_org = $1")
            .bind(github_org)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_active(pool: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM companies WHERE is_active = TRUE ORDER BY name",
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_needing_scrape(
        pool: &sqlx::PgPool,
        source: &str,
        stale_hours: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT c.* FROM companies c
            LEFT JOIN scraper_state ss ON ss.company_id = c.id AND ss.source = $1
            WHERE c.is_active = TRUE
              AND (ss.last_successful_at IS NULL 
                   OR ss.last_successful_at < NOW() - INTERVAL '1 hour' * $2)
            ORDER BY ss.last_successful_at NULLS FIRST
            LIMIT 50
            "#,
        )
        .bind(source)
        .bind(stale_hours)
        .fetch_all(pool)
        .await
    }

    pub async fn update_last_scraped(
        pool: &sqlx::PgPool,
        id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE companies SET last_scraped_at = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
