pub mod app;
#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod database;
pub mod routes;
#[cfg(feature = "ssr")]
pub mod setup;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    use leptos::*;

    tracing_wasm::set_as_global_default();
    console_error_panic_hook::set_once();

    leptos::mount_to_body(move |cx| view! { cx, <App/> });
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen(module = "/js/jwt.js")]
extern "C" {
    fn decodeJWT(token: String) -> String;
}
