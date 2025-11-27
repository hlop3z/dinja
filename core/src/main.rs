//! # Rust CMS - MDX Rendering Service
//!
//! A high-performance web service for converting MDX (Markdown with JSX) to HTML and JavaScript.
//! Extracts YAML frontmatter and transforms TSX to JavaScript using the Oxc compiler.

#[cfg(feature = "http")]
use actix_web::{web, App, HttpServer};
#[cfg(feature = "http")]
use dinja_core::handlers;
#[cfg(feature = "http")]
use dinja_core::service::{RenderService, RenderServiceConfig};

/// Entry point for the Actix Web server
#[cfg(feature = "http")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Read host and port from environment variables with defaults
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("{}:{}", host, port);

    println!("🦀 Starting Dinja MDX server on http://{}", bind_addr);

    let config = RenderServiceConfig::from_env();
    let service = RenderService::new(config)
        .expect("Failed to initialize render service with invalid configuration");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(service.clone()))
            .service(handlers::render)
    })
    .bind(&bind_addr)?
    .run()
    .await
}

#[cfg(not(feature = "http"))]
fn main() {
    eprintln!("Error: This binary requires the 'http' feature to be enabled.");
    eprintln!("Please build with: cargo build --features http");
    std::process::exit(1);
}
