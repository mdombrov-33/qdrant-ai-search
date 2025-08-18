//! Timing utilities for performance measurement.

use std::time::Instant;

/// Calculates elapsed time in milliseconds from a start instant.
pub fn elapsed_ms(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}
