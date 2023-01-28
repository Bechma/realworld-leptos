use crate::app::*;
use axum::handler::Handler;
use leptos::tracing;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};

pub async fn init_app(configuration_path: Option<&str>) {
    simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");

    // Init the pool into static
    crate::database::init_db()
        .await
        .expect("problem during initialization of the database");

    // Get leptos configuration
    let conf = get_configuration(configuration_path).await.unwrap();
    let addr = conf.leptos_options.site_address;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(|cx| view! { cx, <App/> }).await;
    let leptos_options = &conf.leptos_options;
    let site_root = &leptos_options.site_root;

    // Register the server functions, this is required
    crate::routes::register_server_fn();

    let app = axum::Router::new()
        // We need to register the server function handlers
        .route(
            "/api/*fn_name",
            axum::routing::post(leptos_axum::handle_server_fns),
        )
        .leptos_routes(leptos_options.clone(), routes, |cx| view! { cx, <App/> })
        .fallback(
            // if the routes doesn't match, serve local files from site_root
            crate::files::get_static_file
                .layer(axum::Extension(std::sync::Arc::new(site_root.clone()))),
        )
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(
                    tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO),
                )
                .on_request(tower_http::trace::DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO),
                )
                .on_failure(tower_http::trace::DefaultOnFailure::new().level(tracing::Level::INFO)),
        );

    // run with hyper `axum::Server` is a re-export of `hyper::Server`
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
