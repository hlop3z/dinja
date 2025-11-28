//! # Dinja - MDX Rendering Service
//!
//! A high-performance web service for converting MDX (Markdown with JSX) to HTML and JavaScript.
//! Extracts YAML frontmatter and transforms TSX to JavaScript using the Oxc compiler.

#[cfg(feature = "http")]
use actix_web::{web, App, HttpServer};
#[cfg(feature = "http")]
use clap::Parser;
#[cfg(feature = "http")]
use dinja_core::handlers;
#[cfg(feature = "http")]
use dinja_core::models::ResourceLimits;
#[cfg(feature = "http")]
use dinja_core::service::{RenderService, RenderServiceConfig};
#[cfg(feature = "http")]
use std::path::PathBuf;

/// Dinja MDX Rendering Server
#[cfg(feature = "http")]
#[derive(Parser, Debug)]
#[command(name = "dinja")]
#[command(author, version, about = "High-performance MDX rendering service", long_about = None)]
struct Cli {
    /// Host address to bind to
    #[arg(short = 'H', long, default_value = "0.0.0.0", env = "HOST")]
    host: String,

    /// Port to listen on
    #[arg(short, long, default_value = "8080", env = "PORT")]
    port: u16,

    /// Path to configuration file (TOML)
    #[arg(short, long, env = "DINJA_CONFIG")]
    config: Option<PathBuf>,

    /// Directory containing static files (engine.min.js)
    #[arg(short, long, default_value = "static", env = "RUST_CMS_STATIC_DIR")]
    static_dir: PathBuf,

    /// Maximum cached renderers per thread
    #[arg(long, default_value = "4", env = "DINJA_MAX_CACHED_RENDERERS")]
    max_cached_renderers: usize,

    /// Maximum files per batch request
    #[arg(long, default_value = "1000", env = "DINJA_MAX_BATCH_SIZE")]
    max_batch_size: usize,

    /// Maximum MDX content size in bytes (default: 10MB)
    #[arg(long, default_value = "10485760", env = "DINJA_MAX_MDX_SIZE")]
    max_mdx_size: usize,

    /// Maximum component code size in bytes (default: 1MB)
    #[arg(long, default_value = "1048576", env = "DINJA_MAX_COMPONENT_SIZE")]
    max_component_size: usize,
}

#[cfg(feature = "http")]
impl Cli {
    fn into_config(self) -> RenderServiceConfig {
        RenderServiceConfig {
            static_dir: self.static_dir,
            max_cached_renderers: self.max_cached_renderers,
            resource_limits: ResourceLimits {
                max_batch_size: self.max_batch_size,
                max_mdx_content_size: self.max_mdx_size,
                max_component_code_size: self.max_component_size,
            },
        }
    }
}

/// Entry point for the Actix Web server
#[cfg(feature = "http")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let bind_addr = format!("{}:{}", cli.host, cli.port);

    println!("🦀 Dinja MDX Server");
    println!("   Listening on http://{}", bind_addr);
    println!("   Static dir:  {}", cli.static_dir.display());
    println!("   Max renderers: {}", cli.max_cached_renderers);
    println!("   Max batch: {}", cli.max_batch_size);

    let config = if let Some(ref config_path) = cli.config {
        println!("   Config file: {}", config_path.display());
        RenderServiceConfig::from_file_and_env(config_path).unwrap_or_else(|e| {
            eprintln!("⚠️  Failed to load config file: {}", e);
            eprintln!("   Falling back to CLI arguments");
            cli.into_config()
        })
    } else {
        cli.into_config()
    };

    let service = match RenderService::new(config) {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("❌ Failed to initialize render service: {}", err);
            eprintln!("   Check your configuration and resource limits.");
            std::process::exit(1);
        }
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(service.clone()))
            .service(handlers::health)
            .service(handlers::render)
            .service(handlers::render_html)
            .service(handlers::render_javascript)
            .service(handlers::render_schema)
            .service(handlers::render_json)
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
