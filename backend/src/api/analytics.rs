use actix_web::{web, HttpRequest, HttpResponse, Responder};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::services::deliverability::DeliverabilityService;
use crate::middleware::auth::{extract_claims, get_workspace_id};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/analytics")
            .route("/overview", web::get().to(get_overview))
            .route("/campaigns", web::get().to(get_campaign_analytics))
            .route("/leads", web::get().to(get_lead_analytics))
            .route("/deliverability", web::get().to(get_deliverability_report))
    );
}

#[derive(Debug, Serialize)]
struct OverviewStats {
    total_leads: i64,
    verified_leads: i64,
    total_campaigns: i64,
    active_campaigns: i64,
    total_sent: i64,
    total_opened: i64,
    total_replied: i64,
    open_rate: f64,
    reply_rate: f64,
}

async fn get_overview(pool: web::Data<PgPool>, req: HttpRequest) -> impl Responder {
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()})),
    };
    let workspace_id = match get_workspace_id(&claims) {
        Ok(id) => id,
        Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()})),
    };

    // Get lead stats
    let lead_stats = sqlx::query_as::<_, (i64, i64)>(
        r#"
        SELECT 
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE verification_status = 'valid') as verified
        FROM leads
        WHERE workspace_id = $1
        "#
    )
    .bind(workspace_id)
    .fetch_one(pool.get_ref())
    .await;

    // Get campaign stats
    let campaign_stats = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
        r#"
        SELECT 
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE status = 'active') as active,
            COALESCE(SUM(sent), 0) as sent,
            COALESCE(SUM(opened), 0) as opened,
            COALESCE(SUM(replied), 0) as replied
        FROM campaigns
        WHERE workspace_id = $1
        "#
    )
    .bind(workspace_id)
    .fetch_one(pool.get_ref())
    .await;

    match (lead_stats, campaign_stats) {
        (Ok((total_leads, verified_leads)), Ok((total_campaigns, active_campaigns, sent, opened, replied))) => {
            let open_rate = if sent > 0 { opened as f64 / sent as f64 } else { 0.0 };
            let reply_rate = if sent > 0 { replied as f64 / sent as f64 } else { 0.0 };
            
            HttpResponse::Ok().json(OverviewStats {
                total_leads,
                verified_leads,
                total_campaigns,
                active_campaigns,
                total_sent: sent,
                total_opened: opened,
                total_replied: replied,
                open_rate,
                reply_rate,
            })
        }
        _ => HttpResponse::Ok().json(OverviewStats {
            total_leads: 0,
            verified_leads: 0,
            total_campaigns: 0,
            active_campaigns: 0,
            total_sent: 0,
            total_opened: 0,
            total_replied: 0,
            open_rate: 0.0,
            reply_rate: 0.0,
        }),
    }
}

#[derive(Debug, Serialize)]
struct CampaignAnalytics {
    id: uuid::Uuid,
    name: String,
    status: String,
    total_leads: i32,
    sent: i32,
    opened: i32,
    clicked: i32,
    replied: i32,
    open_rate: f64,
    click_rate: f64,
    reply_rate: f64,
}

async fn get_campaign_analytics(pool: web::Data<PgPool>, req: HttpRequest) -> impl Responder {
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()})),
    };
    let workspace_id = match get_workspace_id(&claims) {
        Ok(id) => id,
        Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()})),
    };

    let result = sqlx::query_as::<_, crate::models::campaign::Campaign>(
        "SELECT * FROM campaigns WHERE workspace_id = $1 ORDER BY created_at DESC LIMIT 50"
    )
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(campaigns) => {
            let analytics: Vec<CampaignAnalytics> = campaigns.into_iter().map(|c| {
                let open_rate = if c.sent > 0 { c.opened as f64 / c.sent as f64 } else { 0.0 };
                let click_rate = if c.sent > 0 { c.clicked as f64 / c.sent as f64 } else { 0.0 };
                let reply_rate = if c.sent > 0 { c.replied as f64 / c.sent as f64 } else { 0.0 };
                
                CampaignAnalytics {
                    id: c.id,
                    name: c.name,
                    status: c.status,
                    total_leads: c.total_leads,
                    sent: c.sent,
                    opened: c.opened,
                    clicked: c.clicked,
                    replied: c.replied,
                    open_rate,
                    click_rate,
                    reply_rate,
                }
            }).collect();
            
            HttpResponse::Ok().json(analytics)
        }
        Err(e) => HttpResponse::InternalServerError().json(
            serde_json::json!({"error": e.to_string()})
        ),
    }
}

#[derive(Debug, Serialize)]
struct LeadAnalytics {
    total: i64,
    by_status: StatusBreakdown,
    by_vertical: Vec<VerticalCount>,
    avg_confidence: f64,
}

#[derive(Debug, Serialize)]
struct StatusBreakdown {
    pending: i64,
    valid: i64,
    invalid: i64,
    risky: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct VerticalCount {
    vertical: Option<String>,
    count: i64,
}

async fn get_lead_analytics(pool: web::Data<PgPool>, req: HttpRequest) -> impl Responder {
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()})),
    };
    let workspace_id = match get_workspace_id(&claims) {
        Ok(id) => id,
        Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()})),
    };

    // Get total and status breakdown
    let status_stats = sqlx::query_as::<_, (i64, i64, i64, i64, i64, f64)>(
        r#"
        SELECT 
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE verification_status = 'pending') as pending,
            COUNT(*) FILTER (WHERE verification_status = 'valid') as valid,
            COUNT(*) FILTER (WHERE verification_status = 'invalid') as invalid,
            COUNT(*) FILTER (WHERE verification_status = 'risky') as risky,
            COALESCE(AVG(confidence_score), 0) as avg_confidence
        FROM leads
        WHERE workspace_id = $1
        "#
    )
    .bind(workspace_id)
    .fetch_one(pool.get_ref())
    .await;

    match status_stats {
        Ok((total, pending, valid, invalid, risky, avg_confidence)) => {
            HttpResponse::Ok().json(LeadAnalytics {
                total,
                by_status: StatusBreakdown {
                    pending,
                    valid,
                    invalid,
                    risky,
                },
                by_vertical: vec![], // Would need to track vertical in leads table
                avg_confidence,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(
            serde_json::json!({"error": e.to_string()})
        ),
    }
}

async fn get_deliverability_report(_pool: web::Data<PgPool>, req: HttpRequest) -> impl Responder {
    // Validate auth even though we're returning mock data
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()})),
    };
    let _workspace_id = match get_workspace_id(&claims) {
        Ok(id) => id,
        Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()})),
    };

    let service = DeliverabilityService::new();
    
    // In production, these would come from actual tracking data
    let report = service.generate_report(
        1000,  // total_sent
        950,   // delivered
        30,    // bounced
        5,     // spam_complaints
    );
    
    HttpResponse::Ok().json(report)
}
