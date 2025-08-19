mod api;
mod attestation;
mod github;
mod utils;
mod verification;

use actix_files::Files;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
use std::sync::Arc;
use tracing::{info, warn};

use crate::api::handlers;
use crate::utils::storage::ProofStorage;

pub type AppState = web::Data<Arc<AppData>>;

pub struct AppData {
    pub proof_storage: ProofStorage,
    pub github_client: github::GitHubClient,
    pub maa_client: attestation::MAAClient,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting GitHub Activity Verifier");

    // Check MAA configuration
    let maa_endpoint = match std::env::var("MAA_ENDPOINT") {
        Ok(endpoint) if !endpoint.is_empty() => {
            info!("MAA Endpoint configured: {}", endpoint);
            endpoint
        }
        _ => {
            warn!("MAA_ENDPOINT not configured - MAA functionality disabled");
            String::new()
        }
    };

    // Initialize application state
    let app_data = Arc::new(AppData {
        proof_storage: ProofStorage::new(),
        github_client: github::GitHubClient::new(),
        maa_client: attestation::MAAClient::new(maa_endpoint),
    });

    let port = std::env::var("PORT").unwrap_or_else(|_| "9000".to_string());
    let bind_address = format!("0.0.0.0:{}", port);

    info!("Binding to: {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .wrap(Logger::default())
            .service(web::scope("/api").route("/verify", web::post().to(handlers::verify)))
            .route("/proof/{proof_hash}", web::get().to(handlers::get_proof))
            .service(Files::new("/static", "./static").index_file("index.html"))
            .route("/", web::get().to(serve_index))
            .default_service(web::route().to(not_found))
    })
    .bind(&bind_address)?
    .run()
    .await
}

async fn serve_index() -> Result<HttpResponse> {
    let index_content = std::fs::read_to_string("./static/index.html").unwrap_or_else(|_| {
        r#"<!DOCTYPE html>
<html>
<head><title>GitHub Activity Verifier</title></head>
<body>
    <h1>GitHub Activity Verifier</h1>
    <p>Static files not found. Please make sure static/index.html exists.</p>
</body>
</html>"#
            .to_string()
    });

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(index_content))
}

async fn not_found() -> Result<HttpResponse> {
    Ok(HttpResponse::NotFound().json(serde_json::json!({
        "error": "Not found",
        "error_code": "NOT_FOUND"
    })))
}
