// Import needed types:
// - HttpResponse is the object returned to the client
// - Result is a type alias for actix's response with error handling
use actix_web::{HttpResponse, Result};

// Import the macro `json!` from serde_json
// - Lets us build JSON objects easily like in JavaScript: json!({ "foo": "bar" })
use serde_json::json;

/// The actual handler function for the `/health` route
/// - It's asynchronous (because Actix is async-first)
/// - Returns a Result-wrapped HttpResponse
pub async fn health_check() -> Result<HttpResponse> {
    // Create a JSON object using the `json!` macro
    // This will be serialized to a response body
    let response = json!({
        "status": "healthy",                // Status message
        "service": "rust-accelerator",      // Name of the service
        "message": "Service is running properly" // Optional info
    });

    // Return an HTTP 200 OK response with a JSON body
    // The .json(response) method tells Actix to serialize the object
    // Ok(...) wraps the response in a Result — required by Actix
    Ok(HttpResponse::Ok().json(response))
}
