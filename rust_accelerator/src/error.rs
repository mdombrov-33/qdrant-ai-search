// Error types and handling for the application.

use std::fmt;

/// Custom error types for the application.

#[derive(Debug)]
pub enum AppError {
    /// Input validation failed (corresponds to HTTP 400)
    InvalidInput(String),
}

/// Implement Display trait for user-friendly error messages.
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
        }
    }
}

/// Implement Error trait for AppError.
impl std::error::Error for AppError {}
