//! HTTP request handlers.
//!
//! This module contains all the functions that handle incoming HTTP requests.
//! Each handler is responsible for parsing the request, calling the appropriate
//! service, and formatting the response.

pub mod health;
pub mod rerank;
