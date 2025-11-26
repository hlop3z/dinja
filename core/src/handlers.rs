//! HTTP request handlers

#[cfg(feature = "http")]
use crate::models::NamedMdxBatchInput;
#[cfg(feature = "http")]
use crate::service::{RenderBatchError, RenderService};
#[cfg(feature = "http")]
use actix_web::{http::StatusCode, post, web, HttpResponse, Responder};
#[cfg(feature = "http")]
use serde_json::json;

#[cfg(feature = "http")]
#[post("/render")]
pub async fn render(
    service: web::Data<RenderService>,
    input: web::Json<NamedMdxBatchInput>,
) -> impl Responder {
    let payload = input.into_inner();
    match service.render_batch(&payload) {
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

/// Helper function to create error responses with consistent formatting
#[cfg(feature = "http")]
fn error_response(status: StatusCode, message: String) -> HttpResponse {
    HttpResponse::build(status)
        .content_type("application/json")
        .json(json!({ "error": message }))
}
