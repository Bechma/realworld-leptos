#[tracing::instrument]
pub fn get_username() -> Option<String> {
    use wasm_bindgen::JsCast;

    let doc = leptos::document().unchecked_into::<web_sys::HtmlDocument>();
    let cookies = doc.cookie().unwrap_or_default();
    cookies
        .split("; ")
        .find(|x| x.starts_with(super::AUTH_COOKIE))
        .and_then(|x| x.split('=').last().map(|x| x.to_string()))
        .map(crate::decodeJWT)
}
