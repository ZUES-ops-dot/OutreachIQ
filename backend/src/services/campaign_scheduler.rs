use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;

pub struct CampaignScheduler {
    pool: Arc<PgPool>,
}

#[derive(Debug, sqlx::FromRow)]
struct PendingLead {
    id: Uuid,
    lead_id: Uuid,
    campaign_id: Uuid,
    email: String,
}

#[derive(Debug, sqlx::FromRow)]
struct AvailableInbox {
    id: Uuid,
    email: String,
    daily_limit: i32,
    sent_today: i32,
    health_score: f32,
}

impl CampaignScheduler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn schedule_campaign_sends(&self, campaign_id: Uuid) -> Result<i32, String> {
        // Get workspace_id for the campaign
        let workspace_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT workspace_id FROM campaigns WHERE id = $1"
        )
        .bind(campaign_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;

        let workspace_id = workspace_id.ok_or("Campaign not found")?;

        // Get campaign leads that need sending (excluding suppressed emails)
        let leads = sqlx::query_as::<_, PendingLead>(
            r#"
            SELECT cl.id, cl.lead_id, cl.campaign_id, l.email
            FROM campaign_leads cl
            JOIN leads l ON cl.lead_id = l.id
            WHERE cl.campaign_id = $1 
              AND cl.status = 'pending'
              AND l.email NOT IN (
                  SELECT email FROM suppression_list 
                  WHERE workspace_id = $2
              )
            ORDER BY cl.created_at ASC
            LIMIT 100
            "#
        )
        .bind(campaign_id)
        .bind(workspace_id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;

        if leads.is_empty() {
            return Ok(0);
        }

        // Get available inboxes with capacity
        let inboxes = self.get_available_inboxes(workspace_id).await?;

        if inboxes.is_empty() {
            return Err("No available inboxes with sending capacity".to_string());
        }

        let mut scheduled = 0;

        // Distribute leads across inboxes respecting daily limits
        for (idx, lead) in leads.iter().enumerate() {
            let inbox = &inboxes[idx % inboxes.len()];
            
            // Check if inbox still has capacity
            let remaining_capacity = inbox.daily_limit - inbox.sent_today;
            if remaining_capacity <= 0 {
                continue;
            }

            // Create send job
            let job_id = Uuid::new_v4();
            let payload = serde_json::json!({
                "campaign_lead_id": lead.id,
                "campaign_id": campaign_id,
                "lead_id": lead.lead_id,
                "inbox_id": inbox.id,
                "email": lead.email
            });

            let result = sqlx::query(
                r#"
                INSERT INTO jobs (id, workspace_id, job_type, payload, status, created_at, retry_count, max_retries)
                VALUES ($1, $2, '"SendEmail"', $3, 'pending', $4, 0, 3)
                "#
            )
            .bind(job_id)
            .bind(workspace_id)
            .bind(&payload)
            .bind(Utc::now())
            .execute(self.pool.as_ref())
            .await;

            if result.is_ok() {
                // Mark lead as scheduled
                let _ = sqlx::query(
                    "UPDATE campaign_leads SET status = 'scheduled' WHERE id = $1"
                )
                .bind(lead.id)
                .execute(self.pool.as_ref())
                .await;

                scheduled += 1;
            }
        }

        Ok(scheduled)
    }

    async fn get_available_inboxes(&self, workspace_id: Uuid) -> Result<Vec<AvailableInbox>, String> {
        sqlx::query_as::<_, AvailableInbox>(
            r#"
            SELECT id, email, daily_limit, sent_today, health_score 
            FROM email_accounts
            WHERE workspace_id = $1
              AND warmup_status IN ('active', 'warming')
              AND sent_today < daily_limit
              AND health_score >= 50.0
            ORDER BY health_score DESC, sent_today ASC
            "#
        )
        .bind(workspace_id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn process_active_campaigns(&self) -> Result<(), String> {
        // Get all active campaigns
        let campaign_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM campaigns WHERE status = 'active'"
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;

        for campaign_id in campaign_ids {
            match self.schedule_campaign_sends(campaign_id).await {
                Ok(count) => {
                    if count > 0 {
                        println!("Scheduled {} emails for campaign {}", count, campaign_id);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to schedule campaign {}: {}", campaign_id, e);
                }
            }
        }

        Ok(())
    }
}
