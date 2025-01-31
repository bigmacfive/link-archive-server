use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFoundError(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),

    #[error("Internal server error")]
    InternalServerError,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::AuthError(msg) => HttpResponse::Unauthorized().json(json!({ "error": msg })),
            AppError::ValidationError(msg) => HttpResponse::BadRequest().json(json!({ "error": msg })),
            AppError::NotFoundError(msg) => HttpResponse::NotFound().json(json!({ "error": msg })),
            AppError::DatabaseError(e) => {
                log::error!("Database error: {}", e);
                HttpResponse::InternalServerError().json(json!({ "error": "Internal server error" }))
            },
            AppError::ExternalServiceError(msg) => HttpResponse::ServiceUnavailable().json(json!({ "error": msg })),
            AppError::InternalServerError => {
                HttpResponse::InternalServerError().json(json!({ "error": "Internal server error" }))
            }
        }
    }
}