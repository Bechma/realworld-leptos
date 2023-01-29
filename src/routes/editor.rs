use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum EditorResponse {
    ValidationError(String),
    CreateUserError(String),
    Success,
}

#[cfg(feature = "ssr")]
fn validate_article(username: String, email: String, password: String) -> Result<(), String> {
    Ok(())
}

#[server(EditorAction, "/api")]
pub async fn editor_action(
    cx: Scope,
    username: String,
    email: String,
    password: String,
) -> Result<EditorResponse, ServerFnError> {
    if let Err(x) = validate_article(username.clone(), email.clone(), password.clone()) {
        return Ok(EditorResponse::ValidationError(x));
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
            super::set_username(cx, username).await;
            Ok(EditorResponse::Success)
        }
        Err(x) => {
            let x = x.to_string();
            Ok(if x.contains("users_email_key") {
                EditorResponse::CreateUserError("Duplicated email".to_string())
            } else if x.contains("users_pkey") {
                EditorResponse::CreateUserError("Duplicated user".to_string())
            } else {
                EditorResponse::CreateUserError(
                    "There is an unknown problem, try again later".to_string(),
                )
            })
        }
    }
}

#[component]
pub fn Editor(cx: Scope) -> impl IntoView {
    let (error, set_error) = create_signal(cx, view! {cx, <ul></ul>});

    let editor_server_action = create_server_action::<EditorAction>(cx);
    let result_of_call = editor_server_action.value();

    let navigate = use_navigate(cx);
    let username = use_context::<crate::app::AuthState>(cx).unwrap();

    create_effect(cx, move |_| {
        let r = result_of_call();
        if super::get_username(cx).is_none() {
            navigate("/login", NavigateOptions::default()).unwrap();
            log::debug!("You need to login");
            return;
        }
        if let Some(msg) = r {
            match msg {
                Ok(EditorResponse::ValidationError(x)) => set_error(view! {cx,
                    <ul class="error-messages">
                        <li>"Problem while validating: "{x}</li>
                    </ul>
                }),
                Ok(EditorResponse::CreateUserError(x)) => set_error(view! {cx,
                    <ul class="error-messages">
                        <li>{x}</li>
                    </ul>
                }),
                _ => set_error(view! {cx,
                    <ul class="error-messages">
                        <li>"There was a problem, try again later"</li>
                    </ul>
                }),
            }
        }
        log::debug!("Editor Effect!");
    });

    view! { cx,
        <Title text="Editor"/>
        <div class="editor-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-10 offset-md-1 col-xs-12">
                        <ActionForm action=editor_server_action>
                            <fieldset>
                                <fieldset class="form-group">
                                    <input name="title" type="text" class="form-control form-control-lg"
                                        placeholder="Article Title" />
                                </fieldset>
                                <fieldset class="form-group">
                                    <input name="description" type="text" class="form-control"
                                        placeholder="What's this article about?" />
                                </fieldset>
                                <fieldset class="form-group">
                                    <textarea name="body" class="form-control" rows="8"
                                        placeholder="Write your article (in markdown)"></textarea>
                                </fieldset>
                                <fieldset class="form-group">
                                    <input name="tagList" type="text" class="form-control"
                                        placeholder="Enter tags(space separated)" />
                                </fieldset>
                                <button class="btn btn-lg pull-xs-right btn-primary" type="submit">
                                    "Publish Article"
                                </button>
                            </fieldset>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
