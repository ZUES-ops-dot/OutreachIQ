use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};

#[derive(Debug)]
pub struct AutoPauseResult {
    pub should_pause: bool,
    pub reason: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct WorkspaceThresholds {
    auto_pause_enabled: bool,
    spam_rate_threshold: f64,
    reply_drop_threshold: f64,
    bounce_rate_threshold: f64,
}

#[derive(Debug, sqlx::FromRow)]
struct CampaignMetrics {
    campaign_id: Uuid,
    campaign_name: String,
    current_spam_rate: f64,
    current_reply_rate: f64,
    current_bounce_rate: f64,
    previous_reply_rate: f64,
}

pub async fn check_and_auto_pause(pool: &PgPool, workspace_id: Uuid) -> Result<Vec<AutoPauseResult>, sqlx::Error> {
    // Get workspace settings
    let settings: Option<WorkspaceThresholds> = sqlx::query_as(
        r#"
        SELECT auto_pause_enabled, spam_rate_threshold, reply_drop_threshold, bounce_rate_threshold
        FROM workspace_settings
        WHERE workspace_id = $1
        "#
    )
    .bind(workspace_id)
    .fetch_optional(pool)
    .await?;

    let settings = settings.unwrap_or(WorkspaceThresholds {
        auto_pause_enabled: true,
        spam_rate_threshold: 0.03,
        reply_drop_threshold: 0.40,
        bounce_rate_threshold: 0.08,
    });

    if !settings.auto_pause_enabled {
        return Ok(vec![]);
    }

    let mut results = Vec::new();

    // Check each active campaign
    let campaigns: Vec<CampaignMetrics> = sqlx::query_as(
        r#"
        WITH current_metrics AS (
            SELECT 
                c.id as campaign_id,
                c.name as campaign_name,
                COALESCE(
                    (SELECT AVG(ihm.spam_rate) 
                     FROM inbox_health_metrics ihm 
                     JOIN email_accounts ea ON ihm.email_account_id = ea.id
                     WHERE ea.workspace_id = c.workspace_id 
                     AND ihm.measured_at > NOW() - INTERVAL '24 hours'), 
                    0
                ) as current_spam_rate,
                CASE WHEN c.sent > 0 THEN c.replied::FLOAT / c.sent::FLOAT ELSE 0 END as current_reply_rate,
                COALESCE(
                    (SELECT AVG(ihm.bounce_rate) 
                     FROM inbox_health_metrics ihm 
                     JOIN email_accounts ea ON ihm.email_account_id = ea.id
                     WHERE ea.workspace_id = c.workspace_id 
                     AND ihm.measured_at > NOW() - INTERVAL '24 hours'), 
                    0
                ) as current_bounce_rate
            FROM campaigns c
            WHERE c.workspace_id = $1 
            AND c.status = 'active' 
            AND COALESCE(c.auto_paused, FALSE) = FALSE
        ),
        previous_metrics AS (
            SELECT 
                c.id as campaign_id,
                CASE WHEN c.sent > 0 THEN c.replied::FLOAT / c.sent::FLOAT ELSE 0 END as previous_reply_rate
            FROM campaigns c
            WHERE c.workspace_id = $1
        )
        SELECT 
            cm.campaign_id,
            cm.campaign_name,
            cm.current_spam_rate,
            cm.current_reply_rate,
            cm.current_bounce_rate,
            COALESCE(pm.previous_reply_rate, 0) as previous_reply_rate
        FROM current_metrics cm
        LEFT JOIN previous_metrics pm ON cm.campaign_id = pm.campaign_id
        "#
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await?;

    for campaign in campaigns {
        let result = check_campaign_thresholds(&campaign, &settings);
        
        if result.should_pause {
            // Auto-pause the campaign
            let _ = pause_campaign(pool, workspace_id, campaign.campaign_id, &result).await;
            results.push(result);
        }
    }

    Ok(results)
}

fn check_campaign_thresholds(metrics: &CampaignMetrics, settings: &WorkspaceThresholds) -> AutoPauseResult {
    // Check spam rate
    if metrics.current_spam_rate > settings.spam_rate_threshold {
        return AutoPauseResult {
            should_pause: true,
            reason: Some("spam_rate".to_string()),
            detail: Some(format!(
                "Spam rate spiked to {:.1}% (threshold: {:.1}%)",
                metrics.current_spam_rate * 100.0,
                settings.spam_rate_threshold * 100.0
            )),
        };
    }

    // Check reply rate drop
    if metrics.previous_reply_rate > 0.0 {
        let reply_drop = (metrics.previous_reply_rate - metrics.current_reply_rate) / metrics.previous_reply_rate;
        if reply_drop > settings.reply_drop_threshold {
            return AutoPauseResult {
                should_pause: true,
                reason: Some("reply_drop".to_string()),
                detail: Some(format!(
                    "Reply rate dropped {:.0}% in 48 hours (from {:.1}% to {:.1}%)",
                    reply_drop * 100.0,
                    metrics.previous_reply_rate * 100.0,
                    metrics.current_reply_rate * 100.0
                )),
            };
        }
    }

    // Check bounce rate
    if metrics.current_bounce_rate > settings.bounce_rate_threshold {
        return AutoPauseResult {
            should_pause: true,
            reason: Some("bounce_rate".to_string()),
            detail: Some(format!(
                "Bounce rate reached {:.1}% (threshold: {:.1}%)",
                metrics.current_bounce_rate * 100.0,
                settings.bounce_rate_threshold * 100.0
            )),
        };
    }

    AutoPauseResult {
        should_pause: false,
        reason: None,
        detail: None,
    }
}

async fn pause_campaign(
    pool: &PgPool,
    workspace_id: Uuid,
    campaign_id: Uuid,
    result: &AutoPauseResult,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();

    // Update campaign status
    sqlx::query(
        r#"
        UPDATE campaigns 
        SET status = 'paused', 
            auto_paused = TRUE, 
            auto_pause_reason = $3,
            paused_at = $4
        WHERE id = $1 AND workspace_id = $2
        "#
    )
    .bind(campaign_id)
    .bind(workspace_id)
    .bind(&result.detail)
    .bind(now)
    .execute(pool)
    .await?;

    // Create auto-pause event
    sqlx::query(
        r#"
        INSERT INTO auto_pause_events (
            workspace_id, campaign_id, pause_reason, pause_reason_detail, created_at
        ) VALUES ($1, $2, $3, $4, $5)
        "#
    )
    .bind(workspace_id)
    .bind(campaign_id)
    .bind(&result.reason)
    .bind(&result.detail)
    .bind(now)
    .execute(pool)
    .await?;

    tracing::info!(
        "Auto-paused campaign {} for workspace {}: {:?}",
        campaign_id,
        workspace_id,
        result.detail
    );

    Ok(())
}

pub async fn update_inbox_health_metrics(pool: &PgPool, workspace_id: Uuid) -> Result<(), sqlx::Error> {
    // Calculate and store health metrics for all inboxes
    sqlx::query(
        r#"
        INSERT INTO inbox_health_metrics (
            email_account_id, workspace_id, spam_rate, reply_rate, bounce_rate,
            health_status, health_score, emails_sent, emails_delivered, emails_opened, emails_replied
        )
        SELECT 
            ea.id,
            ea.workspace_id,
            COALESCE(ea.spam_rate, 0),
            COALESCE(ea.reply_rate, 0),
            COALESCE(ea.bounce_rate, 0),
            CASE 
                WHEN ea.spam_rate > 0.03 OR ea.bounce_rate > 0.08 THEN 'danger'
                WHEN ea.spam_rate > 0.02 OR ea.bounce_rate > 0.05 THEN 'warning'
                ELSE 'healthy'
            END,
            ea.health_score,
            ea.sent_today,
            ea.sent_today,  -- Assuming all sent are delivered for now
            0,  -- Would need tracking
            0   -- Would need tracking
        FROM email_accounts ea
        WHERE ea.workspace_id = $1
        "#
    )
    .bind(workspace_id)
    .execute(pool)
    .await?;

    // Update last health check timestamp
    sqlx::query(
        "UPDATE email_accounts SET last_health_check = NOW() WHERE workspace_id = $1"
    )
    .bind(workspace_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn run_health_check_job(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Get all workspaces with active campaigns
    let workspaces: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT workspace_id 
        FROM campaigns 
        WHERE status = 'active'
        "#
    )
    .fetch_all(pool)
    .await?;

    for (workspace_id,) in workspaces {
        // Update health metrics
        if let Err(e) = update_inbox_health_metrics(pool, workspace_id).await {
            tracing::error!("Failed to update health metrics for workspace {}: {}", workspace_id, e);
        }

        // Check for auto-pause conditions
        if let Err(e) = check_and_auto_pause(pool, workspace_id).await {
            tracing::error!("Failed to check auto-pause for workspace {}: {}", workspace_id, e);
        }
    }

    Ok(())
}

pub fn detect_email_provider(email: &str) -> (&'static str, i32) {
    let domain = email.split('@').last().unwrap_or("");
    
    if domain.contains("gmail") || domain.contains("google") {
        ("google", 500)
    } else if domain.contains("outlook") || domain.contains("hotmail") || domain.contains("live") || domain.contains("microsoft") {
        ("outlook", 300)
    } else if domain.contains("zoho") {
        ("zoho", 200)
    } else if domain.contains("yahoo") {
        ("yahoo", 200)
    } else if domain.contains("icloud") || domain.contains("me.com") || domain.contains("mac.com") {
        ("apple", 200)
    } else {
        ("other", 100)
    }
}

pub async fn set_provider_limits(pool: &PgPool, email_account_id: Uuid, email: &str) -> Result<(), sqlx::Error> {
    let (provider, limit) = detect_email_provider(email);
    
    sqlx::query(
        r#"
        UPDATE email_accounts 
        SET detected_provider = $2, provider_daily_limit = $3
        WHERE id = $1
        "#
    )
    .bind(email_account_id)
    .bind(provider)
    .bind(limit)
    .execute(pool)
    .await?;

    Ok(())
}
