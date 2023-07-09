use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum SettingsUpdateError {
    PasswordsNotMatch,
    Successful,
    ValidationError(String),
}

#[tracing::instrument]
#[server(SettingsUpdateAction, "/api")]
pub async fn settings_update(
    cx: Scope,
    image: String,
    bio: String,
    email: String,
    password: String,
    confirm_password: String,
) -> Result<SettingsUpdateError, ServerFnError> {
    let user = get_user(cx).await?;
    let username = user.username();
    let user = match update_user_validation(user, image, bio, email, password, confirm_password) {
        Ok(x) => x,
        Err(x) => return Ok(x),
    };
    user.update()
        .await
        .map(|_| SettingsUpdateError::Successful)
        .map_err(move |x| {
            tracing::error!(
                "Problem while updating user: {} with error {}",
                username,
                x.to_string()
            );
            ServerFnError::ServerError("Problem while updating user".into())
        })
}

fn update_user_validation(
    mut user: crate::models::User,
    image: String,
    bio: String,
    email: String,
    password: String,
    confirm_password: String,
) -> Result<crate::models::User, SettingsUpdateError> {
    if !password.is_empty() {
        if password != confirm_password {
            return Err(SettingsUpdateError::PasswordsNotMatch);
        } else {
            user = user
                .set_password(password)
                .map_err(SettingsUpdateError::ValidationError)?;
        }
    }
    Ok(user
        .set_email(email)
        .map_err(SettingsUpdateError::ValidationError)?
        .set_bio(bio)
        .set_image(image))
}

#[cfg(feature = "ssr")]
async fn get_user(cx: Scope) -> Result<crate::models::User, ServerFnError> {
    let Some(username) = crate::auth::get_username(cx) else {
        leptos_axum::redirect(cx, "/login");
        return Err(ServerFnError::ServerError("You need to be authenticated".to_string()));
    };

    crate::models::User::get(username).await.map_err(|x| {
        let err = x.to_string();
        tracing::error!("problem while getting the user {err}");
        ServerFnError::ServerError(err)
    })
}

#[tracing::instrument]
#[server(SettingsGetAction, "/api", "GetJson")]
pub async fn settings_get(cx: Scope) -> Result<crate::models::User, ServerFnError> {
    get_user(cx).await
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct UserGet {
    username: String,
    email: String,
    bio: Option<String>,
    image: Option<String>,
}

#[component]
pub fn Settings(
    cx: Scope,
    logout: Action<super::login::LogoutAction, Result<(), ServerFnError>>,
) -> impl IntoView {
    let settings_server_action = create_server_action::<SettingsUpdateAction>(cx);
    let (user, user_set) = create_signal(cx, crate::models::User::default());
    let error = create_rw_signal(cx, view! {cx, <ul></ul>});
    spawn_local(async move {
        let user = settings_get(cx).await;
        tracing::debug!("user received: {user:?}");
        user_set.set(user.unwrap_or_default());
    });
    tracing::debug!("Settings route");

    view! { cx,
        <Title text="Settings"/>

        <div class="settings-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Your Settings"</h1>

                        {error}

                        <ActionForm action=settings_server_action on:submit=move |ev| {
                            let Ok(data) = SettingsUpdateAction::from_event(&ev) else {
                                return ev.prevent_default();
                            };
                            if let Err(x) = update_user_validation(crate::models::User::default(), data.image, data.bio, data.email, data.password, data.confirm_password) {
                                let x = format!("{x:?}");
                                error.set(view! {cx,
                                    <ul class="error-messages">
                                        <li>"Problem while validating: "{x}</li>
                                    </ul>
                                });
                                ev.prevent_default();
                            }
                        }>
                            <fieldset>
                                <fieldset class="form-group">
                                    <input name="image" value=move || {user.with(|x| x.image().unwrap_or_default())} class="form-control" type="text"
                                        placeholder="URL of profile picture" />
                                </fieldset>
                                <fieldset class="form-group">
                                    <input disabled value=move || {user.with(|x| x.username())} class="form-control form-control-lg" type="text"
                                        placeholder="Your Name" />
                                </fieldset>
                                <fieldset class="form-group">
                                    <textarea name="bio" class="form-control form-control-lg" rows="8"
                                        placeholder="Short bio about you" prop:value=move || user.with(|x| x.bio().unwrap_or_default())>
                                    </textarea>
                                </fieldset>
                                <fieldset class="form-group">
                                    <input name="email" value={move || user.with(|x| x.email())} class="form-control form-control-lg" type="text"
                                        placeholder="Email" />
                                </fieldset>
                                <fieldset class="form-group">
                                    <input name="password" class="form-control form-control-lg" type="password"
                                        placeholder="New Password" />
                                    <input name="confirm_password" class="form-control form-control-lg" type="password"
                                        placeholder="Confirm New Password" />
                                </fieldset>
                                <button class="btn btn-lg btn-primary pull-xs-right" type="submit">"Update Settings"</button>
                            </fieldset>
                        </ActionForm>
                        <hr />
                        <ActionForm action=logout>
                            <button type="submit" class="btn btn-outline-danger">"Or click here to logout."</button>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
