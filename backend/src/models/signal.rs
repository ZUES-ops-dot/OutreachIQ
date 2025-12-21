use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// Signal Types (matching DB enum)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    Hiring,
    Funding,
    GithubActivity,
    TechAdoption,
    Expansion,
    ProductLaunch,
}

impl SignalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalType::Hiring => "hiring",
            SignalType::Funding => "funding",
            SignalType::GithubActivity => "github_activity",
            SignalType::TechAdoption => "tech_adoption",
            SignalType::Expansion => "expansion",
            SignalType::ProductLaunch => "product_launch",
        }
    }
}

impl std::fmt::Display for SignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Signal Sources (matching DB enum)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignalSource {
    Wellfound,
    Github,
    RssFeed,
    Twitter,
    Linkedin,
    Crunchbase,
    Manual,
}

impl SignalSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalSource::Wellfound => "wellfound",
            SignalSource::Github => "github",
            SignalSource::RssFeed => "rss_feed",
            SignalSource::Twitter => "twitter",
            SignalSource::Linkedin => "linkedin",
            SignalSource::Crunchbase => "crunchbase",
            SignalSource::Manual => "manual",
        }
    }
}

impl std::fmt::Display for SignalSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Main Signal Model
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Signal {
    pub id: Uuid,
    pub company_id: Uuid,
    pub signal_type: String,
    pub source: String,
    pub title: String,
    pub description: Option<String>,
    pub source_url: Option<String>,
    pub raw_data: serde_json::Value,
    pub confidence_score: rust_decimal::Decimal,
    pub confidence_factors: serde_json::Value,
    pub detected_at: DateTime<Utc>,
    pub signal_date: Option<NaiveDate>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_verified: bool,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSignal {
    pub company_id: Uuid,
    pub signal_type: SignalType,
    pub source: SignalSource,
    pub title: String,
    pub description: Option<String>,
    pub source_url: Option<String>,
    pub raw_data: serde_json::Value,
    pub confidence_score: f64,
    pub confidence_factors: serde_json::Value,
    pub signal_date: Option<NaiveDate>,
    pub expires_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Public Feed Signal (for API responses)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PublicSignal {
    pub id: Uuid,
    pub signal_type: String,
    pub source: String,
    pub title: String,
    pub description: Option<String>,
    pub source_url: Option<String>,
    pub confidence_score: rust_decimal::Decimal,
    pub detected_at: DateTime<Utc>,
    pub signal_date: Option<NaiveDate>,
    pub company_id: Uuid,
    pub company_name: String,
    pub company_domain: String,
    pub company_logo: Option<String>,
    pub industry: Option<String>,
}

// ============================================================================
// Hiring Signal Detail
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HiringSignal {
    pub id: Uuid,
    pub signal_id: Uuid,
    pub job_title: String,
    pub department: Option<String>,
    pub location: Option<String>,
    pub job_type: Option<String>,
    pub experience_level: Option<String>,
    pub salary_range: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub is_web3_role: bool,
    pub posted_date: Option<NaiveDate>,
    pub source_job_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateHiringSignal {
    pub signal_id: Uuid,
    pub job_title: String,
    pub department: Option<String>,
    pub location: Option<String>,
    pub job_type: Option<String>,
    pub experience_level: Option<String>,
    pub salary_range: Option<String>,
    pub keywords: Vec<String>,
    pub is_web3_role: bool,
    pub posted_date: Option<NaiveDate>,
    pub source_job_id: Option<String>,
}

// ============================================================================
// GitHub Signal Detail
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GithubSignal {
    pub id: Uuid,
    pub signal_id: Uuid,
    pub repo_name: String,
    pub repo_url: Option<String>,
    pub commits_last_7d: i32,
    pub commits_last_30d: i32,
    pub stars_count: i32,
    pub stars_gained_7d: i32,
    pub forks_count: i32,
    pub contributors_count: i32,
    pub open_issues: i32,
    pub last_commit_at: Option<DateTime<Utc>>,
    pub last_release_at: Option<DateTime<Utc>>,
    pub last_release_tag: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGithubSignal {
    pub signal_id: Uuid,
    pub repo_name: String,
    pub repo_url: Option<String>,
    pub commits_last_7d: i32,
    pub commits_last_30d: i32,
    pub stars_count: i32,
    pub stars_gained_7d: i32,
    pub forks_count: i32,
    pub contributors_count: i32,
    pub open_issues: i32,
    pub last_commit_at: Option<DateTime<Utc>>,
    pub last_release_at: Option<DateTime<Utc>>,
    pub last_release_tag: Option<String>,
}

// ============================================================================
// Funding Signal Detail
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FundingSignal {
    pub id: Uuid,
    pub signal_id: Uuid,
    pub round_type: Option<String>,
    pub amount_usd: Option<i64>,
    pub amount_display: Option<String>,
    pub investors: Option<Vec<String>>,
    pub lead_investor: Option<String>,
    pub announced_date: Option<NaiveDate>,
    pub source_article_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Confidence Scoring (Rule-Based, NOT AI)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfidenceFactors {
    pub base_score: f64,
    pub factors: Vec<ConfidenceFactor>,
    pub final_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceFactor {
    pub name: String,
    pub value: f64,
    pub reason: String,
}

impl ConfidenceFactors {
    pub fn new() -> Self {
        Self {
            base_score: 0.5,
            factors: Vec::new(),
            final_score: 0.5,
        }
    }

    pub fn add_factor(&mut self, name: &str, value: f64, reason: &str) {
        self.factors.push(ConfidenceFactor {
            name: name.to_string(),
            value,
            reason: reason.to_string(),
        });
        self.recalculate();
    }

    fn recalculate(&mut self) {
        let total_adjustment: f64 = self.factors.iter().map(|f| f.value).sum();
        self.final_score = (self.base_score + total_adjustment).clamp(0.0, 1.0);
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

// ============================================================================
// Database Operations
// ============================================================================

impl Signal {
    pub async fn create(
        pool: &sqlx::PgPool,
        signal: CreateSignal,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO signals (
                company_id, signal_type, source, title, description,
                source_url, raw_data, confidence_score, confidence_factors,
                signal_date, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(signal.company_id)
        .bind(signal.signal_type.as_str())
        .bind(signal.source.as_str())
        .bind(&signal.title)
        .bind(&signal.description)
        .bind(&signal.source_url)
        .bind(&signal.raw_data)
        .bind(signal.confidence_score)
        .bind(&signal.confidence_factors)
        .bind(signal.signal_date)
        .bind(signal.expires_at)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM signals WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_recent(
        pool: &sqlx::PgPool,
        limit: i64,
    ) -> Result<Vec<PublicSignal>, sqlx::Error> {
        sqlx::query_as::<_, PublicSignal>(
            r#"
            SELECT * FROM public_signal_feed
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_company(
        pool: &sqlx::PgPool,
        company_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM signals WHERE company_id = $1 ORDER BY detected_at DESC",
        )
        .bind(company_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_type(
        pool: &sqlx::PgPool,
        signal_type: &str,
        limit: i64,
    ) -> Result<Vec<PublicSignal>, sqlx::Error> {
        sqlx::query_as::<_, PublicSignal>(
            r#"
            SELECT * FROM public_signal_feed
            WHERE signal_type = $1
            LIMIT $2
            "#,
        )
        .bind(signal_type)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn exists_duplicate(
        pool: &sqlx::PgPool,
        company_id: Uuid,
        signal_type: &str,
        source: &str,
        title: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM signals
            WHERE company_id = $1 
              AND signal_type = $2 
              AND source = $3 
              AND title = $4
              AND detected_at > NOW() - INTERVAL '7 days'
            "#,
        )
        .bind(company_id)
        .bind(signal_type)
        .bind(source)
        .bind(title)
        .fetch_one(pool)
        .await?;

        Ok(result > 0)
    }
}

impl HiringSignal {
    pub async fn create(
        pool: &sqlx::PgPool,
        signal: CreateHiringSignal,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO hiring_signals (
                signal_id, job_title, department, location, job_type,
                experience_level, salary_range, keywords, is_web3_role,
                posted_date, source_job_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(signal.signal_id)
        .bind(&signal.job_title)
        .bind(&signal.department)
        .bind(&signal.location)
        .bind(&signal.job_type)
        .bind(&signal.experience_level)
        .bind(&signal.salary_range)
        .bind(&signal.keywords)
        .bind(signal.is_web3_role)
        .bind(signal.posted_date)
        .bind(&signal.source_job_id)
        .fetch_one(pool)
        .await
    }
}

impl GithubSignal {
    pub async fn create(
        pool: &sqlx::PgPool,
        signal: CreateGithubSignal,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO github_signals (
                signal_id, repo_name, repo_url, commits_last_7d, commits_last_30d,
                stars_count, stars_gained_7d, forks_count, contributors_count,
                open_issues, last_commit_at, last_release_at, last_release_tag
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(signal.signal_id)
        .bind(&signal.repo_name)
        .bind(&signal.repo_url)
        .bind(signal.commits_last_7d)
        .bind(signal.commits_last_30d)
        .bind(signal.stars_count)
        .bind(signal.stars_gained_7d)
        .bind(signal.forks_count)
        .bind(signal.contributors_count)
        .bind(signal.open_issues)
        .bind(signal.last_commit_at)
        .bind(signal.last_release_at)
        .bind(&signal.last_release_tag)
        .fetch_one(pool)
        .await
    }
}
