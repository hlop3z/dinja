//! HTTP request handlers

#[cfg(feature = "http")]
use crate::models::{NamedMdxBatchInput, OutputFormat, RenderInput};
#[cfg(feature = "http")]
use crate::service::{RenderBatchError, RenderService};
#[cfg(feature = "http")]
use actix_web::{get, http::StatusCode, post, web, HttpResponse, Responder};
#[cfg(feature = "http")]
use serde_json::json;

/// Render MDX to HTML
/// POST /render/html
#[cfg(feature = "http")]
#[post("/render/html")]
pub async fn render_html(
    service: web::Data<RenderService>,
    input: web::Json<RenderInput>,
) -> impl Responder {
    render_with_format(service, input.into_inner(), OutputFormat::Html)
}

/// Render MDX to JavaScript
/// POST /render/javascript
#[cfg(feature = "http")]
#[post("/render/javascript")]
pub async fn render_javascript(
    service: web::Data<RenderService>,
    input: web::Json<RenderInput>,
) -> impl Responder {
    render_with_format(service, input.into_inner(), OutputFormat::Javascript)
}

/// Extract schema from MDX (component names)
/// POST /render/schema
#[cfg(feature = "http")]
#[post("/render/schema")]
pub async fn render_schema(
    service: web::Data<RenderService>,
    input: web::Json<RenderInput>,
) -> impl Responder {
    render_with_format(service, input.into_inner(), OutputFormat::Schema)
}

/// Render MDX to JSON tree
/// POST /render/json
#[cfg(feature = "http")]
#[post("/render/json")]
pub async fn render_json(
    service: web::Data<RenderService>,
    input: web::Json<RenderInput>,
) -> impl Responder {
    render_with_format(service, input.into_inner(), OutputFormat::Json)
}

/// Legacy endpoint - render with settings in body
/// POST /render
#[cfg(feature = "http")]
#[post("/render")]
pub async fn render(
    service: web::Data<RenderService>,
    input: web::Json<NamedMdxBatchInput>,
) -> impl Responder {
    let payload = input.into_inner();
    handle_render_result(service.render_batch(&payload))
}

/// Internal helper for format-specific rendering
#[cfg(feature = "http")]
fn render_with_format(
    service: web::Data<RenderService>,
    input: RenderInput,
    format: OutputFormat,
) -> HttpResponse {
    let batch_input = input.into_batch_input(format);
    handle_render_result(service.render_batch(&batch_input))
}

/// Handle render result and convert to HTTP response
#[cfg(feature = "http")]
fn handle_render_result(
    result: Result<crate::service::BatchRenderOutcome, RenderBatchError>,
) -> HttpResponse {
    match result {
        Ok(outcome) => {
            let status = if outcome.is_all_success() {
                StatusCode::OK
            } else if outcome.is_complete_failure() {
                StatusCode::INTERNAL_SERVER_ERROR
            } else {
                StatusCode::MULTI_STATUS
            };
            HttpResponse::build(status)
                .content_type("application/json")
                .json(outcome)
        }
        Err(RenderBatchError::Forbidden(message)) => error_response(StatusCode::FORBIDDEN, message),
        Err(RenderBatchError::InvalidRequest(message)) => {
            error_response(StatusCode::BAD_REQUEST, message)
        }
        Err(RenderBatchError::Internal(err)) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
        }
    }
}

/// Health check endpoint
#[cfg(feature = "http")]
#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({ "status": "ok" }))
}

/// Helper function to create error responses with consistent formatting
#[cfg(feature = "http")]
fn error_response(status: StatusCode, message: String) -> HttpResponse {
    HttpResponse::build(status)
        .content_type("application/json")
        .json(json!({ "error": message }))
}
