use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};
use std::sync::Arc;

pub struct WarmupService {
    pool: Arc<PgPool>,
}

#[derive(Debug, sqlx::FromRow)]
struct WarmingInbox {
    id: Uuid,
    email: String,
    warmup_status: String,
    daily_limit: i32,
    sent_today: i32,
    health_score: f32,
    created_at: chrono::DateTime<Utc>,
    workspace_id: Option<Uuid>,
}

impl WarmupService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn execute_warmup_cycle(&self) -> Result<(), String> {
        // Get all warming inboxes
        let inboxes = sqlx::query_as::<_, WarmingInbox>(
            r#"
            SELECT id, email, warmup_status, daily_limit, sent_today, health_score, created_at, workspace_id
            FROM email_accounts 
            WHERE warmup_status = 'warming'
            "#
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;

        println!("Processing {} warming inboxes", inboxes.len());

        for inbox in inboxes {
            let target_volume = self.calculate_target_volume(&inbox);
            let remaining = target_volume.saturating_sub(inbox.sent_today);
            
            if remaining > 0 {
                println!("Inbox {} needs {} warmup emails today", inbox.email, remaining);
                
                // In production, you would send warmup emails to partner inboxes
                // For now, we just track the progress
                if let Err(e) = self.update_warmup_progress(&inbox).await {
                    eprintln!("Failed to update warmup progress for {}: {}", inbox.email, e);
                }
            }

            // Check if warmup is complete
            if self.is_warmup_complete(&inbox) {
                if let Err(e) = self.mark_warmup_complete(inbox.id).await {
                    eprintln!("Failed to mark warmup complete for {}: {}", inbox.email, e);
                }
            }
        }

        Ok(())
    }

    fn calculate_target_volume(&self, inbox: &WarmingInbox) -> i32 {
        // Gradual increase based on days since creation
        let days_active = (Utc::now() - inbox.created_at).num_days();
        match days_active {
            0..=2 => 5,
            3..=5 => 10,
            6..=9 => 15,
            10..=14 => 20,
            15..=21 => 30,
            22..=28 => 40,
            _ => 50,
        }
    }

    fn is_warmup_complete(&self, inbox: &WarmingInbox) -> bool {
        let days_active = (Utc::now() - inbox.created_at).num_days();
        // Warmup complete after 30 days with good health score
        days_active >= 30 && inbox.health_score >= 90.0
    }

    async fn update_warmup_progress(&self, inbox: &WarmingInbox) -> Result<(), String> {
        let target = self.calculate_target_volume(inbox);
        
        // Update daily limit to match warmup schedule
        sqlx::query(
            "UPDATE email_accounts SET daily_limit = $1 WHERE id = $2"
        )
        .bind(target)
        .bind(inbox.id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn mark_warmup_complete(&self, inbox_id: Uuid) -> Result<(), String> {
        sqlx::query(
            "UPDATE email_accounts SET warmup_status = 'active', daily_limit = 50 WHERE id = $1"
        )
        .bind(inbox_id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;

        println!("Inbox {} warmup complete, now active", inbox_id);
        Ok(())
    }

    pub async fn monitor_and_protect(&self) -> Result<(), String> {
        // Find inboxes with low health scores
        let risky_inboxes = sqlx::query_as::<_, WarmingInbox>(
            r#"
            SELECT id, email, warmup_status, daily_limit, sent_today, health_score, created_at, workspace_id
            FROM email_accounts
            WHERE health_score < 75.0 
              AND warmup_status IN ('warming', 'active')
            "#
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;

        for inbox in risky_inboxes {
            println!("⚠️ Inbox {} has low health score: {}", inbox.email, inbox.health_score);
            
            // Auto-pause if health score is critical
            if inbox.health_score < 50.0 {
                sqlx::query(
                    "UPDATE email_accounts SET warmup_status = 'paused' WHERE id = $1"
                )
                .bind(inbox.id)
                .execute(self.pool.as_ref())
                .await
                .map_err(|e| e.to_string())?;

                println!("🛑 Auto-paused inbox {} due to critical health score", inbox.email);
            }
        }

        Ok(())
    }

    pub async fn reset_daily_counters(&self) -> Result<(), String> {
        // Reset sent_today for all inboxes (run at midnight)
        sqlx::query("UPDATE email_accounts SET sent_today = 0")
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| e.to_string())?;

        println!("Reset daily send counters for all inboxes");
        Ok(())
    }

    pub async fn update_health_scores(&self) -> Result<(), String> {
        // Calculate health scores based on recent delivery metrics
        // In production, this would analyze bounce rates, spam complaints, etc.
        
        // For now, slightly decay health scores for active inboxes
        // and improve them for inboxes with good delivery
        sqlx::query(
            r#"
            UPDATE email_accounts 
            SET health_score = LEAST(100.0, health_score + 0.5)
            WHERE warmup_status IN ('warming', 'active')
              AND health_score < 100.0
            "#
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}
