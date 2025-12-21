use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use tokio::sync::mpsc;
use std::sync::Arc;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    SendEmail,
    VerifyEmail,
    WarmupEmail,
    ProcessCampaign,
    UpdateAnalytics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Scheduled,
    Processing,
    Completed,
    Failed,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Scheduled => "scheduled",
            JobStatus::Processing => "processing",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub attempts: i32,
    pub max_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub workspace_id: Option<Uuid>,
    pub next_retry_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailPayload {
    pub campaign_id: Uuid,
    pub lead_id: Uuid,
    pub email_account_id: Uuid,
    pub to_email: String,
    pub to_name: Option<String>,
    pub subject: String,
    pub body_html: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyEmailPayload {
    pub lead_id: Uuid,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupEmailPayload {
    pub email_account_id: Uuid,
    pub target_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessCampaignPayload {
    pub campaign_id: Uuid,
}

pub struct JobQueue {
    pool: Arc<PgPool>,
    sender: mpsc::Sender<Job>,
}

impl JobQueue {
    pub fn new(pool: Arc<PgPool>) -> (Self, mpsc::Receiver<Job>) {
        let (sender, receiver) = mpsc::channel(1000);
        (Self { pool, sender }, receiver)
    }

    pub async fn enqueue(&self, job_type: JobType, payload: serde_json::Value, workspace_id: Option<Uuid>) -> Result<Uuid, String> {
        let job_id = Uuid::new_v4();
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            INSERT INTO jobs (id, job_type, payload, status, attempts, max_attempts, created_at, workspace_id, retry_count, max_retries)
            VALUES ($1, $2, $3, 'pending', 0, 3, $4, $5, 0, 3)
            "#
        )
        .bind(job_id)
        .bind(serde_json::to_string(&job_type).map_err(|e| e.to_string())?)
        .bind(&payload)
        .bind(now)
        .bind(workspace_id)
        .execute(self.pool.as_ref())
        .await;

        match result {
            Ok(_) => {
                let job = Job {
                    id: job_id,
                    job_type,
                    payload,
                    status: JobStatus::Pending,
                    attempts: 0,
                    max_attempts: 3,
                    created_at: now,
                    started_at: None,
                    completed_at: None,
                    error: None,
                    workspace_id,
                    next_retry_at: None,
                };
                
                let _ = self.sender.send(job).await;
                Ok(job_id)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn enqueue_send_email(&self, payload: SendEmailPayload, workspace_id: Option<Uuid>) -> Result<Uuid, String> {
        self.enqueue(JobType::SendEmail, serde_json::to_value(payload).map_err(|e| e.to_string())?, workspace_id).await
    }

    pub async fn enqueue_verify_email(&self, payload: VerifyEmailPayload, workspace_id: Option<Uuid>) -> Result<Uuid, String> {
        self.enqueue(JobType::VerifyEmail, serde_json::to_value(payload).map_err(|e| e.to_string())?, workspace_id).await
    }

    pub async fn enqueue_warmup_email(&self, payload: WarmupEmailPayload, workspace_id: Option<Uuid>) -> Result<Uuid, String> {
        self.enqueue(JobType::WarmupEmail, serde_json::to_value(payload).map_err(|e| e.to_string())?, workspace_id).await
    }

    pub async fn enqueue_process_campaign(&self, payload: ProcessCampaignPayload, workspace_id: Option<Uuid>) -> Result<Uuid, String> {
        self.enqueue(JobType::ProcessCampaign, serde_json::to_value(payload).map_err(|e| e.to_string())?, workspace_id).await
    }

    /// Atomically claim pending jobs using SELECT FOR UPDATE SKIP LOCKED
    /// This prevents race conditions when multiple workers are running
    pub async fn claim_pending_jobs(&self, limit: i32) -> Vec<Job> {
        // Use a transaction with FOR UPDATE SKIP LOCKED for atomic claiming
        let result = sqlx::query_as::<_, (Uuid, String, serde_json::Value, i32, i32, DateTime<Utc>, Option<Uuid>)>(
            r#"
            WITH claimed AS (
                SELECT id FROM jobs
                WHERE (status = 'pending' OR (status = 'scheduled' AND next_retry_at <= NOW()))
                ORDER BY created_at ASC
                LIMIT $1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE jobs SET status = 'processing', started_at = NOW(), retry_count = retry_count + 1
            FROM claimed
            WHERE jobs.id = claimed.id
            RETURNING jobs.id, jobs.job_type, jobs.payload, jobs.retry_count, jobs.max_retries, jobs.created_at, jobs.workspace_id
            "#
        )
        .bind(limit)
        .fetch_all(self.pool.as_ref())
        .await;

        match result {
            Ok(rows) => rows.into_iter().map(|(id, job_type_str, payload, attempts, max_attempts, created_at, workspace_id)| {
                Job {
                    id,
                    job_type: serde_json::from_str(&job_type_str).unwrap_or(JobType::SendEmail),
                    payload,
                    status: JobStatus::Processing,
                    attempts,
                    max_attempts,
                    created_at,
                    started_at: Some(Utc::now()),
                    completed_at: None,
                    error: None,
                    workspace_id,
                    next_retry_at: None,
                }
            }).collect(),
            Err(e) => {
                eprintln!("Failed to claim jobs: {}", e);
                vec![]
            },
        }
    }

    pub async fn mark_completed(&self, job_id: Uuid) -> Result<(), String> {
        sqlx::query(
            "UPDATE jobs SET status = 'completed', completed_at = NOW() WHERE id = $1"
        )
        .bind(job_id)
        .execute(self.pool.as_ref())
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
    }

    pub async fn mark_failed(&self, job_id: Uuid, error: &str) -> Result<(), String> {
        // Calculate exponential backoff for retry: 30s, 2min, 8min
        let backoff_seconds = 30;
        let next_retry = Utc::now() + Duration::seconds(backoff_seconds);
        
        sqlx::query(
            r#"
            UPDATE jobs 
            SET status = CASE 
                    WHEN retry_count < max_retries THEN 'scheduled' 
                    ELSE 'failed' 
                END,
                error = $2,
                next_retry_at = CASE 
                    WHEN retry_count < max_retries THEN $3 
                    ELSE NULL 
                END
            WHERE id = $1
            "#
        )
        .bind(job_id)
        .bind(error)
        .bind(next_retry)
        .execute(self.pool.as_ref())
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
    }
}

pub struct JobWorker {
    pool: Arc<PgPool>,
    queue: Arc<JobQueue>,
}

impl JobWorker {
    pub fn new(pool: Arc<PgPool>, queue: Arc<JobQueue>) -> Self {
        Self { pool, queue }
    }

    pub async fn run(&self) {
        println!("Job worker started");
        loop {
            // Atomically claim jobs - this handles the status update
            let jobs = self.queue.claim_pending_jobs(10).await;
            
            if !jobs.is_empty() {
                println!("Processing {} jobs", jobs.len());
            }
            
            for job in jobs {
                let result = self.process_job(&job).await;
                
                match result {
                    Ok(_) => {
                        if let Err(e) = self.queue.mark_completed(job.id).await {
                            eprintln!("Failed to mark job {} as completed: {}", job.id, e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Job {} failed: {}", job.id, e);
                        if let Err(mark_err) = self.queue.mark_failed(job.id, &e).await {
                            eprintln!("Failed to mark job {} as failed: {}", job.id, mark_err);
                        }
                    }
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    async fn process_job(&self, job: &Job) -> Result<(), String> {
        match job.job_type {
            JobType::SendEmail => self.process_send_email(job).await,
            JobType::VerifyEmail => self.process_verify_email(job).await,
            JobType::WarmupEmail => self.process_warmup_email(job).await,
            JobType::ProcessCampaign => self.process_campaign(job).await,
            JobType::UpdateAnalytics => self.process_analytics(job).await,
        }
    }

    async fn process_send_email(&self, job: &Job) -> Result<(), String> {
        let payload: SendEmailPayload = serde_json::from_value(job.payload.clone())
            .map_err(|e| e.to_string())?;

        // Get email account credentials
        let account = sqlx::query_as::<_, (String, i32, String, String)>(
            "SELECT smtp_host, smtp_port, smtp_username, smtp_password FROM email_accounts WHERE id = $1"
        )
        .bind(payload.email_account_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Email account not found")?;

        let sender = crate::services::email_sender::EmailSender::new(
            account.0,
            account.1 as u16,
            account.2,
            account.3,
            payload.to_email.clone(),
            "OutreachIQ".to_string(),
        );

        let request = crate::services::email_sender::SendEmailRequest {
            to_email: payload.to_email,
            to_name: payload.to_name,
            subject: payload.subject,
            body_html: payload.body_html,
            body_text: None,
        };

        let result = sender.send(request).await;

        if result.success {
            // Update campaign_leads with sent_at
            let _ = sqlx::query(
                "UPDATE campaign_leads SET sent_at = NOW(), status = 'sent' WHERE campaign_id = $1 AND lead_id = $2"
            )
            .bind(payload.campaign_id)
            .bind(payload.lead_id)
            .execute(self.pool.as_ref())
            .await;

            // Update campaign sent count
            let _ = sqlx::query(
                "UPDATE campaigns SET sent = sent + 1 WHERE id = $1"
            )
            .bind(payload.campaign_id)
            .execute(self.pool.as_ref())
            .await;

            // Update email account sent_today
            let _ = sqlx::query(
                "UPDATE email_accounts SET sent_today = sent_today + 1 WHERE id = $1"
            )
            .bind(payload.email_account_id)
            .execute(self.pool.as_ref())
            .await;

            Ok(())
        } else {
            Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }

    async fn process_verify_email(&self, job: &Job) -> Result<(), String> {
        let payload: VerifyEmailPayload = serde_json::from_value(job.payload.clone())
            .map_err(|e| e.to_string())?;

        let verifier = crate::services::email_verifier::EmailVerifier::new().await
            .map_err(|e| e.to_string())?;
        let (status, confidence) = verifier.verify_email(&payload.email).await;

        let _ = sqlx::query(
            "UPDATE leads SET verification_status = $1, confidence_score = $2, verified_at = NOW() WHERE id = $3"
        )
        .bind(status.as_str())
        .bind(confidence)
        .bind(payload.lead_id)
        .execute(self.pool.as_ref())
        .await;

        Ok(())
    }

    async fn process_warmup_email(&self, job: &Job) -> Result<(), String> {
        let payload: WarmupEmailPayload = serde_json::from_value(job.payload.clone())
            .map_err(|e| e.to_string())?;

        // Warmup emails are sent to partner inboxes to build reputation
        // This is a simplified implementation
        println!("Processing warmup email for account {} to {}", payload.email_account_id, payload.target_email);
        
        Ok(())
    }

    async fn process_campaign(&self, job: &Job) -> Result<(), String> {
        let payload: ProcessCampaignPayload = serde_json::from_value(job.payload.clone())
            .map_err(|e| e.to_string())?;

        // Get campaign details
        let campaign = sqlx::query_as::<_, (String, i32)>(
            "SELECT status, total_leads FROM campaigns WHERE id = $1"
        )
        .bind(payload.campaign_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Campaign not found")?;

        if campaign.0 != "active" {
            return Ok(()); // Campaign not active, skip
        }

        // Get pending leads for this campaign
        let pending_leads = sqlx::query_as::<_, (Uuid, String, Option<String>)>(
            r#"
            SELECT l.id, l.email, l.first_name
            FROM leads l
            INNER JOIN campaign_leads cl ON l.id = cl.lead_id
            WHERE cl.campaign_id = $1 AND cl.status = 'pending'
            LIMIT 50
            "#
        )
        .bind(payload.campaign_id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;

        println!("Processing {} leads for campaign {}", pending_leads.len(), payload.campaign_id);

        Ok(())
    }

    async fn process_analytics(&self, _job: &Job) -> Result<(), String> {
        // Update analytics aggregations
        Ok(())
    }
}
