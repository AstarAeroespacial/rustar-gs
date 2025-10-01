# Usage

```rust
    let addr = "localhost:9999";
    let listener = TcpListener::bind(&addr).await.unwrap();

    let router = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(root))
        .route("/jobs", post(add_job));

    tokio::spawn(async move {
        println!("Swagger UI available at http://{addr}/docs");

        axum::serve(listener, router).await.unwrap();
    });
```