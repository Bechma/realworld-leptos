use axum::{
    http::{header, Request, StatusCode},
    response::Response,
};

use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

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

pub(crate) static JWT_SECRET: &[u8] = b"hello darkness my old friend";
pub static AUTH_COOKIE: &str = "token";
pub(crate) static REMOVE_COOKIE: &str = "token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT";

pub async fn auth_middleware<B>(req: Request<B>, next: axum::middleware::Next<B>) -> Response {
    match get_username_from_headers(req.headers()) {
        Some(token) => {
            let Ok(claims) = decode_token(token) else {
                tracing::info!("fail to decode cookie");
                return redirect(req, next).await;
            };

            let Ok(_) = sqlx::query!("SELECT * FROM Users WHERE username = $1", claims.claims.sub)
                .fetch_one(crate::database::get_db())
                .await else {
                    tracing::info!("no user associated with this token");
                    return redirect(req, next).await
                };

            let path = req.uri().path();
            if path.starts_with("/login") || path.starts_with("/signup") {
                // If the user is authenticated, we don't want to show the login or signup pages
                return to("/");
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
        to("/login")
    } else {
        next.run(req).await
    }
}

fn to(path: &str) -> Response {
    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, path)
        .header(header::SET_COOKIE, REMOVE_COOKIE)
        .body(axum::body::boxed(axum::body::Empty::new()))
        .unwrap()
}

pub(crate) fn decode_token(
    token: String,
) -> Result<jsonwebtoken::TokenData<TokenClaims>, jsonwebtoken::errors::Error> {
    decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
}

pub(crate) fn get_username_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    headers.get(axum::http::header::COOKIE).and_then(|x| {
        x.to_str()
            .unwrap()
            .split("; ")
            .find(|&x| x == crate::auth::AUTH_COOKIE)
            .and_then(|x| x.split('=').last().map(|x| x.to_string()))
            .and_then(|x| crate::auth::decode_token(x).map(|jwt| jwt.claims.sub).ok())
    })
}
