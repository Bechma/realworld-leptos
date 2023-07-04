use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum LoginMessages {
    Successful,
    Unsuccessful,
}

#[tracing::instrument]
#[server(LoginAction, "/api")]
pub async fn login_action(
    cx: Scope,
    username: String,
    password: String,
) -> Result<LoginMessages, ServerFnError> {
    let mut conn = crate::database::get_db().acquire().await.unwrap();

    if sqlx::query_scalar!(
        "SELECT username FROM Users where username=$1 and password=crypt($2, password)",
        username,
        password,
    )
    .fetch_one(&mut conn)
    .await
    .unwrap_or_default()
        == username
    {
        super::set_username(cx, username).await;
        Ok(LoginMessages::Successful)
    } else {
        let response_options = use_context::<leptos_axum::ResponseOptions>(cx).unwrap();
        response_options.set_status(axum::http::StatusCode::FORBIDDEN);
        Ok(LoginMessages::Unsuccessful)
    }
}

#[tracing::instrument]
#[server(LogoutAction, "/api")]
pub async fn logout_action(cx: Scope) -> Result<(), ServerFnError> {
    let response_options = use_context::<leptos_axum::ResponseOptions>(cx).unwrap();
    response_options.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(
            "session=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT",
        )
        .expect("header value couldn't be set"),
    );
    Ok(())
}

#[tracing::instrument]
#[component]
pub fn Login(cx: Scope) -> impl IntoView {
    let (error, set_error) = create_signal(cx, view! {cx, <ul></ul>});
    let login_server_action = create_server_action::<LoginAction>(cx);

    let result_of_call = login_server_action.value();

    let username_set = use_context::<crate::app::AuthState>(cx).unwrap();
    let username_set = username_set.username_set;

    create_effect(cx, move |_| {
        let r = result_of_call.get();
        request_animation_frame(move || {
            let navigate = use_navigate(cx);
            if let Some(user) = super::get_username(cx) {
                tracing::debug!("You are logged");
                username_set.set(Some(user));
                navigate("/", NavigateOptions::default()).unwrap();
                return;
            }
            if let Some(msg) = r {
                match msg {
                    Ok(LoginMessages::Unsuccessful) => set_error.set(view! {cx,
                        <ul class="error-messages">
                            <li>"Incorrect user or password"</li>
                        </ul>
                    }),
                    _ => set_error.set(view! {cx,
                        <ul class="error-messages">
                            <li>"There was a problem, try again later"</li>
                        </ul>
                    }),
                }
            }
        });
        tracing::debug!("Login Effect!");
    });

    view! { cx,
        <Title text="Login"/>
        <div class="auth-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Login"</h1>

                        {error}

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
