use actix_web::{web, App, HttpServer, middleware as actix_middleware};
use actix_cors::Cors;
use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;

mod api;
mod models;
mod services;
mod db;
mod config;
mod middleware;
use middleware as app_middleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    println!("🚀 OutreachIQ API starting on http://0.0.0.0:8080");

    HttpServer::new(move || {
        // Secure CORS - only allow configured frontend
        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        
        let cors = Cors::default()
            .allowed_origin(&frontend_url)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::ACCEPT,
            ])
            .supports_credentials()
            .max_age(3600);
        
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .wrap(actix_middleware::Logger::default())
            .wrap(app_middleware::auth::AuthMiddleware)
            .service(
                web::scope("/api")
                    .configure(api::auth::configure)
                    .configure(api::leads::configure)
                    .configure(api::campaigns::configure)
                    .configure(api::analytics::configure)
                    .configure(api::email_accounts::configure)
                    .configure(api::compliance::configure)
                    .configure(api::billing::configure)
                    .configure(api::signals::configure)
                    .configure(api::founder_dashboard::configure)
            )
            .route("/health", web::get().to(|| async { "OK" }))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
