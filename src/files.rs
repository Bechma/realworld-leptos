/// Example code gotten from https://github.com/tokio-rs/axum/discussions/446#discussioncomment-1577374
/// to serve the files from a dir
#[cfg(feature = "ssr")]
pub async fn get_static_file(
    uri: axum::http::Uri,
    root: axum::Extension<std::sync::Arc<String>>,
) -> Result<axum::http::Response<axum::body::BoxBody>, (axum::http::StatusCode, String)> {
    use tower::ServiceExt;

    let req = axum::http::Request::builder()
        .uri(uri.clone())
        .body(axum::body::Body::empty())
        .unwrap();
    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    // This path is relative to the cargo root
    tower_http::services::ServeDir::new(&**root)
        .oneshot(req)
        .await
        .map(|res| res.map(axum::body::boxed))
        .map_err(|err| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {err}"),
            )
        })
}
