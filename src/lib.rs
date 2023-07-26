pub mod app;
pub(crate) mod auth;
pub(crate) mod components;
#[cfg(feature = "ssr")]
pub(crate) mod database;
pub(crate) mod models;
pub(crate) mod routes;
#[cfg(feature = "ssr")]
pub mod setup;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::App;
    use leptos::*;

    tracing_wasm::set_as_global_default();
    console_error_panic_hook::set_once();

    leptos::mount_to_body(move |cx| view! { cx, <App/> });
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen(module = "/js/utils.js")]
extern "C" {
    fn decodeJWT(token: String) -> String;
    fn emailRegex(email: &str) -> bool;
}
