use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use dotenvy::dotenv;
use std::env;
use uuid::Uuid;
use chrono::{Utc, Timelike};

// Import from main crate
use outreachiq::services::email_sender::{CampaignEmailSender, SendEmailJobPayload};
use outreachiq::services::campaign_scheduler::CampaignScheduler;
use outreachiq::services::warmup_service::WarmupService;
use outreachiq::services::auto_pause;

#[derive(Debug, sqlx::FromRow)]
struct Job {
    id: Uuid,
    workspace_id: Option<Uuid>,
    job_type: String,
    payload: serde_json::Value,
    status: String,
    retry_count: i32,
    max_retries: i32,
    created_at: chrono::DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let pool = Arc::new(pool);

    println!("🔄 OutreachIQ Worker started");
    println!("   - Processing email jobs");
    println!("   - Running campaign scheduler");
    println!("   - Managing inbox warmup");
    println!("   - Auto-pause health checks (every 6 hours)");

    let email_sender = CampaignEmailSender::new(pool.clone());
    let campaign_scheduler = CampaignScheduler::new(pool.clone());
    let warmup_service = WarmupService::new(pool.clone());

    let mut iteration = 0u64;

    loop {
        iteration += 1;

        // Process pending jobs
        match claim_pending_jobs(&pool, 10).await {
            Ok(jobs) => {
                if !jobs.is_empty() {
                    println!("[{}] Processing {} jobs", iteration, jobs.len());
                }
                
                for job in jobs {
                    let result = process_job(&job, &email_sender).await;
                    
                    match result {
                        Ok(_) => {
                            if let Err(e) = mark_completed(&pool, job.id).await {
                                eprintln!("Failed to mark job {} as completed: {}", job.id, e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Job {} failed: {}", job.id, e);
                            if let Err(mark_err) = mark_failed(&pool, job.id, &e).await {
                                eprintln!("Failed to mark job {} as failed: {}", job.id, mark_err);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error fetching jobs: {}", e);
            }
        }

        // Run campaign scheduler every 10 iterations (~50 seconds)
        if iteration % 10 == 0 {
            if let Err(e) = campaign_scheduler.process_active_campaigns().await {
                eprintln!("Campaign scheduler error: {}", e);
            }
        }

        // Run warmup cycle every 60 iterations (~5 minutes)
        if iteration % 60 == 0 {
            if let Err(e) = warmup_service.execute_warmup_cycle().await {
                eprintln!("Warmup cycle error: {}", e);
            }
            
            if let Err(e) = warmup_service.monitor_and_protect().await {
                eprintln!("Warmup monitor error: {}", e);
            }
        }

        // Run auto-pause health check every 4320 iterations (~6 hours)
        // This checks spam rates, reply drops, and bounce rates
        if iteration % 4320 == 0 {
            println!("🔍 Running auto-pause health check...");
            if let Err(e) = auto_pause::run_health_check_job(&pool).await {
                eprintln!("Auto-pause health check error: {}", e);
            }
        }

        // Reset daily counters at midnight (check every iteration)
        if should_reset_daily_counters() {
            if let Err(e) = warmup_service.reset_daily_counters().await {
                eprintln!("Failed to reset daily counters: {}", e);
            }
        }

        // Sleep before next iteration
        sleep(Duration::from_secs(5)).await;
    }
}

async fn claim_pending_jobs(pool: &sqlx::PgPool, limit: i32) -> Result<Vec<Job>, String> {
    // Atomically claim pending jobs using FOR UPDATE SKIP LOCKED
    sqlx::query_as::<_, Job>(
        r#"
        WITH claimed AS (
            SELECT id FROM jobs
            WHERE status = 'pending' 
               OR (status = 'scheduled' AND next_retry_at <= NOW())
            ORDER BY created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        )
        UPDATE jobs 
        SET status = 'processing', 
            started_at = NOW(),
            retry_count = retry_count + 1
        FROM claimed
        WHERE jobs.id = claimed.id
        RETURNING jobs.id, jobs.workspace_id, jobs.job_type, jobs.payload, 
                  jobs.status, jobs.retry_count, jobs.max_retries, jobs.created_at
        "#
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())
}

async fn process_job(job: &Job, email_sender: &CampaignEmailSender) -> Result<(), String> {
    // Parse job type (it's stored as JSON string like "\"SendEmail\"")
    let job_type = job.job_type.trim_matches('"');
    
    match job_type {
        "SendEmail" => {
            let payload: SendEmailJobPayload = serde_json::from_value(job.payload.clone())
                .map_err(|e| format!("Invalid payload: {}", e))?;
            
            email_sender.send_campaign_email(&payload).await?;
            println!("✉️  Sent email to {} for campaign {}", payload.email, payload.campaign_id);
            Ok(())
        }
        "VerifyEmail" => {
            // TODO: Implement email verification job
            println!("📧 Verify email job (not implemented)");
            Ok(())
        }
        "WarmupEmail" => {
            // TODO: Implement warmup email job
            println!("🔥 Warmup email job (not implemented)");
            Ok(())
        }
        "ProcessCampaign" => {
            // TODO: Implement campaign processing job
            println!("📊 Process campaign job (not implemented)");
            Ok(())
        }
        _ => {
            Err(format!("Unknown job type: {}", job_type))
        }
    }
}

async fn mark_completed(pool: &sqlx::PgPool, job_id: Uuid) -> Result<(), String> {
    sqlx::query(
        "UPDATE jobs SET status = 'completed', completed_at = NOW() WHERE id = $1"
    )
    .bind(job_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(())
}

async fn mark_failed(pool: &sqlx::PgPool, job_id: Uuid, error: &str) -> Result<(), String> {
    // Exponential backoff: 5min, 20min, 80min
    sqlx::query(
        r#"
        UPDATE jobs
        SET status = CASE 
                WHEN retry_count >= max_retries THEN 'failed'
                ELSE 'scheduled'
            END,
            error = $2,
            next_retry_at = CASE
                WHEN retry_count < max_retries 
                THEN NOW() + interval '5 minutes' * POWER(2, retry_count)
                ELSE NULL
            END
        WHERE id = $1
        "#
    )
    .bind(job_id)
    .bind(error)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(())
}

fn should_reset_daily_counters() -> bool {
    // Check if it's around midnight (between 00:00 and 00:05)
    let now = Utc::now();
    now.time().hour() == 0 && now.time().minute() < 5
}
