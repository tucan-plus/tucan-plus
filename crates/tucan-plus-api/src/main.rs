#[cfg(not(target_arch = "wasm32"))]
mod main {
    use tower::ServiceBuilder;
    use tower_http::cors::CorsLayer;
    use tucan_connector::TucanConnector;
    use tucan_plus_api::router;
    use tucan_plus_worker::MyDatabase;
    use utoipa_swagger_ui::SwaggerUi;

    #[tokio::main]
    pub async fn main() {
        env_logger::init();
        // our router
        let (router, api) = router().split_for_parts();

        let router =
            router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()));

        // run our app with hyper, listening globally on port 3000
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

        let cors = CorsLayer::very_permissive();

        let router = router.with_state(
            TucanConnector::new(MyDatabase::wait_for_worker().await)
                .await
                .unwrap(),
        );

        axum::serve(listener, router.layer(ServiceBuilder::new().layer(cors)))
            .await
            .unwrap();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use main::main;

#[cfg(target_arch = "wasm32")]
pub fn main() {}