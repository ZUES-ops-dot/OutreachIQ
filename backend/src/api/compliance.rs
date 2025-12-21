use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use crate::middleware::auth::{extract_claims, get_workspace_id as parse_workspace_id};

#[derive(Debug, Deserialize)]
pub struct UnsubscribeRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct UnsubscribeResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct AddSuppressionRequest {
    pub email: String,
    pub reason: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/compliance")
            .route("/unsubscribe", web::get().to(handle_unsubscribe))
            .route("/unsubscribe", web::post().to(handle_unsubscribe_post))
            .route("/suppression", web::get().to(get_suppression_list))
            .route("/suppression", web::post().to(add_to_suppression))
            .route("/suppression/{email}", web::delete().to(remove_from_suppression))
    );
}

// Public endpoint - no auth required (for email unsubscribe links)
async fn handle_unsubscribe(
    pool: web::Data<PgPool>,
    query: web::Query<UnsubscribeRequest>,
) -> HttpResponse {
    process_unsubscribe(&pool, &query.token).await
}

// Public endpoint - no auth required (for one-click unsubscribe)
async fn handle_unsubscribe_post(
    pool: web::Data<PgPool>,
    query: web::Query<UnsubscribeRequest>,
) -> HttpResponse {
    process_unsubscribe(&pool, &query.token).await
}

async fn process_unsubscribe(pool: &PgPool, token: &str) -> HttpResponse {
    // Decode token: format is "lead_id:email:workspace_id"
    let decoded = match URL_SAFE_NO_PAD.decode(token) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => return HttpResponse::BadRequest().json(UnsubscribeResponse {
                success: false,
                message: "Invalid token format".to_string(),
            }),
        },
        Err(_) => return HttpResponse::BadRequest().json(UnsubscribeResponse {
            success: false,
            message: "Invalid token".to_string(),
        }),
    };

    let parts: Vec<&str> = decoded.split(':').collect();
    if parts.len() < 2 {
        return HttpResponse::BadRequest().json(UnsubscribeResponse {
            success: false,
            message: "Invalid token structure".to_string(),
        });
    }

    let email = parts[1];
    let workspace_id: Option<Uuid> = parts.get(2)
        .and_then(|s| Uuid::parse_str(s).ok());

    // Add to suppression list
    let result = sqlx::query(
        r#"
        INSERT INTO suppression_list (id, workspace_id, email, reason, source, created_at)
        VALUES ($1, $2, $3, 'unsubscribe', 'user_request', $4)
        ON CONFLICT (workspace_id, email) DO UPDATE SET
            reason = 'unsubscribe',
            source = 'user_request',
            created_at = $4
        "#
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(email)
    .bind(Utc::now())
    .execute(pool)
    .await;

    match result {
        Ok(_) => {
            // Also update any campaign_leads to mark as unsubscribed
            let _ = sqlx::query(
                r#"
                UPDATE campaign_leads cl
                SET unsubscribed_at = NOW(), status = 'unsubscribed'
                FROM leads l
                WHERE cl.lead_id = l.id AND l.email = $1
                "#
            )
            .bind(email)
            .execute(pool)
            .await;

            HttpResponse::Ok().json(UnsubscribeResponse {
                success: true,
                message: "You have been successfully unsubscribed.".to_string(),
            })
        }
        Err(e) => {
            eprintln!("Failed to process unsubscribe: {}", e);
            HttpResponse::InternalServerError().json(UnsubscribeResponse {
                success: false,
                message: "Failed to process unsubscribe request.".to_string(),
            })
        }
    }
}

// Protected endpoint - requires auth
async fn get_suppression_list(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    #[derive(Debug, sqlx::FromRow, Serialize)]
    struct SuppressionEntry {
        id: Uuid,
        email: String,
        reason: String,
        source: Option<String>,
        created_at: chrono::DateTime<Utc>,
    }

    let entries = sqlx::query_as::<_, SuppressionEntry>(
        r#"
        SELECT id, email, reason, source, created_at
        FROM suppression_list
        WHERE workspace_id = $1
        ORDER BY created_at DESC
        LIMIT 1000
        "#
    )
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(entries))
}

// Protected endpoint - requires auth
async fn add_to_suppression(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<AddSuppressionRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    let result = sqlx::query(
        r#"
        INSERT INTO suppression_list (id, workspace_id, email, reason, source, created_at)
        VALUES ($1, $2, $3, $4, 'manual', $5)
        ON CONFLICT (workspace_id, email) DO UPDATE SET
            reason = $4,
            source = 'manual',
            created_at = $5
        "#
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(&body.email)
    .bind(&body.reason)
    .bind(Utc::now())
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "email": body.email,
        "reason": body.reason
    })))
}

// Protected endpoint - requires auth
async fn remove_from_suppression(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let email = path.into_inner();

    let result = sqlx::query(
        "DELETE FROM suppression_list WHERE workspace_id = $1 AND email = $2"
    )
    .bind(workspace_id)
    .bind(&email)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true, "email": email})))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Email not found in suppression list"})))
    }
}
