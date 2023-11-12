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
    image: String,
    bio: String,
    email: String,
    password: String,
    confirm_password: String,
) -> Result<SettingsUpdateError, ServerFnError> {
    let user = get_user().await?;
    let username = user.username();
    let user = match update_user_validation(user, image, bio, email, password, &confirm_password) {
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
    confirm_password: &str,
) -> Result<crate::models::User, SettingsUpdateError> {
    if !password.is_empty() {
        if password != confirm_password {
            return Err(SettingsUpdateError::PasswordsNotMatch);
        }
        user = user
            .set_password(password)
            .map_err(SettingsUpdateError::ValidationError)?;
    }

    user.set_email(email)
        .map_err(SettingsUpdateError::ValidationError)?
        .set_bio(bio)
        .map_err(SettingsUpdateError::ValidationError)?
        .set_image(image)
        .map_err(SettingsUpdateError::ValidationError)
}

#[cfg(feature = "ssr")]
async fn get_user() -> Result<crate::models::User, ServerFnError> {
    let Some(username) = crate::auth::get_username() else {
        leptos_axum::redirect("/login");
        return Err(ServerFnError::ServerError(
            "You need to be authenticated".to_string(),
        ));
    };

    crate::models::User::get(username).await.map_err(|x| {
        let err = x.to_string();
        tracing::error!("problem while getting the user {err}");
        ServerFnError::ServerError(err)
    })
}

#[tracing::instrument]
#[server(SettingsGetAction, "/api", "GetJson")]
pub async fn settings_get() -> Result<crate::models::User, ServerFnError> {
    get_user().await
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct UserGet {
    username: String,
    email: String,
    bio: Option<String>,
    image: Option<String>,
}

#[component]
pub fn Settings(logout: crate::auth::LogoutSignal) -> impl IntoView {
    let resource = create_resource(|| (), move |_| settings_get());

    view! {
        <Title text="Settings"/>

        <div class="settings-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Your Settings"</h1>

                        <Suspense fallback=move || view!{<p>"Loading user settings"</p>} >
                            <ErrorBoundary fallback=|_| view!{<p>"There was a problem while fetching settings, try again later"</p>}>
                                {move || {
                                    resource.get().map(move |x| {
                                        x.map(move |user| view!{<SettingsViewForm user />})
                                    })
                                }}
                            </ErrorBoundary>
                        </Suspense>
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

#[component]
fn SettingsViewForm(user: crate::models::User) -> impl IntoView {
    let settings_server_action = create_server_action::<SettingsUpdateAction>();
    let result = settings_server_action.value();
    let error = move || {
        result.with(|x| {
            x.as_ref().map_or(true, |y| {
                y.is_err() || !matches!(y, Ok(SettingsUpdateError::Successful))
            })
        })
    };

    view! {
        <p class="text-xs-center"
            class:text-success=move || !error()
            class:error-messages=error
        >
            <strong>
                {move || result.with(|x| {
                    match x {
                        Some(Ok(SettingsUpdateError::Successful)) => {
                            "Successfully update settings".to_string()
                        },
                        Some(Ok(SettingsUpdateError::ValidationError(x))) => {
                            format!("Problem while validating: {x:?}")
                        },
                        Some(Ok(SettingsUpdateError::PasswordsNotMatch)) => {
                            "Passwords don't match".to_string()
                        },
                        Some(Err(x)) => format!("{x:?}"),
                        None => String::new(),
                    }
                })}
            </strong>
        </p>

        <ActionForm action=settings_server_action on:submit=move |ev| {
            let Ok(data) = SettingsUpdateAction::from_event(&ev) else {
                return ev.prevent_default();
            };
            if let Err(x) = update_user_validation(crate::models::User::default(), data.image, data.bio, data.email, data.password, &data.confirm_password) {
                result.set(Some(Ok(x)));
                ev.prevent_default();
            }
        }>
            <fieldset>
                <fieldset class="form-group">
                    <input name="image" value=user.image() class="form-control" type="text"
                        placeholder="URL of profile picture" />
                </fieldset>
                <fieldset class="form-group">
                    <input disabled value=user.username() class="form-control form-control-lg" type="text"
                        placeholder="Your Name" />
                </fieldset>
                <fieldset class="form-group">
                    <textarea name="bio" class="form-control form-control-lg" rows="8"
                        placeholder="Short bio about you" prop:value=user.bio().unwrap_or_default()>
                    </textarea>
                </fieldset>
                <fieldset class="form-group">
                    <input name="email" value=user.email() class="form-control form-control-lg" type="text"
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
    }
}
