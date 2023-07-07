use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[cfg(feature = "ssr")]
static EMAIL_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum SignupResponse {
    ValidationError(String),
    CreateUserError(String),
    Success(String),
}

#[cfg(feature = "ssr")]
#[tracing::instrument]
fn validate_form(username: String, email: String, password: String) -> Result<(), String> {
    if username.len() < 4 {
        return Err("Username is too short, at least 4".into());
    }

    if !EMAIL_REGEX
        .get_or_init(|| regex::Regex::new(r"^[\w\-\.]+@([\w-]+\.)+\w{2,4}$").unwrap())
        .is_match(email.as_str())
    {
        return Err("You need to provide an email address".into());
    }

    if password.len() < 4 {
        return Err("You need to provide a stronger password".into());
    }

    Ok(())
}

#[tracing::instrument]
#[server(SignupAction, "/api")]
pub async fn signup_action(
    cx: Scope,
    username: String,
    email: String,
    password: String,
) -> Result<SignupResponse, ServerFnError> {
    if let Err(x) = validate_form(username.clone(), email.clone(), password.clone()) {
        return Ok(SignupResponse::ValidationError(x));
    }
    let mut conn = crate::database::get_db().acquire().await.unwrap();
    match sqlx::query!(
        "INSERT INTO Users(username, email, password) VALUES ($1, $2, crypt($3, gen_salt('bf')))",
        username.clone(),
        email,
        password,
    )
    .execute(&mut conn)
    .await
    {
        Ok(_) => {
            super::set_username(cx, username.clone()).await;
            Ok(SignupResponse::Success(username))
        }
        Err(x) => {
            let x = x.to_string();
            Ok(if x.contains("users_email_key") {
                SignupResponse::CreateUserError("Duplicated email".to_string())
            } else if x.contains("users_pkey") {
                SignupResponse::CreateUserError("Duplicated user".to_string())
            } else {
                SignupResponse::CreateUserError(
                    "There is an unknown problem, try again later".to_string(),
                )
            })
        }
    }
}

#[tracing::instrument]
#[component]
pub fn Signup(cx: Scope) -> impl IntoView {
    let (error, set_error) = create_signal(cx, view! {cx, <ul></ul>});

    let signup_server_action = create_server_action::<SignupAction>(cx);
    let result_of_call = signup_server_action.value();

    let username = use_context::<crate::app::AuthState>(cx).unwrap();

    create_effect(cx, move |_| {
        if let Some(msg) = result_of_call.get() {
            match msg {
                Ok(SignupResponse::ValidationError(x)) => set_error.set(view! {cx,
                    <ul class="error-messages">
                        <li>"Problem while validating: "{x}</li>
                    </ul>
                }),
                Ok(SignupResponse::CreateUserError(x)) => set_error.set(view! {cx,
                    <ul class="error-messages">
                        <li>{x}</li>
                    </ul>
                }),
                Ok(SignupResponse::Success(x)) => {
                    username.set(Some(x));
                    use_navigate(cx)("/", NavigateOptions::default()).unwrap();
                }
                _ => set_error.set(view! {cx,
                    <ul class="error-messages">
                        <li>"There was a problem, try again later"</li>
                    </ul>
                }),
            }
        }
        tracing::debug!("Signup Effect!");
    });

    view! { cx,
        <Title text="Signup"/>
        <div class="auth-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Sign up"</h1>
                        <p class="text-xs-center">
                            <A href="/login">"Have an account?"</A>
                        </p>

                        {error}

                        <ActionForm action=signup_server_action>
                            <fieldset class="form-group">
                                <input name="username" class="form-control form-control-lg" type="text" placeholder="Your Username" required=true/>
                            </fieldset>
                            <fieldset class="form-group">
                                <input name="email" class="form-control form-control-lg" type="email" placeholder="Email" required=true/>
                            </fieldset>
                            <fieldset class="form-group">
                                <input name="password" class="form-control form-control-lg" type="password" placeholder="Password" required=true/>
                            </fieldset>
                            <input type="submit" class="btn btn-lg btn-primary pull-xs-right" value="Sign up" />
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
