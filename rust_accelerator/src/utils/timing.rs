use std::time::Instant;
pub fn elapsed_ms(start: Instant) -> u64 {
    let duration = start.elapsed(); // Duration since start
    duration.as_millis() as u64 // Convert to milliseconds as u64
}
