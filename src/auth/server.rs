use axum::{
    http::{header, Request, StatusCode},
    response::Response,
};

use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

static AUTH_COOKIE: &str = "token";

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String, // Optional. Subject (whom token refers to)
    pub exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
                    // aud: String,         // Optional. Audience
                    // iat: usize,          // Optional. Issued at (as UTC timestamp)
                    // iss: String,         // Optional. Issuer
                    // nbf: usize,          // Optional. Not Before (as UTC timestamp)
                    // sub: String,         // Optional. Subject (whom token refers to)
}

pub(crate) static REMOVE_COOKIE: &str = "token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT";

pub async fn auth_middleware<B>(req: Request<B>, next: axum::middleware::Next<B>) -> Response {
    match get_username_from_headers(req.headers()) {
        Some(username) => {
            let Ok(_) = crate::models::User::get(username).await else {
                tracing::info!("no user associated with this token");
                return redirect(req, next).await;
            };

            let path = req.uri().path();
            if path.starts_with("/login") || path.starts_with("/signup") {
                // If the user is authenticated, we don't want to show the login or signup pages
                return Response::builder()
                    .status(StatusCode::FOUND)
                    .header(header::LOCATION, "/")
                    .body(axum::body::boxed(axum::body::Empty::new()))
                    .unwrap();
            }
            next.run(req).await
        }
        None => redirect(req, next).await,
    }
}

async fn redirect<B>(req: Request<B>, next: axum::middleware::Next<B>) -> Response {
    let path = req.uri().path();

    if path.starts_with("/settings") || path.starts_with("/editor") {
        // authenticated routes
        Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/login")
            .header(header::SET_COOKIE, REMOVE_COOKIE)
            .body(axum::body::boxed(axum::body::Empty::new()))
            .unwrap()
    } else {
        next.run(req).await
    }
}

pub(crate) fn decode_token(
    token: &str,
) -> Result<jsonwebtoken::TokenData<TokenClaims>, jsonwebtoken::errors::Error> {
    let secret = std::env!("JWT_SECRET");
    decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
}

#[tracing::instrument]
pub(crate) fn get_username_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    headers.get(axum::http::header::COOKIE).and_then(|x| {
        x.to_str()
            .unwrap()
            .split("; ")
            .find(|&x| x.starts_with(AUTH_COOKIE))
            .and_then(|x| x.split('=').last())
            .and_then(|x| decode_token(x).map(|jwt| jwt.claims.sub).ok())
    })
}

#[tracing::instrument]
pub fn get_username() -> Option<String> {
    if let Some(req) = leptos::use_context::<leptos_axum::RequestParts>() {
        crate::auth::get_username_from_headers(&req.headers)
    } else {
        None
    }
}

#[tracing::instrument]
pub async fn set_username(username: String) -> bool {
    if let Some(res) = leptos::use_context::<leptos_axum::ResponseOptions>() {
        let claims = crate::auth::TokenClaims {
            sub: username,
            exp: (sqlx::types::chrono::Utc::now().timestamp() as usize) + 3_600_000,
        };
        let secret = std::env!("JWT_SECRET");
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();
        res.insert_header(
            axum::http::header::SET_COOKIE,
            axum::http::HeaderValue::from_str(&format!("{AUTH_COOKIE}={token}; path=/; HttpOnly"))
                .expect("header value couldn't be set"),
        );
        true
    } else {
        false
    }
}
