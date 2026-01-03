use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::models::signal::{
    ConfidenceFactors, CreateGithubSignal, CreateSignal, GithubSignal, Signal,
    SignalSource, SignalType,
};

// ============================================================================
// GitHub API Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GithubRepo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub description: Option<String>,
    pub stargazers_count: i32,
    pub forks_count: i32,
    pub open_issues_count: i32,
    pub pushed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub language: Option<String>,
    pub topics: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct GithubCommit {
    pub sha: String,
    pub commit: GithubCommitDetail,
}

#[derive(Debug, Deserialize)]
pub struct GithubCommitDetail {
    pub message: String,
    pub author: GithubCommitAuthor,
}

#[derive(Debug, Deserialize)]
pub struct GithubCommitAuthor {
    pub name: String,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct GithubRelease {
    pub id: i64,
    pub tag_name: String,
    pub name: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub html_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GithubContributor {
    pub login: String,
    pub contributions: i32,
}

// ============================================================================
// GitHub Activity Data (what we extract)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubActivityData {
    pub org_name: String,
    pub repos: Vec<RepoActivity>,
    pub total_commits_7d: i32,
    pub total_commits_30d: i32,
    pub total_stars: i32,
    pub stars_gained_7d: i32,
    pub active_repos: i32,
    pub latest_release: Option<ReleaseInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoActivity {
    pub name: String,
    pub url: String,
    pub stars: i32,
    pub forks: i32,
    pub commits_7d: i32,
    pub commits_30d: i32,
    pub last_commit: Option<DateTime<Utc>>,
    pub language: Option<String>,
    pub topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub repo: String,
    pub tag: String,
    pub name: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub url: String,
}

// ============================================================================
// GitHub Connector
// ============================================================================

pub struct GithubConnector {
    client: Client,
    token: Option<String>,
}

impl GithubConnector {
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            token,
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            "OutreachIQ/1.0".parse().unwrap(),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github.v3+json".parse().unwrap(),
        );
        if let Some(ref token) = self.token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token).parse().unwrap(),
            );
        }
        headers
    }

    /// Fetch activity data for a GitHub organization
    pub async fn fetch_org_activity(
        &self,
        org_name: &str,
    ) -> Result<GithubActivityData, Box<dyn std::error::Error + Send + Sync>> {
        info!("Fetching GitHub activity for org: {}", org_name);

        // Get organization repos
        let repos = self.fetch_org_repos(org_name).await?;
        
        let mut repo_activities = Vec::new();
        let mut total_commits_7d = 0;
        let mut total_commits_30d = 0;
        let mut total_stars = 0;
        let mut latest_release: Option<ReleaseInfo> = None;

        // Process top repos (limit to avoid rate limits)
        let top_repos: Vec<_> = repos.into_iter().take(10).collect();
        
        for repo in &top_repos {
            let commits_7d = self.count_recent_commits(&repo.full_name, 7).await.unwrap_or(0);
            let commits_30d = self.count_recent_commits(&repo.full_name, 30).await.unwrap_or(0);
            
            total_commits_7d += commits_7d;
            total_commits_30d += commits_30d;
            total_stars += repo.stargazers_count;

            // Check for releases
            if let Ok(Some(release)) = self.fetch_latest_release(&repo.full_name).await {
                if latest_release.is_none() 
                    || release.published_at > latest_release.as_ref().and_then(|r| r.published_at) 
                {
                    latest_release = Some(ReleaseInfo {
                        repo: repo.name.clone(),
                        tag: release.tag_name,
                        name: release.name,
                        published_at: release.published_at,
                        url: release.html_url,
                    });
                }
            }

            repo_activities.push(RepoActivity {
                name: repo.name.clone(),
                url: repo.html_url.clone(),
                stars: repo.stargazers_count,
                forks: repo.forks_count,
                commits_7d,
                commits_30d,
                last_commit: repo.pushed_at,
                language: repo.language.clone(),
                topics: repo.topics.clone().unwrap_or_default(),
            });
        }

        let active_repos = repo_activities.iter().filter(|r| r.commits_7d > 0).count() as i32;

        Ok(GithubActivityData {
            org_name: org_name.to_string(),
            repos: repo_activities,
            total_commits_7d,
            total_commits_30d,
            total_stars,
            stars_gained_7d: 0, // Would need historical data to calculate
            active_repos,
            latest_release,
        })
    }

    async fn fetch_org_repos(
        &self,
        org_name: &str,
    ) -> Result<Vec<GithubRepo>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.github.com/orgs/{}/repos?sort=pushed&per_page=30",
            org_name
        );

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("GitHub API error for {}: {} - {}", org_name, status, body);
            return Err(format!("GitHub API error: {}", status).into());
        }

        let repos: Vec<GithubRepo> = response.json().await?;
        Ok(repos)
    }

    async fn count_recent_commits(
        &self,
        repo_full_name: &str,
        days: i32,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let since = Utc::now() - chrono::Duration::days(days as i64);
        let url = format!(
            "https://api.github.com/repos/{}/commits?since={}&per_page=100",
            repo_full_name,
            since.to_rfc3339()
        );

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(0);
        }

        let commits: Vec<GithubCommit> = response.json().await?;
        Ok(commits.len() as i32)
    }

    async fn fetch_latest_release(
        &self,
        repo_full_name: &str,
    ) -> Result<Option<GithubRelease>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            repo_full_name
        );

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Ok(None);
        }

        let release: GithubRelease = response.json().await?;
        Ok(Some(release))
    }

    /// Calculate confidence score based on activity (rule-based, NOT AI)
    pub fn calculate_confidence(&self, activity: &GithubActivityData) -> ConfidenceFactors {
        let mut factors = ConfidenceFactors::new();

        // Factor 1: Recent commit activity
        if activity.total_commits_7d >= 50 {
            factors.add_factor("high_commit_activity", 0.25, "50+ commits in last 7 days");
        } else if activity.total_commits_7d >= 20 {
            factors.add_factor("medium_commit_activity", 0.15, "20+ commits in last 7 days");
        } else if activity.total_commits_7d >= 5 {
            factors.add_factor("low_commit_activity", 0.05, "5+ commits in last 7 days");
        }

        // Factor 2: Multiple active repos
        if activity.active_repos >= 5 {
            factors.add_factor("many_active_repos", 0.15, "5+ repos with recent commits");
        } else if activity.active_repos >= 2 {
            factors.add_factor("some_active_repos", 0.08, "2+ repos with recent commits");
        }

        // Factor 3: Recent release
        if let Some(ref release) = activity.latest_release {
            if let Some(published) = release.published_at {
                let days_ago = (Utc::now() - published).num_days();
                if days_ago <= 7 {
                    factors.add_factor("very_recent_release", 0.20, "Release in last 7 days");
                } else if days_ago <= 30 {
                    factors.add_factor("recent_release", 0.10, "Release in last 30 days");
                }
            }
        }

        // Factor 4: Star count (indicates project maturity)
        if activity.total_stars >= 10000 {
            factors.add_factor("high_stars", 0.10, "10k+ total stars");
        } else if activity.total_stars >= 1000 {
            factors.add_factor("medium_stars", 0.05, "1k+ total stars");
        }

        factors
    }

    /// Create a signal from GitHub activity data
    pub async fn create_signal(
        &self,
        pool: &sqlx::PgPool,
        company_id: uuid::Uuid,
        activity: &GithubActivityData,
    ) -> Result<Option<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        // Only create signal if there's meaningful activity
        if activity.total_commits_7d < 5 && activity.latest_release.is_none() {
            info!("Skipping signal for {} - insufficient activity", activity.org_name);
            return Ok(None);
        }

        let confidence = self.calculate_confidence(activity);
        
        // Build title based on activity
        let title = if let Some(ref release) = activity.latest_release {
            format!(
                "{} released {} ({})",
                activity.org_name,
                release.tag,
                release.repo
            )
        } else {
            format!(
                "{} has {} commits across {} repos this week",
                activity.org_name,
                activity.total_commits_7d,
                activity.active_repos
            )
        };

        // Check for duplicate
        if Signal::exists_duplicate(
            pool,
            company_id,
            SignalType::GithubActivity.as_str(),
            SignalSource::Github.as_str(),
            &title,
        )
        .await?
        {
            info!("Duplicate signal detected, skipping: {}", title);
            return Ok(None);
        }

        let create_signal = CreateSignal {
            company_id,
            signal_type: SignalType::GithubActivity,
            source: SignalSource::Github,
            title,
            description: Some(format!(
                "Active development: {} commits in 7 days, {} commits in 30 days. {} active repositories.",
                activity.total_commits_7d,
                activity.total_commits_30d,
                activity.active_repos
            )),
            source_url: Some(format!("https://github.com/{}", activity.org_name)),
            raw_data: serde_json::to_value(activity)?,
            confidence_score: confidence.final_score,
            confidence_factors: confidence.to_json(),
            signal_date: Some(Utc::now().date_naive()),
            expires_at: Some(Utc::now() + chrono::Duration::days(7)),
        };

        let signal = Signal::create(pool, create_signal).await?;

        // Create detailed GitHub signal record
        if let Some(ref main_repo) = activity.repos.first() {
            let github_detail = CreateGithubSignal {
                signal_id: signal.id,
                repo_name: main_repo.name.clone(),
                repo_url: Some(main_repo.url.clone()),
                commits_last_7d: activity.total_commits_7d,
                commits_last_30d: activity.total_commits_30d,
                stars_count: activity.total_stars,
                stars_gained_7d: activity.stars_gained_7d,
                forks_count: main_repo.forks,
                contributors_count: 0, // Would need separate API call
                open_issues: 0,
                last_commit_at: main_repo.last_commit,
                last_release_at: activity.latest_release.as_ref().and_then(|r| r.published_at),
                last_release_tag: activity.latest_release.as_ref().map(|r| r.tag.clone()),
            };

            GithubSignal::create(pool, github_detail).await?;
        }

        info!("Created GitHub signal for {}: {}", activity.org_name, signal.id);
        Ok(Some(signal))
    }
}

impl Default for GithubConnector {
    fn default() -> Self {
        Self::new(None)
    }
}
