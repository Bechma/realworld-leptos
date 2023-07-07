use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub enum LoginMessages {
    Successful,
    Unsuccessful,
}

#[server(LoginAction, "/api")]
#[tracing::instrument]
pub async fn login_action(
    cx: Scope,
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
        super::set_username(cx, username).await;
        // leptos_axum::redirect(cx, "/"); // TODO remove when it doesn't provoke a full app reload
        Ok(LoginMessages::Successful)
    } else {
        let response_options = use_context::<leptos_axum::ResponseOptions>(cx).unwrap();
        response_options.set_status(axum::http::StatusCode::FORBIDDEN);
        Ok(LoginMessages::Unsuccessful)
    }
}

#[server(LogoutAction, "/api")]
#[tracing::instrument]
pub async fn logout_action(cx: Scope) -> Result<(), ServerFnError> {
    let response_options = use_context::<leptos_axum::ResponseOptions>(cx).unwrap();
    response_options.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(crate::auth::REMOVE_COOKIE)
            .expect("header value couldn't be set"),
    );
    Ok(())
}

#[tracing::instrument]
#[component]
pub fn Login(cx: Scope) -> impl IntoView {
    let login_server_action = create_server_action::<LoginAction>(cx);

    let result_of_call = login_server_action.value();

    let error = move || {
        result_of_call.with(|msg| {
            if let Some(msg) = msg {
                match msg {
                    Ok(LoginMessages::Unsuccessful) => "Incorrect user or password",
                    Ok(LoginMessages::Successful) => {
                        use_context::<crate::app::AuthState>(cx)
                            .unwrap()
                            .set(super::get_username(cx));
                        request_animation_frame(move || {
                            use_navigate(cx)("/", NavigateOptions::default()).unwrap()
                        });
                        ""
                    }
                    Err(_) => "There was a problem, try again later",
                }
            } else {
                ""
            }
        })
    };

    view! { cx,
        <Title text="Login"/>
        <div class="auth-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Login"</h1>

                        <ul class="error-messages">
                            <li>{error}</li>
                        </ul>

                        <ActionForm action=login_server_action>
                            <fieldset class="form-group">
                                <input name="username" class="form-control form-control-lg" type="text"
                                    placeholder="Your Username" />
                            </fieldset>
                            <fieldset class="form-group">
                                <input name="password" class="form-control form-control-lg" type="password"
                                    placeholder="Password" />
                            </fieldset>
                            <input type="submit" class="btn btn-lg btn-primary pull-xs-right" value="Sign in" />
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
