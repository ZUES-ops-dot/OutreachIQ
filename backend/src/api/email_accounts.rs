use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::middleware::auth::{extract_claims, get_workspace_id as parse_workspace_id};
use crate::services::encryption::EncryptionService;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmailAccount {
    pub id: Uuid,
    pub email: String,
    pub provider: String,
    pub smtp_host: String,
    pub smtp_port: i32,
    pub warmup_status: String,
    pub daily_limit: i32,
    pub sent_today: i32,
    pub health_score: f32,
    pub created_at: DateTime<Utc>,
    pub workspace_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmailAccountRequest {
    pub email: String,
    pub provider: String,
    pub smtp_host: String,
    pub smtp_port: i32,
    pub smtp_username: String,
    pub smtp_password: String,
}

#[derive(Debug, Serialize)]
pub struct WarmupStats {
    pub health_score: f32,
    pub daily_volume: i32,
    pub daily_limit: i32,
    pub inbox_rate: f32,
    pub spam_rate: f32,
    pub bounce_rate: f32,
    pub warmup_progress: f32,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/email-accounts")
            .route("", web::get().to(get_email_accounts))
            .route("", web::post().to(create_email_account))
            .route("/{id}", web::get().to(get_email_account))
            .route("/{id}", web::delete().to(delete_email_account))
            .route("/{id}/warmup/start", web::post().to(start_warmup))
            .route("/{id}/warmup/pause", web::post().to(pause_warmup))
            .route("/{id}/warmup/stats", web::get().to(get_warmup_stats))
    );
}

async fn get_email_accounts(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    let accounts = sqlx::query_as::<_, EmailAccount>(
        "SELECT id, email, provider, smtp_host, smtp_port, warmup_status, daily_limit, sent_today, health_score, created_at, workspace_id FROM email_accounts WHERE workspace_id = $1 ORDER BY created_at DESC"
    )
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(accounts))
}

async fn get_email_account(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let account_id = path.into_inner();

    let account = sqlx::query_as::<_, EmailAccount>(
        "SELECT id, email, provider, smtp_host, smtp_port, warmup_status, daily_limit, sent_today, health_score, created_at, workspace_id FROM email_accounts WHERE id = $1 AND workspace_id = $2"
    )
    .bind(account_id)
    .bind(workspace_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    match account {
        Some(acc) => Ok(HttpResponse::Ok().json(acc)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Email account not found"}))),
    }
}

async fn create_email_account(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateEmailAccountRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let account_id = Uuid::new_v4();
    let now = Utc::now();

    // Encrypt SMTP password before storing
    let (encrypted_password, key_id) = match EncryptionService::new() {
        Ok(enc) => match enc.encrypt(&payload.smtp_password) {
            Ok((encrypted, key_id)) => (Some(encrypted), Some(key_id)),
            Err(e) => {
                eprintln!("Warning: Failed to encrypt password: {}. Storing without encryption.", e);
                (None, None)
            }
        },
        Err(e) => {
            eprintln!("Warning: Encryption service unavailable: {}. Storing without encryption.", e);
            (None, None)
        }
    };

    let result = sqlx::query_as::<_, EmailAccount>(
        r#"
        INSERT INTO email_accounts 
        (id, email, provider, smtp_host, smtp_port, smtp_username, smtp_password, smtp_password_encrypted, encryption_key_id, warmup_status, daily_limit, sent_today, health_score, created_at, workspace_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending', 10, 0, 100.0, $10, $11)
        RETURNING id, email, provider, smtp_host, smtp_port, warmup_status, daily_limit, sent_today, health_score, created_at, workspace_id
        "#
    )
    .bind(account_id)
    .bind(&payload.email)
    .bind(&payload.provider)
    .bind(&payload.smtp_host)
    .bind(payload.smtp_port)
    .bind(&payload.smtp_username)
    .bind(if encrypted_password.is_some() { None::<&str> } else { Some(payload.smtp_password.as_str()) }) // Only store plaintext if encryption failed
    .bind(&encrypted_password)
    .bind(&key_id)
    .bind(now)
    .bind(workspace_id)
    .fetch_one(pool.get_ref())
    .await;

    let account = result
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Created().json(account))
}

async fn delete_email_account(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let account_id = path.into_inner();

    let result = sqlx::query("DELETE FROM email_accounts WHERE id = $1 AND workspace_id = $2")
        .bind(account_id)
        .bind(workspace_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Email account not found"})))
    }
}

async fn start_warmup(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let account_id = path.into_inner();

    let account = sqlx::query_as::<_, EmailAccount>(
        r#"
        UPDATE email_accounts 
        SET warmup_status = 'warming'
        WHERE id = $1 AND workspace_id = $2 AND warmup_status IN ('pending', 'paused')
        RETURNING id, email, provider, smtp_host, smtp_port, warmup_status, daily_limit, sent_today, health_score, created_at, workspace_id
        "#
    )
    .bind(account_id)
    .bind(workspace_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    match account {
        Some(acc) => Ok(HttpResponse::Ok().json(acc)),
        None => Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": "Account not found or cannot start warmup"}))),
    }
}

async fn pause_warmup(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let account_id = path.into_inner();

    let account = sqlx::query_as::<_, EmailAccount>(
        r#"
        UPDATE email_accounts 
        SET warmup_status = 'paused'
        WHERE id = $1 AND workspace_id = $2 AND warmup_status IN ('warming', 'active')
        RETURNING id, email, provider, smtp_host, smtp_port, warmup_status, daily_limit, sent_today, health_score, created_at, workspace_id
        "#
    )
    .bind(account_id)
    .bind(workspace_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    match account {
        Some(acc) => Ok(HttpResponse::Ok().json(acc)),
        None => Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": "Account not found or cannot pause warmup"}))),
    }
}

async fn get_warmup_stats(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let account_id = path.into_inner();

    let account = sqlx::query_as::<_, EmailAccount>(
        "SELECT id, email, provider, smtp_host, smtp_port, warmup_status, daily_limit, sent_today, health_score, created_at, workspace_id FROM email_accounts WHERE id = $1 AND workspace_id = $2"
    )
    .bind(account_id)
    .bind(workspace_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    match account {
        Some(acc) => {
            let target_volume = 50;
            let warmup_progress = (acc.daily_limit as f32 / target_volume as f32) * 100.0;
            
            let stats = WarmupStats {
                health_score: acc.health_score,
                daily_volume: acc.sent_today,
                daily_limit: acc.daily_limit,
                inbox_rate: 98.0,
                spam_rate: 0.5,
                bounce_rate: 1.5,
                warmup_progress: warmup_progress.min(100.0),
            };
            
            Ok(HttpResponse::Ok().json(stats))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Email account not found"}))),
    }
}
