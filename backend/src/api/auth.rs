use actix_web::{web, HttpResponse, Responder, HttpRequest};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use std::env;

fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string())
}

const JWT_EXPIRATION_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub role: String,
    pub workspace_id: Option<String>,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/me", web::get().to(get_current_user))
            .route("/refresh", web::post().to(refresh_token))
    );
}

async fn register(
    pool: web::Data<PgPool>,
    payload: web::Json<RegisterRequest>,
) -> impl Responder {
    // Check if user already exists
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_one(pool.get_ref())
    .await;

    if let Ok(count) = existing {
        if count > 0 {
            return HttpResponse::Conflict().json(
                serde_json::json!({"error": "Email already registered"})
            );
        }
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(payload.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"error": "Failed to hash password"})
        ),
    };

    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let result = sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, name, role, created_at)
        VALUES ($1, $2, $3, $4, 'user', $5)
        "#
    )
    .bind(user_id)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.name)
    .bind(now)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            // Create a default workspace for the new user
            let workspace_id = Uuid::new_v4();
            let workspace_slug = format!("{}-workspace", payload.name.to_lowercase().replace(' ', "-"));
            
            let workspace_result = sqlx::query(
                r#"
                INSERT INTO workspaces (id, name, slug, plan_tier, created_at)
                VALUES ($1, $2, $3, 'starter', $4)
                "#
            )
            .bind(workspace_id)
            .bind(format!("{}'s Workspace", payload.name))
            .bind(&workspace_slug)
            .bind(now)
            .execute(pool.get_ref())
            .await;

            if let Err(e) = workspace_result {
                eprintln!("Failed to create workspace: {}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Failed to create workspace"})
                );
            }

            // Add user as owner of the workspace
            let member_result = sqlx::query(
                r#"
                INSERT INTO workspace_members (id, workspace_id, user_id, role, joined_at)
                VALUES ($1, $2, $3, 'owner', $4)
                "#
            )
            .bind(Uuid::new_v4())
            .bind(workspace_id)
            .bind(user_id)
            .bind(now)
            .execute(pool.get_ref())
            .await;

            if let Err(e) = member_result {
                eprintln!("Failed to add workspace member: {}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Failed to setup workspace membership"})
                );
            }

            let token = generate_token(&user_id.to_string(), &payload.email, "user", Some(&workspace_id.to_string()));
            
            HttpResponse::Created().json(AuthResponse {
                token,
                user: UserResponse {
                    id: user_id,
                    email: payload.email.clone(),
                    name: payload.name.clone(),
                    role: "user".to_string(),
                },
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(
            serde_json::json!({"error": e.to_string()})
        ),
    }
}

async fn login(
    pool: web::Data<PgPool>,
    payload: web::Json<LoginRequest>,
) -> impl Responder {
    let result = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(user)) => {
            // Verify password
            let parsed_hash = match PasswordHash::new(&user.password_hash) {
                Ok(hash) => hash,
                Err(_) => return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Invalid password hash"})
                ),
            };

            if Argon2::default().verify_password(payload.password.as_bytes(), &parsed_hash).is_err() {
                return HttpResponse::Unauthorized().json(
                    serde_json::json!({"error": "Invalid credentials"})
                );
            }

            // Update last login
            let _ = sqlx::query("UPDATE users SET last_login = NOW() WHERE id = $1")
                .bind(user.id)
                .execute(pool.get_ref())
                .await;

            // Get user's workspace (first workspace they're a member of)
            let workspace_id: Option<String> = sqlx::query_scalar(
                r#"
                SELECT w.id::text FROM workspaces w
                INNER JOIN workspace_members wm ON w.id = wm.workspace_id
                WHERE wm.user_id = $1
                ORDER BY wm.joined_at ASC
                LIMIT 1
                "#
            )
            .bind(user.id)
            .fetch_optional(pool.get_ref())
            .await
            .ok()
            .flatten();

            let token = generate_token(&user.id.to_string(), &user.email, &user.role, workspace_id.as_deref());

            HttpResponse::Ok().json(AuthResponse {
                token,
                user: UserResponse {
                    id: user.id,
                    email: user.email,
                    name: user.name,
                    role: user.role,
                },
            })
        }
        Ok(None) => HttpResponse::Unauthorized().json(
            serde_json::json!({"error": "Invalid credentials"})
        ),
        Err(e) => HttpResponse::InternalServerError().json(
            serde_json::json!({"error": e.to_string()})
        ),
    }
}

async fn get_current_user(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match extract_claims(&req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(
            serde_json::json!({"error": "Invalid or missing token"})
        ),
    };

    let user_id = match Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().json(
            serde_json::json!({"error": "Invalid user ID"})
        ),
    };

    let result = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(user)) => HttpResponse::Ok().json(UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
        }),
        Ok(None) => HttpResponse::NotFound().json(
            serde_json::json!({"error": "User not found"})
        ),
        Err(e) => HttpResponse::InternalServerError().json(
            serde_json::json!({"error": e.to_string()})
        ),
    }
}

async fn refresh_token(req: HttpRequest) -> impl Responder {
    let claims = match extract_claims(&req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().json(
            serde_json::json!({"error": "Invalid or missing token"})
        ),
    };

    let token = generate_token(&claims.user_id, &claims.sub, &claims.role, claims.workspace_id.as_deref());
    
    HttpResponse::Ok().json(serde_json::json!({
        "token": token
    }))
}

fn generate_token(user_id: &str, email: &str, role: &str, workspace_id: Option<&str>) -> String {
    let now = Utc::now();
    let exp = now + Duration::hours(JWT_EXPIRATION_HOURS);
    let secret = get_jwt_secret();

    let claims = Claims {
        sub: email.to_string(),
        user_id: user_id.to_string(),
        role: role.to_string(),
        workspace_id: workspace_id.map(|s| s.to_string()),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap_or_default()
}

pub fn extract_claims(req: &HttpRequest) -> Option<Claims> {
    let auth_header = req.headers().get("Authorization")?;
    let auth_str = auth_header.to_str().ok()?;
    
    if !auth_str.starts_with("Bearer ") {
        return None;
    }

    let token = &auth_str[7..];
    
    let secret = get_jwt_secret();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .ok()
    .map(|data| data.claims)
}

pub fn require_auth(req: &HttpRequest) -> Result<Claims, HttpResponse> {
    extract_claims(req).ok_or_else(|| {
        HttpResponse::Unauthorized().json(
            serde_json::json!({"error": "Authentication required"})
        )
    })
}

pub fn require_admin(req: &HttpRequest) -> Result<Claims, HttpResponse> {
    let claims = require_auth(req)?;
    
    if claims.role != "admin" {
        return Err(HttpResponse::Forbidden().json(
            serde_json::json!({"error": "Admin access required"})
        ));
    }
    
    Ok(claims)
}
