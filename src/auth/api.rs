use leptos::*;

pub type LogoutSignal = Action<LogoutAction, Result<(), ServerFnError>>;
pub type UsernameSignal = RwSignal<Option<String>>;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum SignupResponse {
    ValidationError(String),
    CreateUserError(String),
    Success,
}

#[tracing::instrument]
pub fn validate_signup(
    username: String,
    email: String,
    password: String,
) -> Result<crate::models::User, String> {
    crate::models::User::default()
        .set_username(username)?
        .set_password(password)?
        .set_email(email)
}

#[tracing::instrument]
#[server(SignupAction, "/api")]
pub async fn signup_action(
    username: String,
    email: String,
    password: String,
) -> Result<SignupResponse, ServerFnError> {
    match validate_signup(username.clone(), email, password) {
        Ok(user) => match user.insert().await {
            Ok(_) => {
                crate::auth::set_username(username).await;
                Ok(SignupResponse::Success)
            }
            Err(x) => {
                let x = x.to_string();
                Ok(if x.contains("users_email_key") {
                    SignupResponse::CreateUserError("Duplicated email".to_string())
                } else if x.contains("users_pkey") {
                    SignupResponse::CreateUserError("Duplicated user".to_string())
                } else {
                    tracing::error!("error from DB: {}", x);
                    SignupResponse::CreateUserError(
                        "There is an unknown problem, try again later".to_string(),
                    )
                })
            }
        },
        Err(x) => Ok(SignupResponse::ValidationError(x)),
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub enum LoginMessages {
    Successful,
    Unsuccessful,
}

#[server(LoginAction, "/api")]
#[tracing::instrument]
pub async fn login_action(
    username: String,
    password: String,
) -> Result<LoginMessages, ServerFnError> {
    if sqlx::query_scalar!(
        "SELECT username FROM Users where username=$1 and password=crypt($2, password)",
        username,
        password,
    )
    .fetch_one(crate::database::get_db())
    .await
    .unwrap_or_default()
        == username
    {
        crate::auth::set_username(username).await;
        // leptos_axum::redirect(cx, "/"); // TODO remove when it doesn't provoke a full app reload
        Ok(LoginMessages::Successful)
    } else {
        let response_options = use_context::<leptos_axum::ResponseOptions>().unwrap();
        response_options.set_status(axum::http::StatusCode::FORBIDDEN);
        Ok(LoginMessages::Unsuccessful)
    }
}

#[server(LogoutAction, "/api")]
#[tracing::instrument]
pub async fn logout_action() -> Result<(), ServerFnError> {
    let response_options = use_context::<leptos_axum::ResponseOptions>().unwrap();
    response_options.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(crate::auth::REMOVE_COOKIE)
            .expect("header value couldn't be set"),
    );
    Ok(())
}
