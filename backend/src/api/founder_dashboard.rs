use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::middleware::auth::{extract_claims, get_workspace_id as parse_workspace_id};

// ============================================================================
// DATA TYPES
// ============================================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DashboardOverview {
    pub total_campaigns: i64,
    pub active_campaigns: i64,
    pub paused_campaigns: i64,
    pub total_sent: i64,
    pub total_replies: i64,
    pub total_meetings: i64,
    pub cost_per_meeting: f64,
    pub cost_per_meeting_trend: f64,  // % change from previous period
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct InboxHealthCard {
    pub id: Uuid,
    pub email: String,
    pub provider: String,
    pub health_status: String,      // healthy, warning, danger
    pub health_score: f32,
    pub spam_rate: f32,
    pub reply_rate: f32,
    pub bounce_rate: f32,
    pub daily_limit: i32,
    pub sent_today: i32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CampaignCard {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub health_status: String,
    pub auto_paused: bool,
    pub auto_pause_reason: Option<String>,
    pub total_leads: i32,
    pub sent: i32,
    pub replied: i32,
    pub meetings_booked: i32,
    pub reply_rate: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReplyCard {
    pub id: Uuid,
    pub from_email: String,
    pub from_name: Option<String>,
    pub subject: Option<String>,
    pub body_preview: String,
    pub intent: String,
    pub intent_confidence: f32,
    pub campaign_id: Option<Uuid>,
    pub campaign_name: Option<String>,
    pub received_at: DateTime<Utc>,
    pub is_read: bool,
    pub is_actioned: bool,
}

#[derive(Debug, Serialize)]
pub struct FounderDashboardData {
    pub overview: DashboardOverview,
    pub campaigns: Vec<CampaignCard>,
    pub inboxes: Vec<InboxHealthCard>,
    pub recent_replies: Vec<ReplyCard>,
    pub unread_count: i64,
    pub action_required_count: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AutoPauseEvent {
    pub id: Uuid,
    pub campaign_id: Option<Uuid>,
    pub campaign_name: Option<String>,
    pub pause_reason: String,
    pub pause_reason_detail: Option<String>,
    pub created_at: DateTime<Utc>,
    pub is_resolved: bool,
}

#[derive(Debug, Deserialize)]
pub struct ClassifyReplyRequest {
    pub reply_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCostsRequest {
    pub campaign_id: Uuid,
    pub domain_cost: Option<f64>,
    pub inbox_cost: Option<f64>,
    pub lead_cost: Option<f64>,
    pub tool_cost: Option<f64>,
    pub other_cost: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub auto_pause_enabled: Option<bool>,
    pub spam_rate_threshold: Option<f64>,
    pub reply_drop_threshold: Option<f64>,
    pub bounce_rate_threshold: Option<f64>,
    pub notification_email: Option<String>,
    pub slack_webhook_url: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WorkspaceSettings {
    pub auto_pause_enabled: bool,
    pub spam_rate_threshold: f64,
    pub reply_drop_threshold: f64,
    pub bounce_rate_threshold: f64,
    pub google_daily_limit: i32,
    pub outlook_daily_limit: i32,
    pub zoho_daily_limit: i32,
    pub notification_email: Option<String>,
    pub slack_webhook_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CostPerMeetingStats {
    pub current_period: f64,
    pub previous_period: f64,
    pub trend_percentage: f64,
    pub total_cost: f64,
    pub total_meetings: i32,
    pub by_campaign: Vec<CampaignCostSummary>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CampaignCostSummary {
    pub campaign_id: Uuid,
    pub campaign_name: String,
    pub total_cost: f64,
    pub meetings_booked: i32,
    pub cost_per_meeting: f64,
}

// ============================================================================
// ROUTE CONFIGURATION
// ============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/founder")
            .route("/dashboard", web::get().to(get_dashboard))
            .route("/campaigns", web::get().to(get_campaigns_with_health))
            .route("/campaigns/{id}/pause", web::post().to(pause_campaign))
            .route("/campaigns/{id}/resume", web::post().to(resume_campaign))
            .route("/inboxes", web::get().to(get_inbox_health))
            .route("/inboxes/{id}/health", web::get().to(get_inbox_health_detail))
            .route("/replies", web::get().to(get_replies))
            .route("/replies/{id}/action", web::post().to(action_reply))
            .route("/replies/classify", web::post().to(classify_reply))
            .route("/auto-pause-events", web::get().to(get_auto_pause_events))
            .route("/auto-pause-events/{id}/resolve", web::post().to(resolve_pause_event))
            .route("/costs", web::get().to(get_cost_stats))
            .route("/costs", web::post().to(update_costs))
            .route("/meetings", web::get().to(get_meetings))
            .route("/meetings", web::post().to(create_meeting))
            .route("/settings", web::get().to(get_settings))
            .route("/settings", web::put().to(update_settings))
    );
}

// ============================================================================
// MAIN DASHBOARD ENDPOINT
// ============================================================================

async fn get_dashboard(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    // Get overview stats
    let overview = get_overview_stats(&pool, workspace_id).await?;
    
    // Get campaigns with health status
    let campaigns = sqlx::query_as::<_, CampaignCard>(
        r#"
        SELECT 
            c.id,
            c.name,
            c.status,
            CASE 
                WHEN c.auto_paused THEN 'danger'
                WHEN c.status = 'paused' THEN 'warning'
                ELSE 'healthy'
            END as health_status,
            COALESCE(c.auto_paused, FALSE) as auto_paused,
            c.auto_pause_reason,
            c.total_leads,
            c.sent,
            c.replied,
            COALESCE(c.meetings_booked, 0) as meetings_booked,
            CASE WHEN c.sent > 0 THEN (c.replied::FLOAT / c.sent::FLOAT) ELSE 0 END as reply_rate,
            c.created_at
        FROM campaigns c
        WHERE c.workspace_id = $1
        ORDER BY 
            CASE WHEN c.auto_paused THEN 0 ELSE 1 END,
            c.created_at DESC
        LIMIT 10
        "#
    )
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // Get inbox health
    let inboxes = sqlx::query_as::<_, InboxHealthCard>(
        r#"
        SELECT 
            ea.id,
            ea.email,
            ea.provider,
            CASE 
                WHEN ea.spam_rate > 0.03 OR ea.bounce_rate > 0.08 THEN 'danger'
                WHEN ea.spam_rate > 0.02 OR ea.bounce_rate > 0.05 OR ea.reply_rate < 0.02 THEN 'warning'
                ELSE 'healthy'
            END as health_status,
            ea.health_score,
            COALESCE(ea.spam_rate, 0) as spam_rate,
            COALESCE(ea.reply_rate, 0) as reply_rate,
            COALESCE(ea.bounce_rate, 0) as bounce_rate,
            ea.daily_limit,
            ea.sent_today
        FROM email_accounts ea
        WHERE ea.workspace_id = $1
        ORDER BY 
            CASE 
                WHEN ea.spam_rate > 0.03 OR ea.bounce_rate > 0.08 THEN 0
                WHEN ea.spam_rate > 0.02 OR ea.bounce_rate > 0.05 THEN 1
                ELSE 2
            END,
            ea.email
        "#
    )
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // Get recent replies needing action
    let recent_replies = sqlx::query_as::<_, ReplyCard>(
        r#"
        SELECT 
            er.id,
            er.from_email,
            er.from_name,
            er.subject,
            LEFT(COALESCE(er.body_text, ''), 150) as body_preview,
            er.intent,
            er.intent_confidence,
            er.campaign_id,
            c.name as campaign_name,
            er.received_at,
            er.is_read,
            er.is_actioned
        FROM email_replies er
        LEFT JOIN campaigns c ON er.campaign_id = c.id
        WHERE er.workspace_id = $1 AND er.is_actioned = FALSE
        ORDER BY 
            CASE er.intent 
                WHEN 'interested' THEN 1 
                WHEN 'objection' THEN 2
                WHEN 'maybe_later' THEN 3
                ELSE 4 
            END,
            er.received_at DESC
        LIMIT 20
        "#
    )
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // Get counts
    let unread_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM email_replies WHERE workspace_id = $1 AND is_read = FALSE"
    )
    .bind(workspace_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let action_required: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM email_replies WHERE workspace_id = $1 AND is_actioned = FALSE AND intent = 'interested'"
    )
    .bind(workspace_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let dashboard = FounderDashboardData {
        overview,
        campaigns,
        inboxes,
        recent_replies,
        unread_count: unread_count.0,
        action_required_count: action_required.0,
    };

    Ok(HttpResponse::Ok().json(dashboard))
}

async fn get_overview_stats(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<DashboardOverview, actix_web::Error> {
    // Get campaign stats
    let campaign_stats: (i64, i64, i64, i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT 
            COUNT(*),
            COUNT(*) FILTER (WHERE status = 'active'),
            COUNT(*) FILTER (WHERE auto_paused = TRUE OR status = 'paused'),
            COALESCE(SUM(sent), 0),
            COALESCE(SUM(replied), 0),
            COALESCE(SUM(meetings_booked), 0)
        FROM campaigns
        WHERE workspace_id = $1
        "#
    )
    .bind(workspace_id)
    .fetch_one(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // Get cost per meeting (last 30 days)
    let cost_stats: (Option<f64>, Option<i32>) = sqlx::query_as(
        r#"
        SELECT 
            SUM(total_cost)::FLOAT,
            SUM(meetings_booked)
        FROM campaign_costs
        WHERE workspace_id = $1 
        AND period_end >= CURRENT_DATE - INTERVAL '30 days'
        "#
    )
    .bind(workspace_id)
    .fetch_one(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let total_cost = cost_stats.0.unwrap_or(0.0);
    let meetings = cost_stats.1.unwrap_or(0).max(1) as f64;
    let cost_per_meeting = total_cost / meetings;

    // Get previous period for trend
    let prev_cost_stats: (Option<f64>, Option<i32>) = sqlx::query_as(
        r#"
        SELECT 
            SUM(total_cost)::FLOAT,
            SUM(meetings_booked)
        FROM campaign_costs
        WHERE workspace_id = $1 
        AND period_end >= CURRENT_DATE - INTERVAL '60 days'
        AND period_end < CURRENT_DATE - INTERVAL '30 days'
        "#
    )
    .bind(workspace_id)
    .fetch_one(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let prev_cost = prev_cost_stats.0.unwrap_or(0.0);
    let prev_meetings = prev_cost_stats.1.unwrap_or(0).max(1) as f64;
    let prev_cost_per_meeting = prev_cost / prev_meetings;

    let trend = if prev_cost_per_meeting > 0.0 {
        ((cost_per_meeting - prev_cost_per_meeting) / prev_cost_per_meeting) * 100.0
    } else {
        0.0
    };

    Ok(DashboardOverview {
        total_campaigns: campaign_stats.0,
        active_campaigns: campaign_stats.1,
        paused_campaigns: campaign_stats.2,
        total_sent: campaign_stats.3,
        total_replies: campaign_stats.4,
        total_meetings: campaign_stats.5,
        cost_per_meeting,
        cost_per_meeting_trend: trend,
    })
}

// ============================================================================
// CAMPAIGN ENDPOINTS
// ============================================================================

async fn get_campaigns_with_health(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    let campaigns = sqlx::query_as::<_, CampaignCard>(
        r#"
        SELECT 
            c.id,
            c.name,
            c.status,
            CASE 
                WHEN c.auto_paused THEN 'danger'
                WHEN c.status = 'paused' THEN 'warning'
                ELSE 'healthy'
            END as health_status,
            COALESCE(c.auto_paused, FALSE) as auto_paused,
            c.auto_pause_reason,
            c.total_leads,
            c.sent,
            c.replied,
            COALESCE(c.meetings_booked, 0) as meetings_booked,
            CASE WHEN c.sent > 0 THEN (c.replied::FLOAT / c.sent::FLOAT) ELSE 0 END as reply_rate,
            c.created_at
        FROM campaigns c
        WHERE c.workspace_id = $1
        ORDER BY c.created_at DESC
        "#
    )
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(campaigns))
}

async fn pause_campaign(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let campaign_id = path.into_inner();

    let result = sqlx::query(
        r#"
        UPDATE campaigns 
        SET status = 'paused', paused_at = NOW()
        WHERE id = $1 AND workspace_id = $2 AND status = 'active'
        "#
    )
    .bind(campaign_id)
    .bind(workspace_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({"status": "paused"})))
    } else {
        Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": "Campaign not found or not active"})))
    }
}

async fn resume_campaign(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let campaign_id = path.into_inner();

    // Clear auto-pause flag and resume
    let result = sqlx::query(
        r#"
        UPDATE campaigns 
        SET status = 'active', auto_paused = FALSE, auto_pause_reason = NULL, paused_at = NULL
        WHERE id = $1 AND workspace_id = $2 AND (status = 'paused' OR auto_paused = TRUE)
        "#
    )
    .bind(campaign_id)
    .bind(workspace_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // Resolve any pending auto-pause events
    let _ = sqlx::query(
        r#"
        UPDATE auto_pause_events 
        SET is_resolved = TRUE, resolved_at = NOW(), resolution_action = 'resumed'
        WHERE campaign_id = $1 AND is_resolved = FALSE
        "#
    )
    .bind(campaign_id)
    .execute(pool.get_ref())
    .await;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({"status": "active"})))
    } else {
        Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": "Campaign not found or not paused"})))
    }
}

// ============================================================================
// INBOX HEALTH ENDPOINTS
// ============================================================================

async fn get_inbox_health(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    let inboxes = sqlx::query_as::<_, InboxHealthCard>(
        r#"
        SELECT 
            ea.id,
            ea.email,
            ea.provider,
            CASE 
                WHEN ea.spam_rate > 0.03 OR ea.bounce_rate > 0.08 THEN 'danger'
                WHEN ea.spam_rate > 0.02 OR ea.bounce_rate > 0.05 OR ea.reply_rate < 0.02 THEN 'warning'
                ELSE 'healthy'
            END as health_status,
            ea.health_score,
            COALESCE(ea.spam_rate, 0) as spam_rate,
            COALESCE(ea.reply_rate, 0) as reply_rate,
            COALESCE(ea.bounce_rate, 0) as bounce_rate,
            ea.daily_limit,
            ea.sent_today
        FROM email_accounts ea
        WHERE ea.workspace_id = $1
        ORDER BY ea.email
        "#
    )
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(inboxes))
}

async fn get_inbox_health_detail(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let inbox_id = path.into_inner();

    // Get inbox details
    let inbox = sqlx::query_as::<_, InboxHealthCard>(
        r#"
        SELECT 
            ea.id,
            ea.email,
            ea.provider,
            CASE 
                WHEN ea.spam_rate > 0.03 OR ea.bounce_rate > 0.08 THEN 'danger'
                WHEN ea.spam_rate > 0.02 OR ea.bounce_rate > 0.05 OR ea.reply_rate < 0.02 THEN 'warning'
                ELSE 'healthy'
            END as health_status,
            ea.health_score,
            COALESCE(ea.spam_rate, 0) as spam_rate,
            COALESCE(ea.reply_rate, 0) as reply_rate,
            COALESCE(ea.bounce_rate, 0) as bounce_rate,
            ea.daily_limit,
            ea.sent_today
        FROM email_accounts ea
        WHERE ea.id = $1 AND ea.workspace_id = $2
        "#
    )
    .bind(inbox_id)
    .bind(workspace_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    match inbox {
        Some(i) => Ok(HttpResponse::Ok().json(i)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Inbox not found"}))),
    }
}

// ============================================================================
// REPLY ENDPOINTS
// ============================================================================

async fn get_replies(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    query: web::Query<RepliesQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    let intent_filter = query.intent.as_deref().unwrap_or("");
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let replies = if intent_filter.is_empty() {
        sqlx::query_as::<_, ReplyCard>(
            r#"
            SELECT 
                er.id,
                er.from_email,
                er.from_name,
                er.subject,
                LEFT(COALESCE(er.body_text, ''), 150) as body_preview,
                er.intent,
                er.intent_confidence,
                er.campaign_id,
                c.name as campaign_name,
                er.received_at,
                er.is_read,
                er.is_actioned
            FROM email_replies er
            LEFT JOIN campaigns c ON er.campaign_id = c.id
            WHERE er.workspace_id = $1
            ORDER BY er.received_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(workspace_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
    } else {
        sqlx::query_as::<_, ReplyCard>(
            r#"
            SELECT 
                er.id,
                er.from_email,
                er.from_name,
                er.subject,
                LEFT(COALESCE(er.body_text, ''), 150) as body_preview,
                er.intent,
                er.intent_confidence,
                er.campaign_id,
                c.name as campaign_name,
                er.received_at,
                er.is_read,
                er.is_actioned
            FROM email_replies er
            LEFT JOIN campaigns c ON er.campaign_id = c.id
            WHERE er.workspace_id = $1 AND er.intent = $4
            ORDER BY er.received_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(workspace_id)
        .bind(limit)
        .bind(offset)
        .bind(intent_filter)
        .fetch_all(pool.get_ref())
        .await
    };

    let replies = replies.map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(replies))
}

#[derive(Debug, Deserialize)]
pub struct RepliesQuery {
    pub intent: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ActionReplyRequest {
    pub action: String,  // replied, booked_meeting, snoozed, archived
}

async fn action_reply(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<ActionReplyRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let reply_id = path.into_inner();

    let result = sqlx::query(
        r#"
        UPDATE email_replies 
        SET is_actioned = TRUE, is_read = TRUE, action_taken = $3, action_at = NOW()
        WHERE id = $1 AND workspace_id = $2
        "#
    )
    .bind(reply_id)
    .bind(workspace_id)
    .bind(&body.action)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({"success": true, "action": body.action})))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Reply not found"})))
    }
}

async fn classify_reply(
    pool: web::Data<PgPool>,
    body: web::Json<ClassifyReplyRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    // Get the reply text
    let reply: Option<(String,)> = sqlx::query_as(
        "SELECT body_text FROM email_replies WHERE id = $1 AND workspace_id = $2"
    )
    .bind(body.reply_id)
    .bind(workspace_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let reply_text = match reply {
        Some((text,)) => text,
        None => return Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Reply not found"}))),
    };

    // Classify using Claude (this will be called from the service)
    let (intent, confidence) = crate::services::reply_classifier::classify_reply(&reply_text).await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // Update the reply with classification
    sqlx::query(
        r#"
        UPDATE email_replies 
        SET intent = $3, intent_confidence = $4, classified_at = NOW()
        WHERE id = $1 AND workspace_id = $2
        "#
    )
    .bind(body.reply_id)
    .bind(workspace_id)
    .bind(&intent)
    .bind(confidence)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "reply_id": body.reply_id,
        "intent": intent,
        "confidence": confidence
    })))
}

// ============================================================================
// AUTO-PAUSE EVENTS
// ============================================================================

async fn get_auto_pause_events(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    let events = sqlx::query_as::<_, AutoPauseEvent>(
        r#"
        SELECT 
            ape.id,
            ape.campaign_id,
            c.name as campaign_name,
            ape.pause_reason,
            ape.pause_reason_detail,
            ape.created_at,
            ape.is_resolved
        FROM auto_pause_events ape
        LEFT JOIN campaigns c ON ape.campaign_id = c.id
        WHERE ape.workspace_id = $1
        ORDER BY ape.is_resolved, ape.created_at DESC
        LIMIT 50
        "#
    )
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(events))
}

async fn resolve_pause_event(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let event_id = path.into_inner();

    let result = sqlx::query(
        r#"
        UPDATE auto_pause_events 
        SET is_resolved = TRUE, resolved_at = NOW()
        WHERE id = $1 AND workspace_id = $2
        "#
    )
    .bind(event_id)
    .bind(workspace_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({"resolved": true})))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Event not found"})))
    }
}

// ============================================================================
// COST TRACKING
// ============================================================================

async fn get_cost_stats(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    // Current period (last 30 days)
    let current: (Option<f64>, Option<i32>) = sqlx::query_as(
        r#"
        SELECT SUM(total_cost)::FLOAT, SUM(meetings_booked)
        FROM campaign_costs
        WHERE workspace_id = $1 AND period_end >= CURRENT_DATE - INTERVAL '30 days'
        "#
    )
    .bind(workspace_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // Previous period
    let previous: (Option<f64>, Option<i32>) = sqlx::query_as(
        r#"
        SELECT SUM(total_cost)::FLOAT, SUM(meetings_booked)
        FROM campaign_costs
        WHERE workspace_id = $1 
        AND period_end >= CURRENT_DATE - INTERVAL '60 days'
        AND period_end < CURRENT_DATE - INTERVAL '30 days'
        "#
    )
    .bind(workspace_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let current_cost = current.0.unwrap_or(0.0);
    let current_meetings = current.1.unwrap_or(0).max(1);
    let current_cpm = current_cost / current_meetings as f64;

    let prev_cost = previous.0.unwrap_or(0.0);
    let prev_meetings = previous.1.unwrap_or(0).max(1);
    let prev_cpm = prev_cost / prev_meetings as f64;

    let trend = if prev_cpm > 0.0 {
        ((current_cpm - prev_cpm) / prev_cpm) * 100.0
    } else {
        0.0
    };

    // By campaign
    let by_campaign = sqlx::query_as::<_, CampaignCostSummary>(
        r#"
        SELECT 
            cc.campaign_id,
            c.name as campaign_name,
            SUM(cc.total_cost)::FLOAT as total_cost,
            SUM(cc.meetings_booked) as meetings_booked,
            CASE 
                WHEN SUM(cc.meetings_booked) > 0 
                THEN (SUM(cc.total_cost) / SUM(cc.meetings_booked))::FLOAT
                ELSE 0 
            END as cost_per_meeting
        FROM campaign_costs cc
        JOIN campaigns c ON cc.campaign_id = c.id
        WHERE cc.workspace_id = $1
        GROUP BY cc.campaign_id, c.name
        ORDER BY cost_per_meeting DESC
        "#
    )
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let stats = CostPerMeetingStats {
        current_period: current_cpm,
        previous_period: prev_cpm,
        trend_percentage: trend,
        total_cost: current_cost,
        total_meetings: current_meetings,
        by_campaign,
    };

    Ok(HttpResponse::Ok().json(stats))
}

async fn update_costs(
    pool: web::Data<PgPool>,
    body: web::Json<UpdateCostsRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    // Upsert costs for current month
    let result = sqlx::query(
        r#"
        INSERT INTO campaign_costs (workspace_id, campaign_id, domain_cost, inbox_cost, lead_cost, tool_cost, other_cost, period_start, period_end)
        VALUES ($1, $2, $3, $4, $5, $6, $7, DATE_TRUNC('month', CURRENT_DATE), DATE_TRUNC('month', CURRENT_DATE) + INTERVAL '1 month' - INTERVAL '1 day')
        ON CONFLICT (campaign_id, period_start) 
        DO UPDATE SET 
            domain_cost = COALESCE($3, campaign_costs.domain_cost),
            inbox_cost = COALESCE($4, campaign_costs.inbox_cost),
            lead_cost = COALESCE($5, campaign_costs.lead_cost),
            tool_cost = COALESCE($6, campaign_costs.tool_cost),
            other_cost = COALESCE($7, campaign_costs.other_cost),
            updated_at = NOW()
        "#
    )
    .bind(workspace_id)
    .bind(body.campaign_id)
    .bind(body.domain_cost)
    .bind(body.inbox_cost)
    .bind(body.lead_cost)
    .bind(body.tool_cost)
    .bind(body.other_cost)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"updated": result.rows_affected() > 0})))
}

// ============================================================================
// MEETINGS
// ============================================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Meeting {
    pub id: Uuid,
    pub campaign_id: Option<Uuid>,
    pub campaign_name: Option<String>,
    pub lead_id: Option<Uuid>,
    pub lead_email: Option<String>,
    pub title: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub status: String,
    pub outcome: Option<String>,
}

async fn get_meetings(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    let meetings = sqlx::query_as::<_, Meeting>(
        r#"
        SELECT 
            m.id,
            m.campaign_id,
            c.name as campaign_name,
            m.lead_id,
            l.email as lead_email,
            m.title,
            m.scheduled_at,
            m.status,
            m.outcome
        FROM meetings m
        LEFT JOIN campaigns c ON m.campaign_id = c.id
        LEFT JOIN leads l ON m.lead_id = l.id
        WHERE m.workspace_id = $1
        ORDER BY m.scheduled_at DESC
        LIMIT 50
        "#
    )
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(meetings))
}

#[derive(Debug, Deserialize)]
pub struct CreateMeetingRequest {
    pub campaign_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub reply_id: Option<Uuid>,
    pub title: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

async fn create_meeting(
    pool: web::Data<PgPool>,
    body: web::Json<CreateMeetingRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let meeting_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO meetings (id, workspace_id, campaign_id, lead_id, reply_id, title, scheduled_at, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'scheduled')
        "#
    )
    .bind(meeting_id)
    .bind(workspace_id)
    .bind(body.campaign_id)
    .bind(body.lead_id)
    .bind(body.reply_id)
    .bind(&body.title)
    .bind(body.scheduled_at)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // Update campaign meetings count
    if let Some(campaign_id) = body.campaign_id {
        let _ = sqlx::query(
            "UPDATE campaigns SET meetings_booked = COALESCE(meetings_booked, 0) + 1 WHERE id = $1"
        )
        .bind(campaign_id)
        .execute(pool.get_ref())
        .await;

        // Update cost tracking
        let _ = sqlx::query(
            r#"
            UPDATE campaign_costs 
            SET meetings_booked = meetings_booked + 1,
                cost_per_meeting = CASE WHEN meetings_booked + 1 > 0 THEN total_cost / (meetings_booked + 1) ELSE 0 END
            WHERE campaign_id = $1 AND period_end >= CURRENT_DATE
            "#
        )
        .bind(campaign_id)
        .execute(pool.get_ref())
        .await;
    }

    Ok(HttpResponse::Created().json(serde_json::json!({"id": meeting_id})))
}

// ============================================================================
// SETTINGS
// ============================================================================

async fn get_settings(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    let settings = sqlx::query_as::<_, WorkspaceSettings>(
        r#"
        SELECT 
            auto_pause_enabled,
            spam_rate_threshold,
            reply_drop_threshold,
            bounce_rate_threshold,
            google_daily_limit,
            outlook_daily_limit,
            zoho_daily_limit,
            notification_email,
            slack_webhook_url
        FROM workspace_settings
        WHERE workspace_id = $1
        "#
    )
    .bind(workspace_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    match settings {
        Some(s) => Ok(HttpResponse::Ok().json(s)),
        None => {
            // Return defaults
            Ok(HttpResponse::Ok().json(WorkspaceSettings {
                auto_pause_enabled: true,
                spam_rate_threshold: 0.03,
                reply_drop_threshold: 0.40,
                bounce_rate_threshold: 0.08,
                google_daily_limit: 500,
                outlook_daily_limit: 300,
                zoho_daily_limit: 200,
                notification_email: None,
                slack_webhook_url: None,
            }))
        }
    }
}

async fn update_settings(
    pool: web::Data<PgPool>,
    body: web::Json<UpdateSettingsRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    sqlx::query(
        r#"
        INSERT INTO workspace_settings (workspace_id, auto_pause_enabled, spam_rate_threshold, reply_drop_threshold, bounce_rate_threshold, notification_email, slack_webhook_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (workspace_id) 
        DO UPDATE SET 
            auto_pause_enabled = COALESCE($2, workspace_settings.auto_pause_enabled),
            spam_rate_threshold = COALESCE($3, workspace_settings.spam_rate_threshold),
            reply_drop_threshold = COALESCE($4, workspace_settings.reply_drop_threshold),
            bounce_rate_threshold = COALESCE($5, workspace_settings.bounce_rate_threshold),
            notification_email = COALESCE($6, workspace_settings.notification_email),
            slack_webhook_url = COALESCE($7, workspace_settings.slack_webhook_url),
            updated_at = NOW()
        "#
    )
    .bind(workspace_id)
    .bind(body.auto_pause_enabled)
    .bind(body.spam_rate_threshold)
    .bind(body.reply_drop_threshold)
    .bind(body.bounce_rate_threshold)
    .bind(&body.notification_email)
    .bind(&body.slack_webhook_url)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"updated": true})))
}
