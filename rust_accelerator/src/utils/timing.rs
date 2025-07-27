//! Timing utilities for performance measurement.
//!
//! Simple helpers for measuring how long operations take.
//! Useful for monitoring and optimization.

use std::time::Instant;

/// Calculates elapsed time in milliseconds from a start instant.
///
/// This is a simple helper to convert Rust's Duration type to
/// milliseconds as an integer, which is convenient for JSON responses.
///
/// In Python, this would be like:
/// ```python
/// import time
/// def elapsed_ms(start_time):
///     return int((time.time() - start_time) * 1000)
/// ```
pub fn elapsed_ms(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}
