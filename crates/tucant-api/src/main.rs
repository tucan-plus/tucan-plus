use tucant_api::router;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    // our router
    let (router, api) = router().split_for_parts();

    let router =
        router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
