use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path();
        
        // Skip auth for public routes
        if path.starts_with("/api/auth/") 
            || path.starts_with("/api/compliance/unsubscribe")
            || path == "/api/billing/webhook"
            || path == "/api/billing/pricing"
            || path.starts_with("/api/signals/feed")
            || path.starts_with("/api/signals/companies")
            || path.starts_with("/api/signals/company/")
            || path.starts_with("/api/signals/stats")
            || path == "/health" 
            || path == "/" 
        {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        // Extract JWT from Authorization header
        let auth_header = req.headers().get("Authorization");
        
        let token = match auth_header {
            Some(header) => {
                let header_str = header.to_str().unwrap_or("");
                if header_str.starts_with("Bearer ") {
                    header_str[7..].to_string()
                } else {
                    return Box::pin(async {
                        Err(actix_web::error::ErrorUnauthorized("Invalid authorization header format"))
                    });
                }
            }
            None => {
                return Box::pin(async {
                    Err(actix_web::error::ErrorUnauthorized("Missing authorization header"))
                });
            }
        };

        // Decode and validate JWT
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string());
        
        let validation = Validation::default();
        let token_data = match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        ) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("JWT decode error: {:?}", e);
                return Box::pin(async {
                    Err(actix_web::error::ErrorUnauthorized("Invalid or expired token"))
                });
            }
        };

        // Add claims to request extensions for downstream handlers
        req.extensions_mut().insert(token_data.claims);

        let fut = self.service.call(req);
        Box::pin(async move { fut.await })
    }
}

// Helper to extract claims from ServiceRequest (for middleware use)
pub fn get_claims(req: &ServiceRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

// Helper function for route handlers to extract claims from HttpRequest
pub fn extract_claims(req: &actix_web::HttpRequest) -> Result<Claims, actix_web::Error> {
    req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))
}

// Role-based authorization helpers
pub fn require_role(claims: &Claims, required_roles: &[&str]) -> Result<(), actix_web::Error> {
    if required_roles.contains(&claims.role.as_str()) {
        Ok(())
    } else {
        Err(actix_web::error::ErrorForbidden("Insufficient permissions"))
    }
}

// Check if user can write (owner, admin, member)
pub fn require_write_access(claims: &Claims) -> Result<(), actix_web::Error> {
    require_role(claims, &["owner", "admin", "member"])
}

// Check if user can admin (owner, admin)
pub fn require_admin_access(claims: &Claims) -> Result<(), actix_web::Error> {
    require_role(claims, &["owner", "admin"])
}

// Check if user is owner
pub fn require_owner_access(claims: &Claims) -> Result<(), actix_web::Error> {
    require_role(claims, &["owner"])
}

// Parse workspace_id from claims
pub fn get_workspace_id(claims: &Claims) -> Result<uuid::Uuid, actix_web::Error> {
    let workspace_id = claims.workspace_id.as_ref()
        .ok_or_else(|| actix_web::error::ErrorBadRequest("No workspace ID in token"))?;
    uuid::Uuid::parse_str(workspace_id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid workspace ID in token"))
}

// Parse user_id from claims
pub fn get_user_id(claims: &Claims) -> Result<uuid::Uuid, actix_web::Error> {
    uuid::Uuid::parse_str(&claims.user_id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user ID in token"))
}
