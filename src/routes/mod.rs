pub mod editor;
pub mod login;
pub mod signup;
use leptos::*;

#[cfg(not(feature = "ssr"))]
#[tracing::instrument]
pub fn get_username(_cx: Scope) -> Option<String> {
    use wasm_bindgen::JsCast;

    let doc = document().unchecked_into::<web_sys::HtmlDocument>();
    let cookies = doc.cookie().unwrap_or_default();
    cookies
        .split("; ")
        .find(|x| x.starts_with("token"))
        .and_then(|x| x.split('=').last().map(|x| x.to_string()))
        .map(crate::decodeJWT)
}

#[cfg(feature = "ssr")]
#[tracing::instrument]
pub fn get_username(cx: Scope) -> Option<String> {
    if let Some(req) = use_context::<leptos_axum::RequestParts>(cx) {
        crate::auth::get_username_from_headers(&req.headers)
    } else {
        None
    }
}

#[cfg(feature = "ssr")]
#[tracing::instrument]
pub async fn set_username(cx: Scope, username: String) -> bool {
    if let Some(res) = use_context::<leptos_axum::ResponseOptions>(cx) {
        let claims = crate::auth::TokenClaims {
            sub: username,
            exp: (sqlx::types::chrono::Utc::now().timestamp() as usize) + 3_600_000,
        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(crate::auth::JWT_SECRET),
        )
        .unwrap();
        res.insert_header(
            axum::http::header::SET_COOKIE,
            axum::http::HeaderValue::from_str(&format!(
                "{}={token}; path=/",
                crate::auth::AUTH_COOKIE
            ))
            .expect("header value couldn't be set"),
        );
        true
    } else {
        false
    }
}
