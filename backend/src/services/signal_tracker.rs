use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::models::company::Company;
use crate::models::signal::{PublicSignal, Signal};
use crate::services::github_connector::GithubConnector;
use crate::services::wellfound_connector::WellfoundConnector;

// ============================================================================
// Signal Tracker: Orchestrates real signal ingestion (NO randomness, NO mocks)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompanySignalSummary {
    pub company_id: Uuid,
    pub company_name: String,
    pub domain: String,
    pub total_signals: i32,
    pub hiring_signals: i32,
    pub github_signals: i32,
    pub funding_signals: i32,
    pub latest_signal_date: Option<chrono::DateTime<chrono::Utc>>,
    pub avg_confidence: f64,
}

pub struct SignalTracker {
    github: GithubConnector,
    wellfound: WellfoundConnector,
}

impl SignalTracker {
    pub fn new(github_token: Option<String>) -> Self {
        Self {
            github: GithubConnector::new(github_token),
            wellfound: WellfoundConnector::new(),
        }
    }

    /// Ingest signals for a single company from all sources
    pub async fn ingest_company_signals(
        &self,
        pool: &PgPool,
        company: &Company,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        let mut all_signals = Vec::new();

        info!("Ingesting signals for company: {} ({})", company.name, company.domain);

        // 1. GitHub signals (if org is configured)
        if let Some(ref github_org) = company.github_org {
            match self.github.fetch_org_activity(github_org).await {
                Ok(activity) => {
                    if let Ok(Some(signal)) = self.github.create_signal(pool, company.id, &activity).await {
                        all_signals.push(signal);
                    }
                }
                Err(e) => {
                    warn!("GitHub fetch failed for {}: {}", github_org, e);
                }
            }
        }

        // 2. Wellfound hiring signals (if slug is configured)
        if let Some(ref wellfound_slug) = company.wellfound_slug {
            match self.wellfound.fetch_company_jobs(wellfound_slug).await {
                Ok(jobs) => {
                    match self.wellfound.create_signals(pool, company.id, &jobs).await {
                        Ok(signals) => all_signals.extend(signals),
                        Err(e) => warn!("Failed to create hiring signals for {}: {}", wellfound_slug, e),
                    }
                }
                Err(e) => {
                    warn!("Wellfound fetch failed for {}: {}", wellfound_slug, e);
                }
            }
        }

        // Update company's last_scraped_at
        if let Err(e) = Company::update_last_scraped(pool, company.id).await {
            warn!("Failed to update last_scraped_at for {}: {}", company.name, e);
        }

        info!(
            "Ingested {} signals for {} ({})",
            all_signals.len(),
            company.name,
            company.domain
        );

        Ok(all_signals)
    }

    /// Ingest signals for all active companies
    pub async fn ingest_all_signals(
        &self,
        pool: &PgPool,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        let companies = Company::find_active(pool).await?;
        let mut all_signals = Vec::new();

        info!("Starting signal ingestion for {} companies", companies.len());

        for company in companies {
            match self.ingest_company_signals(pool, &company).await {
                Ok(signals) => all_signals.extend(signals),
                Err(e) => {
                    error!("Failed to ingest signals for {}: {}", company.name, e);
                }
            }

            // Rate limiting: small delay between companies
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        info!("Signal ingestion complete: {} total signals", all_signals.len());
        Ok(all_signals)
    }

    /// Ingest signals for companies that need updating (stale data)
    pub async fn ingest_stale_signals(
        &self,
        pool: &PgPool,
        source: &str,
        stale_hours: i32,
    ) -> Result<Vec<Signal>, Box<dyn std::error::Error + Send + Sync>> {
        let companies = Company::find_needing_scrape(pool, source, stale_hours).await?;
        let mut all_signals = Vec::new();

        info!(
            "Found {} companies needing {} signal refresh (stale > {} hours)",
            companies.len(),
            source,
            stale_hours
        );

        for company in companies {
            match self.ingest_company_signals(pool, &company).await {
                Ok(signals) => all_signals.extend(signals),
                Err(e) => {
                    error!("Failed to ingest signals for {}: {}", company.name, e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        Ok(all_signals)
    }

    /// Get the public signal feed
    pub async fn get_public_feed(
        &self,
        pool: &PgPool,
        limit: i64,
    ) -> Result<Vec<PublicSignal>, Box<dyn std::error::Error + Send + Sync>> {
        let signals = Signal::find_recent(pool, limit).await?;
        Ok(signals)
    }

    /// Get signals by type
    pub async fn get_signals_by_type(
        &self,
        pool: &PgPool,
        signal_type: &str,
        limit: i64,
    ) -> Result<Vec<PublicSignal>, Box<dyn std::error::Error + Send + Sync>> {
        let signals = Signal::find_by_type(pool, signal_type, limit).await?;
        Ok(signals)
    }

    /// Get signal summary for a company
    pub async fn get_company_summary(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Option<CompanySignalSummary>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query_as::<_, (String, String, i64, i64, i64, i64, Option<chrono::DateTime<chrono::Utc>>, Option<f64>)>(
            r#"
            SELECT 
                c.name,
                c.domain,
                COUNT(s.id) as total_signals,
                COUNT(CASE WHEN s.signal_type = 'hiring' THEN 1 END) as hiring_signals,
                COUNT(CASE WHEN s.signal_type = 'github_activity' THEN 1 END) as github_signals,
                COUNT(CASE WHEN s.signal_type = 'funding' THEN 1 END) as funding_signals,
                MAX(s.detected_at) as latest_signal_date,
                AVG(s.confidence_score::float) as avg_confidence
            FROM companies c
            LEFT JOIN signals s ON s.company_id = c.id
            WHERE c.id = $1
            GROUP BY c.id, c.name, c.domain
            "#,
        )
        .bind(company_id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|(name, domain, total, hiring, github, funding, latest, avg_conf)| {
            CompanySignalSummary {
                company_id,
                company_name: name,
                domain,
                total_signals: total as i32,
                hiring_signals: hiring as i32,
                github_signals: github as i32,
                funding_signals: funding as i32,
                latest_signal_date: latest,
                avg_confidence: avg_conf.unwrap_or(0.0),
            }
        }))
    }
}

impl Default for SignalTracker {
    fn default() -> Self {
        Self::new(None)
    }
}
