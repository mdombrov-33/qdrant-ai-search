#[cfg(test)]
mod tests {
    use crate::handlers::health::health_check;
    use actix_web::{App, test, web};

    #[tokio::test]
    async fn test_health_endpoint() {
        let app =
            test::init_service(App::new().route("/health", web::get().to(health_check))).await;

        let req = test::TestRequest::get().uri("/health").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "healthy");
        assert_eq!(body["service"], "rust-accelerator");
    }
}
