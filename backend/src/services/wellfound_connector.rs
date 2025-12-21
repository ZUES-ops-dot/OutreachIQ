use chrono::{NaiveDate, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::models::signal::{
    ConfidenceFactors, CreateHiringSignal, CreateSignal, HiringSignal, Signal,
    SignalSource, SignalType,
};

// ============================================================================
// Web3 Keywords for Role Detection
// ============================================================================

const WEB3_KEYWORDS: &[&str] = &[
    "solidity", "rust", "blockchain", "web3", "defi", "nft", "crypto",
    "ethereum", "smart contract", "dapp", "dao", "token", "wallet",
    "layer 2", "l2", "zk", "zero knowledge", "rollup", "bridge",
    "staking", "yield", "liquidity", "amm", "dex", "protocol",
    "on-chain", "off-chain", "consensus", "validator", "node",
];

const SENIOR_KEYWORDS: &[&str] = &[
    "senior", "lead", "principal", "staff", "head of", "director",
    "vp", "architect", "manager",
];

// ============================================================================
// Job Posting Data (what we extract from Wellfound)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPosting {
    pub id: String,
    pub title: String,
    pub company_name: String,
    pub company_slug: String,
    pub location: Option<String>,
    pub job_type: Option<String>,
    pub salary_range: Option<String>,
    pub posted_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub url: String,
    pub keywords: Vec<String>,
    pub is_web3_role: bool,
    pub experience_level: Option<String>,
    pub department: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyJobs {
    pub company_name: String,
    pub company_slug: String,
    pub total_jobs: i32,
    pub web3_jobs: i32,
    pub jobs: Vec<JobPosting>,
    pub scraped_at: chrono::DateTime<Utc>,
}

// ============================================================================
// Wellfound Connector
// ============================================================================

pub struct WellfoundConnector {
    client: Client,
}

impl WellfoundConnector {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Fetch job postings for a company from Wellfound
    /// Note: This is a simplified implementation. In production, you'd need
    /// to handle Wellfound's actual HTML structure or use their API if available.
    pub async fn fetch_company_jobs(
        &self,
        company_slug: &str,
    ) -> Result<CompanyJobs, Box<dyn std::error::Error + Send + Sync>> {
        info!("Fetching Wellfound jobs for: {}", company_slug);

        let url = format!("https://wellfound.com/company/{}/jobs", company_slug);
        
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            warn!("Wellfound returned {} for {}", response.status(), company_slug);
            return Err(format!("Wellfound error: {}", response.status()).into());
        }

        let html = response.text().await?;
        
        // Parse jobs from HTML
        let jobs = self.parse_jobs_from_html(&html, company_slug)?;
        
        let web3_jobs = jobs.iter().filter(|j| j.is_web3_role).count() as i32;

        Ok(CompanyJobs {
            company_name: company_slug.replace("-", " "),
            company_slug: company_slug.to_string(),
            total_jobs: jobs.len() as i32,
            web3_jobs,
            jobs,
            scraped_at: Utc::now(),
        })
    }

    /// Parse job postings from Wellfound HTML
    /// This is a simplified parser - in production you'd use a proper HTML parser
    fn parse_jobs_from_html(
        &self,
        html: &str,
        company_slug: &str,
    ) -> Result<Vec<JobPosting>, Box<dyn std::error::Error + Send + Sync>> {
        let mut jobs = Vec::new();

        // Look for job titles in the HTML
        // Wellfound typically has job listings in specific div structures
        // This is a heuristic approach - would need refinement based on actual HTML

        // Simple regex-like extraction for job titles
        // In production, use scraper or select.rs crate
        let title_patterns = [
            "Senior Solidity Developer",
            "Blockchain Engineer",
            "Smart Contract Developer",
            "Protocol Engineer",
            "DeFi Developer",
            "Web3 Frontend Engineer",
            "Rust Engineer",
            "Backend Engineer",
            "Full Stack Developer",
            "DevOps Engineer",
            "Product Manager",
            "Engineering Manager",
        ];

        // Check if any job-related content exists
        let has_jobs = html.contains("job") || html.contains("career") || html.contains("position");
        
        if !has_jobs {
            return Ok(jobs);
        }

        // For MVP: Create jobs based on detected patterns in HTML
        for (idx, pattern) in title_patterns.iter().enumerate() {
            if html.to_lowercase().contains(&pattern.to_lowercase()) {
                let job = self.create_job_from_title(pattern, company_slug, idx);
                jobs.push(job);
            }
        }

        Ok(jobs)
    }

    fn create_job_from_title(&self, title: &str, company_slug: &str, idx: usize) -> JobPosting {
        let title_lower = title.to_lowercase();
        
        // Detect if it's a Web3 role
        let is_web3 = WEB3_KEYWORDS.iter().any(|kw| title_lower.contains(kw));
        
        // Detect experience level
        let experience_level = if SENIOR_KEYWORDS.iter().any(|kw| title_lower.contains(kw)) {
            Some("senior".to_string())
        } else if title_lower.contains("junior") || title_lower.contains("entry") {
            Some("junior".to_string())
        } else {
            Some("mid".to_string())
        };

        // Extract keywords
        let keywords: Vec<String> = WEB3_KEYWORDS
            .iter()
            .filter(|kw| title_lower.contains(*kw))
            .map(|s| s.to_string())
            .collect();

        // Detect department
        let department = if title_lower.contains("engineer") || title_lower.contains("developer") {
            Some("Engineering".to_string())
        } else if title_lower.contains("product") {
            Some("Product".to_string())
        } else if title_lower.contains("design") {
            Some("Design".to_string())
        } else if title_lower.contains("marketing") {
            Some("Marketing".to_string())
        } else {
            None
        };

        JobPosting {
            id: format!("{}-{}", company_slug, idx),
            title: title.to_string(),
            company_name: company_slug.replace("-", " "),
            company_slug: company_slug.to_string(),
            location: Some("Remote".to_string()),
            job_type: Some("Full-time".to_string()),
            salary_range: None,
            posted_date: Some(Utc::now().date_naive()),
            description: None,
            url: format!("https://wellfound.com/company/{}/jobs", company_slug),
            keywords,
            is_web3_role: is_web3,
            experience_level,
            department,
        }
    }

    /// Calculate confidence score for hiring signals (rule-based, NOT AI)
    pub fn calculate_confidence(&self, company_jobs: &CompanyJobs) -> ConfidenceFactors {
        let mut factors = ConfidenceFactors::new();

        // Factor 1: Number of open roles
        if company_jobs.total_jobs >= 10 {
            factors.add_factor("many_open_roles", 0.20, "10+ open positions");
        } else if company_jobs.total_jobs >= 5 {
            factors.add_factor("several_open_roles", 0.15, "5+ open positions");
        } else if company_jobs.total_jobs >= 3 {
            factors.add_factor("some_open_roles", 0.10, "3+ open positions");
        }

        // Factor 2: Web3-specific roles
        if company_jobs.web3_jobs >= 3 {
            factors.add_factor("many_web3_roles", 0.20, "3+ Web3-specific roles");
        } else if company_jobs.web3_jobs >= 1 {
            factors.add_factor("has_web3_roles", 0.10, "Has Web3-specific roles");
        }

        // Factor 3: Recent postings
        let recent_jobs = company_jobs.jobs.iter().filter(|j| {
            j.posted_date.map(|d| {
                let days_ago = (Utc::now().date_naive() - d).num_days();
                days_ago <= 7
            }).unwrap_or(false)
        }).count();

        if recent_jobs >= 3 {
            factors.add_factor("very_recent_postings", 0.15, "3+ jobs posted in last 7 days");
        } else if recent_jobs >= 1 {
            factors.add_factor("recent_postings", 0.08, "Jobs posted in last 7 days");
        }

        // Factor 4: Senior roles (indicates growth/investment)
        let senior_roles = company_jobs.jobs.iter().filter(|j| {
            j.experience_level.as_ref().map(|l| l == "senior").unwrap_or(false)
        }).count();

        if senior_roles >= 2 {
            factors.add_factor("hiring_senior", 0.10, "Hiring multiple senior roles");
        }

        factors
    }

    /// Create signals from job postings
    pub async fn create_signals(
        &self,
        pool: &sqlx::PgPool,
        company_id: uuid::Uuid,
        company_jobs: &CompanyJobs,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        let mut signals = Vec::new();

        if company_jobs.total_jobs == 0 {
            info!("No jobs found for {}, skipping signal creation", company_jobs.company_slug);
            return Ok(signals);
        }

        let confidence = self.calculate_confidence(company_jobs);

        // Create one aggregate hiring signal per company
        let title = if company_jobs.web3_jobs > 0 {
            format!(
                "{} is hiring {} roles ({} Web3-specific)",
                company_jobs.company_name,
                company_jobs.total_jobs,
                company_jobs.web3_jobs
            )
        } else {
            format!(
                "{} is hiring {} roles",
                company_jobs.company_name,
                company_jobs.total_jobs
            )
        };

        // Check for duplicate
        if Signal::exists_duplicate(
            pool,
            company_id,
            SignalType::Hiring.as_str(),
            SignalSource::Wellfound.as_str(),
            &title,
        )
        .await?
        {
            info!("Duplicate hiring signal detected, skipping: {}", title);
            return Ok(signals);
        }

        // Build description with job titles
        let job_titles: Vec<String> = company_jobs.jobs.iter().take(5).map(|j| j.title.clone()).collect();
        let description = format!(
            "Open positions include: {}{}",
            job_titles.join(", "),
            if company_jobs.jobs.len() > 5 { " and more..." } else { "" }
        );

        let create_signal = CreateSignal {
            company_id,
            signal_type: SignalType::Hiring,
            source: SignalSource::Wellfound,
            title,
            description: Some(description),
            source_url: Some(format!(
                "https://wellfound.com/company/{}/jobs",
                company_jobs.company_slug
            )),
            raw_data: serde_json::to_value(company_jobs)?,
            confidence_score: confidence.final_score,
            confidence_factors: confidence.to_json(),
            signal_date: Some(Utc::now().date_naive()),
            expires_at: Some(Utc::now() + chrono::Duration::days(14)),
        };

        let signal = Signal::create(pool, create_signal).await?;

        // Create detailed hiring signal records for each job
        for job in &company_jobs.jobs {
            let hiring_detail = CreateHiringSignal {
                signal_id: signal.id,
                job_title: job.title.clone(),
                department: job.department.clone(),
                location: job.location.clone(),
                job_type: job.job_type.clone(),
                experience_level: job.experience_level.clone(),
                salary_range: job.salary_range.clone(),
                keywords: job.keywords.clone(),
                is_web3_role: job.is_web3_role,
                posted_date: job.posted_date,
                source_job_id: Some(job.id.clone()),
            };

            HiringSignal::create(pool, hiring_detail).await?;
        }

        info!(
            "Created hiring signal for {}: {} ({} jobs)",
            company_jobs.company_name, signal.id, company_jobs.total_jobs
        );
        
        signals.push(signal);
        Ok(signals)
    }
}

impl Default for WellfoundConnector {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Manual Job Entry (for bootstrapping without scraping)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualJobEntry {
    pub company_name: String,
    pub company_slug: String,
    pub jobs: Vec<ManualJob>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualJob {
    pub title: String,
    pub department: Option<String>,
    pub location: Option<String>,
    pub url: Option<String>,
}

impl WellfoundConnector {
    /// Create signals from manually entered job data
    /// Use this to bootstrap the system before scraping is fully working
    pub async fn create_signals_from_manual(
        &self,
        pool: &sqlx::PgPool,
        company_id: uuid::Uuid,
        entry: &ManualJobEntry,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        // Convert manual entries to JobPosting format
        let jobs: Vec<JobPosting> = entry.jobs.iter().enumerate().map(|(idx, job)| {
            self.create_job_from_title(&job.title, &entry.company_slug, idx)
        }).collect();

        let web3_jobs = jobs.iter().filter(|j| j.is_web3_role).count() as i32;

        let company_jobs = CompanyJobs {
            company_name: entry.company_name.clone(),
            company_slug: entry.company_slug.clone(),
            total_jobs: jobs.len() as i32,
            web3_jobs,
            jobs,
            scraped_at: Utc::now(),
        };

        self.create_signals(pool, company_id, &company_jobs).await
    }
}
