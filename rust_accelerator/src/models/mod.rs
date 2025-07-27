//! Data models for the re-ranking service.
//!
//! This module organizes all our data structures. In Rust, it's common
//! to separate public API models from internal processing models.

pub mod internal;
pub mod request; // Models for incoming requests
pub mod response; // Models for outgoing responses // Internal models used during processing
