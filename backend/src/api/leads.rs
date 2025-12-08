use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::lead::{Lead, LeadSearchQuery};
use crate::services::lead_generator::LeadGenerator;
use crate::services::email_verifier::EmailVerifier;
use crate::middleware::auth::{extract_claims, get_workspace_id as parse_workspace_id};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/leads")
            .route("", web::get().to(get_leads))
            .route("/{id}", web::get().to(get_lead_by_id))
            .route("/search", web::post().to(search_leads))
            .route("/verify", web::post().to(verify_leads))
            .route("/signals/{domain}", web::get().to(get_signals))
            .route("/{id}", web::delete().to(delete_lead))
    );
}

async fn get_leads(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    let leads = sqlx::query_as::<_, Lead>(
        "SELECT * FROM leads WHERE workspace_id = $1 ORDER BY created_at DESC LIMIT 100"
    )
    .bind(workspace_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(leads))
}

async fn get_lead_by_id(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let lead_id = path.into_inner();

    let lead = sqlx::query_as::<_, Lead>(
        "SELECT * FROM leads WHERE id = $1 AND workspace_id = $2"
    )
    .bind(lead_id)
    .bind(workspace_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    match lead {
        Some(lead) => Ok(HttpResponse::Ok().json(lead)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Lead not found"}))),
    }
}

async fn search_leads(
    query: web::Json<LeadSearchQuery>,
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;

    // Check usage limits
    let usage_count: (i64,) = sqlx::query_as(
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

    let workspace_limit: Option<(i32,)> = sqlx::query_as(
        "SELECT monthly_lead_limit FROM workspaces WHERE id = $1"
    )
    .bind(workspace_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if let Some((limit,)) = workspace_limit {
        if usage_count.0 >= limit as i64 {
            return Ok(HttpResponse::PaymentRequired().json(
                serde_json::json!({"error": "Monthly lead limit exceeded", "limit": limit, "used": usage_count.0})
            ));
        }
    }

    let generator = LeadGenerator::new();
    
    let mut leads = generator.generate_leads(
        &query.vertical,
        query.role.as_deref(),
        query.limit.unwrap_or(50) as usize,
    )
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // Verify emails
    if let Ok(verifier) = EmailVerifier::new().await {
        for lead in &mut leads {
            let (status, confidence) = verifier.verify_email(&lead.email).await;
            lead.verification_status = status;
            lead.confidence_score = confidence;
        }
    }

    // Store in database with workspace_id
    for lead in &leads {
        let _ = sqlx::query(
            r#"
            INSERT INTO leads (id, workspace_id, email, first_name, last_name, company, title, 
                              linkedin_url, verification_status, confidence_score, signals, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (workspace_id, email) DO UPDATE SET
                verification_status = EXCLUDED.verification_status,
                confidence_score = EXCLUDED.confidence_score,
                signals = EXCLUDED.signals
            "#
        )
        .bind(&lead.id)
        .bind(workspace_id)
        .bind(&lead.email)
        .bind(&lead.first_name)
        .bind(&lead.last_name)
        .bind(&lead.company)
        .bind(&lead.title)
        .bind(&lead.linkedin_url)
        .bind(lead.verification_status.as_str())
        .bind(lead.confidence_score)
        .bind(&lead.signals)
        .bind(lead.created_at)
        .execute(pool.get_ref())
        .await;
    }

    // Track usage
    let _ = sqlx::query(
        r#"
        INSERT INTO usage_metrics (id, workspace_id, metric_type, count, period_start, period_end)
        VALUES (gen_random_uuid(), $1, 'leads_generated', $2, date_trunc('month', NOW())::date, (date_trunc('month', NOW()) + interval '1 month')::date)
        ON CONFLICT (workspace_id, metric_type, period_start) 
        DO UPDATE SET count = usage_metrics.count + $2
        "#
    )
    .bind(workspace_id)
    .bind(leads.len() as i32)
    .execute(pool.get_ref())
    .await;

    Ok(HttpResponse::Ok().json(leads))
}

async fn verify_leads(
    emails: web::Json<Vec<String>>,
    _pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let _claims = extract_claims(&req)?;

    let verifier = EmailVerifier::new()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let mut results = Vec::new();

    for email in emails.iter() {
        let (status, confidence) = verifier.verify_email(email).await;
        results.push(serde_json::json!({
            "email": email,
            "status": status.as_str(),
            "confidence": confidence
        }));
    }

    Ok(HttpResponse::Ok().json(results))
}

async fn get_signals(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let _claims = extract_claims(&req)?;

    let domain = path.into_inner();
    
    // Find company by domain and get its signals
    let company = crate::models::company::Company::find_by_domain(pool.get_ref(), &domain)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    match company {
        Some(company) => {
            let signals = crate::models::signal::Signal::find_by_company(pool.get_ref(), company.id)
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
            Ok(HttpResponse::Ok().json(signals))
        }
        None => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Company not found",
                "domain": domain
            })))
        }
    }
}

async fn delete_lead(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = extract_claims(&req)?;
    let workspace_id = parse_workspace_id(&claims)?;
    let lead_id = path.into_inner();

    let result = sqlx::query("DELETE FROM leads WHERE id = $1 AND workspace_id = $2")
        .bind(lead_id)
        .bind(workspace_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Lead not found"})))
    }
}
