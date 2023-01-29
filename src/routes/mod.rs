pub mod editor;
pub mod login;
pub mod signup;
use leptos::*;

#[cfg(feature = "ssr")]
pub fn register_server_fn() {
    self::login::LoginAction::register().unwrap();
    self::login::LogoutAction::register().unwrap();
    self::signup::SignupAction::register().unwrap();
    self::editor::EditorAction::register().unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn get_username(_cx: Scope) -> Option<String> {
    let doc = document().unchecked_into::<web_sys::HtmlDocument>();
    let cookies = doc.cookie().unwrap_or_default();
    log::info!("cookies: {:?}", cookies);
    cookies
        .split("; ")
        .find(|x| x.starts_with("session"))
        .map(|x| x.split('=').last().map(|x| x.to_string()))
        .flatten()
}

#[cfg(feature = "ssr")]
pub fn get_username(cx: Scope) -> Option<String> {
    if let Some(req) = use_context::<leptos_axum::RequestParts>(cx) {
        req.headers.get("cookies").and_then(|x| {
            x.to_str()
                .unwrap()
                .split("; ")
                .find(|x| x.starts_with("session"))
                .and_then(|x| x.split('=').last().map(|x| x.to_string()))
        })
    } else {
        None
    }
}

#[cfg(feature = "ssr")]
pub async fn set_username(cx: Scope, username: String) -> bool {
    if let Some(res) = use_context::<leptos_axum::ResponseOptions>(cx) {
        res.insert_header(
            axum::http::header::SET_COOKIE,
            axum::http::HeaderValue::from_str(&format!("session={username}; path=/"))
                .expect("header value couldn't be set"),
        )
        .await;
        true
    } else {
        false
    }
}
