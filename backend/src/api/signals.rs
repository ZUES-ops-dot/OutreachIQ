use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::company::Company;
use crate::models::signal::{PublicSignal, Signal};
use crate::services::signal_tracker::SignalTracker;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    pub limit: Option<i64>,
    pub signal_type: Option<String>,
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct SignalFeedResponse {
    pub signals: Vec<PublicSignal>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct CompanyListResponse {
    pub companies: Vec<CompanyInfo>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct CompanyInfo {
    pub id: Uuid,
    pub name: String,
    pub domain: String,
    pub industry: Option<String>,
    pub github_org: Option<String>,
    pub twitter_handle: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IngestResponse {
    pub success: bool,
    pub signals_created: usize,
    pub message: String,
}

// ============================================================================
// Public Endpoints (no auth required)
// ============================================================================

/// GET /api/signals/feed - Public signal feed
pub async fn get_signal_feed(
    pool: web::Data<PgPool>,
    query: web::Query<FeedQuery>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(50).min(100);

    let signals = if let Some(ref signal_type) = query.signal_type {
        Signal::find_by_type(pool.get_ref(), signal_type, limit).await
    } else {
        Signal::find_recent(pool.get_ref(), limit).await
    };

    match signals {
        Ok(signals) => {
            let total = signals.len();
            HttpResponse::Ok().json(SignalFeedResponse { signals, total })
        }
        Err(e) => {
            tracing::error!("Failed to fetch signal feed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch signals"
            }))
        }
    }
}

/// GET /api/signals/companies - List tracked companies
pub async fn get_companies(pool: web::Data<PgPool>) -> impl Responder {
    match Company::find_active(pool.get_ref()).await {
        Ok(companies) => {
            let company_infos: Vec<CompanyInfo> = companies
                .into_iter()
                .map(|c| CompanyInfo {
                    id: c.id,
                    name: c.name,
                    domain: c.domain,
                    industry: c.industry,
                    github_org: c.github_org,
                    twitter_handle: c.twitter_handle,
                })
                .collect();
            let total = company_infos.len();
            HttpResponse::Ok().json(CompanyListResponse {
                companies: company_infos,
                total,
            })
        }
        Err(e) => {
            tracing::error!("Failed to fetch companies: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch companies"
            }))
        }
    }
}

/// GET /api/signals/company/{id} - Get signals for a specific company
pub async fn get_company_signals(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let company_id = path.into_inner();

    match Signal::find_by_company(pool.get_ref(), company_id).await {
        Ok(signals) => HttpResponse::Ok().json(signals),
        Err(e) => {
            tracing::error!("Failed to fetch company signals: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch company signals"
            }))
        }
    }
}

/// GET /api/signals/stats - Get signal statistics
pub async fn get_signal_stats(pool: web::Data<PgPool>) -> impl Responder {
    let stats = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
        r#"
        SELECT 
            COUNT(*) as total_signals,
            COUNT(DISTINCT company_id) as companies_with_signals,
            COUNT(CASE WHEN signal_type = 'hiring' THEN 1 END) as hiring_signals,
            COUNT(CASE WHEN signal_type = 'github_activity' THEN 1 END) as github_signals,
            COUNT(CASE WHEN detected_at > NOW() - INTERVAL '24 hours' THEN 1 END) as signals_24h
        FROM signals
        WHERE is_published = TRUE
        "#,
    )
    .fetch_one(pool.get_ref())
    .await;

    match stats {
        Ok((total, companies, hiring, github, recent)) => {
            HttpResponse::Ok().json(serde_json::json!({
                "total_signals": total,
                "companies_with_signals": companies,
                "hiring_signals": hiring,
                "github_signals": github,
                "signals_last_24h": recent
            }))
        }
        Err(e) => {
            tracing::error!("Failed to fetch signal stats: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch stats"
            }))
        }
    }
}

// ============================================================================
// Admin Endpoints (auth required in production)
// ============================================================================

/// POST /api/signals/ingest - Trigger signal ingestion for all companies
pub async fn trigger_ingest(pool: web::Data<PgPool>) -> impl Responder {
    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let tracker = SignalTracker::new(github_token);

    match tracker.ingest_all_signals(pool.get_ref()).await {
        Ok(signals) => HttpResponse::Ok().json(IngestResponse {
            success: true,
            signals_created: signals.len(),
            message: format!("Successfully ingested {} signals", signals.len()),
        }),
        Err(e) => {
            tracing::error!("Signal ingestion failed: {}", e);
            HttpResponse::InternalServerError().json(IngestResponse {
                success: false,
                signals_created: 0,
                message: format!("Ingestion failed: {}", e),
            })
        }
    }
}

/// POST /api/signals/ingest/{company_id} - Trigger ingestion for a specific company
pub async fn trigger_company_ingest(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let company_id = path.into_inner();
    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let tracker = SignalTracker::new(github_token);

    // Find the company
    let company = sqlx::query_as::<_, Company>("SELECT * FROM companies WHERE id = $1")
        .bind(company_id)
        .fetch_optional(pool.get_ref())
        .await;

    match company {
        Ok(Some(company)) => {
            match tracker.ingest_company_signals(pool.get_ref(), &company).await {
                Ok(signals) => HttpResponse::Ok().json(IngestResponse {
                    success: true,
                    signals_created: signals.len(),
                    message: format!(
                        "Successfully ingested {} signals for {}",
                        signals.len(),
                        company.name
                    ),
                }),
                Err(e) => {
                    tracing::error!("Company ingestion failed: {}", e);
                    HttpResponse::InternalServerError().json(IngestResponse {
                        success: false,
                        signals_created: 0,
                        message: format!("Ingestion failed: {}", e),
                    })
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Company not found"
        })),
        Err(e) => {
            tracing::error!("Failed to find company: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error"
            }))
        }
    }
}

// ============================================================================
// Route Configuration
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/signals")
            // Public endpoints
            .route("/feed", web::get().to(get_signal_feed))
            .route("/companies", web::get().to(get_companies))
            .route("/company/{id}", web::get().to(get_company_signals))
            .route("/stats", web::get().to(get_signal_stats))
            // Admin endpoints (should add auth middleware in production)
            .route("/ingest", web::post().to(trigger_ingest))
            .route("/ingest/{id}", web::post().to(trigger_company_ingest)),
    );
}
