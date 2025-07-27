use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    InvalidInput(String),
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            AppError::InternalError(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

// Convert AppError into proper HTTP responses for Actix
impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::InvalidInput(msg) => HttpResponse::BadRequest().json(json!({
                "error": "InvalidInput",
                "message": msg,
            })),
            AppError::InternalError(msg) => HttpResponse::InternalServerError().json(json!({
                "error": "InternalError",
                "message": msg,
            })),
        }
    }
}
