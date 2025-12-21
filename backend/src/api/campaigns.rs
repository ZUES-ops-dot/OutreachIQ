use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use crate::models::campaign::{Campaign, CreateCampaignRequest, UpdateCampaignRequest, CampaignStatus};
use crate::middleware::auth::{extract_claims, get_workspace_id as parse_workspace_id};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/campaigns")
            .route("", web::get().to(get_campaigns))
            .route("", web::post().to(create_campaign))
            .route("/{id}", web::get().to(get_campaign_by_id))
            .route("/{id}", web::put().to(update_campaign))
            .route("/{id}", web::delete().to(delete_campaign))
            .route("/{id}/start", web::post().to(start_campaign))
            .route("/{id}/pause", web::post().to(pause_campaign))
            .route("/{id}/leads", web::get().to(get_campaign_leads))
            .route("/{id}/leads", web::post().to(add_leads_to_campaign))
    );
}

async fn get_campaigns(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    let campaigns = sqlx::query_as::<_, Campaign>(
        "SELECT * FROM campaigns WHERE workspace_id = $1 ORDER BY created_at DESC"
    )
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(campaigns))
}

async fn get_campaign_by_id(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let campaign_id = path.into_inner();

    let campaign = sqlx::query_as::<_, Campaign>(
        "SELECT * FROM campaigns WHERE id = $1 AND workspace_id = $2"
    )
    .bind(campaign_id)
    .bind(workspace_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    match campaign {
        Some(c) => Ok(HttpResponse::Ok().json(c)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Campaign not found"}))),
    }
}

async fn create_campaign(
    pool: web::Data<PgPool>,
    body: web::Json<CreateCampaignRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let campaign_id = Uuid::new_v4();
    let now = Utc::now();
    
    sqlx::query(
        r#"
        INSERT INTO campaigns (id, name, vertical, status, total_leads, sent, opened, clicked, replied, created_at, workspace_id)
        VALUES ($1, $2, $3, $4, 0, 0, 0, 0, 0, $5, $6)
        "#
    )
    .bind(campaign_id)
    .bind(&body.name)
    .bind(&body.vertical)
    .bind(CampaignStatus::Draft.as_str())
    .bind(now)
    .bind(workspace_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // If lead_ids provided, add them to campaign
    if let Some(lead_ids) = &body.lead_ids {
        for lead_id in lead_ids {
            let _ = sqlx::query(
                r#"
                INSERT INTO campaign_leads (id, campaign_id, lead_id, status)
                VALUES ($1, $2, $3, 'pending')
                ON CONFLICT (campaign_id, lead_id) DO NOTHING
                "#
            )
            .bind(Uuid::new_v4())
            .bind(campaign_id)
            .bind(lead_id)
            .execute(pool.get_ref())
            .await;
        }
        
        // Update total_leads count
        let _ = sqlx::query(
            "UPDATE campaigns SET total_leads = (SELECT COUNT(*) FROM campaign_leads WHERE campaign_id = $1) WHERE id = $1"
        )
        .bind(campaign_id)
        .execute(pool.get_ref())
        .await;
    }
    
    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": campaign_id,
        "name": body.name,
        "vertical": body.vertical,
        "status": "draft",
        "created_at": now
    })))
}

async fn update_campaign(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateCampaignRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let campaign_id = path.into_inner();
    
    let mut updates = Vec::new();
    let mut params: Vec<String> = Vec::new();
    
    if let Some(name) = &body.name {
        updates.push(format!("name = ${}", params.len() + 1));
        params.push(name.clone());
    }
    
    if let Some(status) = &body.status {
        updates.push(format!("status = ${}", params.len() + 1));
        params.push(status.clone());
    }
    
    if updates.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": "No fields to update"})));
    }
    
    let query = format!(
        "UPDATE campaigns SET {} WHERE id = ${} AND workspace_id = ${}",
        updates.join(", "),
        params.len() + 1,
        params.len() + 2
    );
    
    let mut q = sqlx::query(&query);
    for param in &params {
        q = q.bind(param);
    }
    q = q.bind(campaign_id);
    q = q.bind(workspace_id);
    
    let result = q.execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({"updated": true})))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Campaign not found"})))
    }
}

async fn delete_campaign(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let campaign_id = path.into_inner();

    let result = sqlx::query("DELETE FROM campaigns WHERE id = $1 AND workspace_id = $2")
        .bind(campaign_id)
        .bind(workspace_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Campaign not found"})))
    }
}

async fn start_campaign(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let campaign_id = path.into_inner();
    let now = Utc::now();
    
    let result = sqlx::query(
        "UPDATE campaigns SET status = 'active', started_at = $1 WHERE id = $2 AND workspace_id = $3 AND status IN ('draft', 'paused')"
    )
    .bind(now)
    .bind(campaign_id)
    .bind(workspace_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "active",
            "started_at": now
        })))
    } else {
        Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Campaign not found or cannot be started"
        })))
    }
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
        "UPDATE campaigns SET status = 'paused' WHERE id = $1 AND workspace_id = $2 AND status = 'active'"
    )
    .bind(campaign_id)
    .bind(workspace_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({"status": "paused"})))
    } else {
        Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Campaign not found or not active"
        })))
    }
}

async fn get_campaign_leads(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let campaign_id = path.into_inner();

    let leads = sqlx::query_as::<_, crate::models::lead::Lead>(
        r#"
        SELECT l.* FROM leads l
        INNER JOIN campaign_leads cl ON l.id = cl.lead_id
        INNER JOIN campaigns c ON c.id = cl.campaign_id
        WHERE cl.campaign_id = $1 AND c.workspace_id = $2
        "#
    )
    .bind(campaign_id)
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(leads))
}

#[derive(serde::Deserialize)]
pub struct AddLeadsRequest {
    pub lead_ids: Vec<Uuid>,
}

async fn add_leads_to_campaign(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<AddLeadsRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let campaign_id = path.into_inner();
    
    // Verify campaign belongs to workspace
    let campaign_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM campaigns WHERE id = $1 AND workspace_id = $2"
    )
    .bind(campaign_id)
    .bind(workspace_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    if campaign_exists == 0 {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Campaign not found"
        })));
    }
    
    let mut added = 0;
    
    // Only add leads that belong to the same workspace
    for lead_id in &body.lead_ids {
        let result = sqlx::query(
            r#"
            INSERT INTO campaign_leads (id, campaign_id, lead_id, status)
            SELECT $1, $2, l.id, 'pending'
            FROM leads l
            WHERE l.id = $3 AND l.workspace_id = $4
            ON CONFLICT (campaign_id, lead_id) DO NOTHING
            "#
        )
        .bind(Uuid::new_v4())
        .bind(campaign_id)
        .bind(lead_id)
        .bind(workspace_id)
        .execute(pool.get_ref())
        .await;
        
        if let Ok(res) = result {
            added += res.rows_affected();
        }
    }
    
    // Update total_leads count
    let _ = sqlx::query(
        "UPDATE campaigns SET total_leads = (SELECT COUNT(*) FROM campaign_leads WHERE campaign_id = $1) WHERE id = $1"
    )
    .bind(campaign_id)
    .execute(pool.get_ref())
    .await;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "added": added,
        "campaign_id": campaign_id
    })))
}
