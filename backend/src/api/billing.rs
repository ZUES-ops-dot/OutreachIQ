use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::middleware::auth::{extract_claims, get_workspace_id as parse_workspace_id};

#[derive(Debug, Clone, Serialize)]
pub struct PricingTier {
    pub id: String,
    pub name: String,
    pub price_monthly: i32,
    pub price_yearly: i32,
    pub leads_per_month: i32,
    pub inboxes: i32,
    pub emails_per_month: i32,
    pub features: Vec<String>,
}

pub fn get_pricing_tiers() -> Vec<PricingTier> {
    vec![
        PricingTier {
            id: "starter".to_string(),
            name: "Starter".to_string(),
            price_monthly: 9700,
            price_yearly: 97000,
            leads_per_month: 1000,
            inboxes: 1,
            emails_per_month: 500,
            features: vec!["Basic warmup".to_string(), "Email support".to_string(), "Lead verification".to_string()],
        },
        PricingTier {
            id: "professional".to_string(),
            name: "Professional".to_string(),
            price_monthly: 29700,
            price_yearly: 297000,
            leads_per_month: 10000,
            inboxes: 5,
            emails_per_month: 5000,
            features: vec!["Advanced warmup".to_string(), "Domain health monitoring".to_string(), "Priority support".to_string(), "A/B testing".to_string(), "Analytics dashboard".to_string()],
        },
        PricingTier {
            id: "business".to_string(),
            name: "Business".to_string(),
            price_monthly: 99700,
            price_yearly: 997000,
            leads_per_month: 50000,
            inboxes: 20,
            emails_per_month: 25000,
            features: vec!["Dedicated deliverability manager".to_string(), "Custom integrations".to_string(), "Slack support".to_string(), "95%+ inbox guarantee".to_string(), "ROI reporting".to_string(), "White-glove onboarding".to_string()],
        },
    ]
}

#[derive(Debug, Deserialize)]
pub struct CreateCheckoutRequest {
    pub tier_id: String,
    pub billing_cycle: String,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub checkout_url: String,
    pub session_id: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/billing")
            .route("/pricing", web::get().to(get_pricing))
            .route("/checkout", web::post().to(create_checkout))
            .route("/portal", web::post().to(create_portal_session))
            .route("/subscription", web::get().to(get_subscription))
            .route("/webhook", web::post().to(handle_webhook))
            .route("/usage", web::get().to(get_usage))
    );
}

async fn get_pricing() -> HttpResponse {
    HttpResponse::Ok().json(get_pricing_tiers())
}

async fn create_checkout(
    _pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<CreateCheckoutRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let _claims = extract_claims(&req)?;
    let _workspace_id = parse_workspace_id(&_claims)?;

    let tiers = get_pricing_tiers();
    let _tier = tiers.iter()
        .find(|t| t.id == body.tier_id)
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid tier"))?;

    // TODO: Implement Stripe checkout when STRIPE_SECRET_KEY is configured
    // For now, return a proper error status so frontend handles it correctly
    Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
        "error": "Stripe not configured. Set STRIPE_SECRET_KEY environment variable.",
        "tier_id": body.tier_id,
        "billing_cycle": body.billing_cycle
    })))
}

async fn create_portal_session(
    _pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let _claims = extract_claims(&req)?;
    let _workspace_id = parse_workspace_id(&_claims)?;

    // TODO: Implement Stripe portal when configured
    Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
        "error": "Stripe not configured. Set STRIPE_SECRET_KEY environment variable."
    })))
}

async fn get_subscription(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    #[derive(Debug, sqlx::FromRow, Serialize)]
    struct WorkspaceSubscription {
        plan_tier: String,
        monthly_lead_limit: i32,
        monthly_email_limit: i32,
        stripe_subscription_id: Option<String>,
    }

    let subscription = sqlx::query_as::<_, WorkspaceSubscription>(
        r#"
        SELECT plan_tier, monthly_lead_limit, monthly_email_limit, stripe_subscription_id
        FROM workspaces WHERE id = $1
        "#
    )
    .bind(workspace_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    match subscription {
        Some(sub) => Ok(HttpResponse::Ok().json(sub)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Workspace not found"}))),
    }
}

async fn get_usage(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    #[derive(Debug, Serialize)]
    struct UsageSummary {
        leads_used: i64,
        leads_limit: i32,
        emails_sent: i64,
        emails_limit: i32,
        period_start: String,
        period_end: String,
    }

    let leads_used: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM leads 
        WHERE workspace_id = $1 
        AND created_at >= date_trunc('month', NOW())
        "#
    )
    .bind(workspace_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let emails_sent: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM campaign_leads cl
        JOIN campaigns c ON cl.campaign_id = c.id
        WHERE c.workspace_id = $1 
        AND cl.sent_at >= date_trunc('month', NOW())
        "#
    )
    .bind(workspace_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let limits: Option<(i32, i32)> = sqlx::query_as(
        "SELECT monthly_lead_limit, monthly_email_limit FROM workspaces WHERE id = $1"
    )
    .bind(workspace_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let (leads_limit, emails_limit) = limits.unwrap_or((1000, 500));

    let now = Utc::now();
    let period_start = now.format("%Y-%m-01").to_string();
    let period_end = (now + chrono::Duration::days(30)).format("%Y-%m-%d").to_string();

    Ok(HttpResponse::Ok().json(UsageSummary {
        leads_used: leads_used.0,
        leads_limit,
        emails_sent: emails_sent.0,
        emails_limit,
        period_start,
        period_end,
    }))
}

async fn handle_webhook(
    _pool: web::Data<PgPool>,
    _req: HttpRequest,
    _body: web::Bytes,
) -> HttpResponse {
    // TODO: Implement Stripe webhook handling when configured
    HttpResponse::Ok().json(serde_json::json!({"received": true}))
}
