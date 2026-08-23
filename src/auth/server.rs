use axum::{
    http::{Request, StatusCode, header},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

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

pub enum TokenError {
    MissingSecret(std::env::VarError),
    JsonWebToken(jsonwebtoken::errors::Error),
}

impl Debug for TokenError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, formatter)
    }
}

impl Display for TokenError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSecret(error) => write!(
                formatter,
                "JWT_SECRET is required; set it to a strong random secret used to sign authentication tokens ({error})"
            ),
            Self::JsonWebToken(error) => Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for TokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingSecret(error) => Some(error),
            Self::JsonWebToken(error) => Some(error),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for TokenError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        Self::JsonWebToken(error)
    }
}

fn jwt_secret() -> Result<String, TokenError> {
    std::env::var("JWT_SECRET").map_err(TokenError::MissingSecret)
}

pub fn validate_config() -> Result<(), TokenError> {
    jwt_secret().map(|_| ())
}

pub static REMOVE_COOKIE: &str = "token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT";

pub async fn auth_middleware(
    req: Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    match get_username_from_headers(req.headers()) {
        Some(username) => {
            let Ok(_) = crate::models::User::get(username).await else {
                tracing::info!("no user associated with this token");
                return redirect(req, next).await;
            };

            let path = req.uri().path();
            if path.starts_with("/login") || path.starts_with("/signup") {
                // If the user is authenticated, we don't want to show the login or signup pages
                return (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response();
            }
            next.run(req).await
        }
        None => redirect(req, next).await,
    }
}

async fn redirect(req: Request<axum::body::Body>, next: axum::middleware::Next) -> Response {
    let path = req.uri().path();

    if path.starts_with("/settings") || path.starts_with("/editor") {
        // authenticated routes
        (
            StatusCode::FOUND,
            [
                (header::LOCATION, "/login"),
                (header::SET_COOKIE, REMOVE_COOKIE),
            ],
        )
            .into_response()
    } else {
        next.run(req).await
    }
}

pub fn decode_token(token: &str) -> Result<jsonwebtoken::TokenData<TokenClaims>, TokenError> {
    let secret = jwt_secret()?;
    Ok(decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?)
}

pub fn encode_token(token_claims: &TokenClaims) -> Result<String, TokenError> {
    let secret = jwt_secret()?;
    Ok(jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        token_claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

#[tracing::instrument]
pub fn get_username_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    headers.get(header::COOKIE).and_then(|x| {
        x.to_str()
            .ok()?
            .split("; ")
            .find(|&x| x.starts_with(AUTH_COOKIE))
            .and_then(|x| x.split('=').next_back())
            .and_then(|x| decode_token(x).map(|jwt| jwt.claims.sub).ok())
    })
}

#[tracing::instrument]
pub fn get_username() -> Option<String> {
    leptos::prelude::use_context::<axum::http::request::Parts>()
        .and_then(|req| get_username_from_headers(&req.headers))
}

#[tracing::instrument]
pub fn set_username(username: String) -> Result<(), leptos::prelude::ServerFnError> {
    let res = leptos::prelude::use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| leptos::prelude::ServerFnError::new("response context is unavailable"))?;
    let exp = usize::try_from(sqlx::types::chrono::Utc::now().timestamp())
        .map_err(leptos::prelude::ServerFnError::new)?
        .saturating_add(3_600_000);
    let token = encode_token(&TokenClaims { sub: username, exp })?;
    let cookie =
        header::HeaderValue::from_str(&format!("{AUTH_COOKIE}={token}; path=/; HttpOnly"))?;
    res.insert_header(header::SET_COOKIE, cookie);
    Ok(())
}
