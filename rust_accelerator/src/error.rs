//! Error types and handling for the application.
//!
//! This module defines custom error types that represent different failure modes in application.

use std::fmt;

/// Custom error types for the application.
///
/// This enum represents all the different ways our application can fail.
/// Each variant contains relevant information about the specific error.

#[derive(Debug)]
pub enum AppError {
    /// Input validation failed (corresponds to HTTP 400)
    InvalidInput(String),
}

/// Implement Display trait for user-friendly error messages.
///
/// This allows us to convert errors to strings for logging and responses.
/// The Display trait is like toString() in other languages.
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
        }
    }
}

/// Implement Error trait to make this a proper Rust error type.
///
/// This trait is required for errors to work with Rust's error handling
/// ecosystem (like the `?` operator and error propagation).
impl std::error::Error for AppError {}
