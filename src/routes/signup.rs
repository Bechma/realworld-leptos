use crate::auth::{validate_signup, SignupAction, SignupResponse};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[tracing::instrument]
#[component]
pub fn Signup(username: crate::auth::UsernameSignal) -> impl IntoView {
    let signup_server_action = create_server_action::<SignupAction>();
    let result_of_call = signup_server_action.value();

    let error_cb = move || {
        result_of_call
            .get()
            .map(|msg| match msg {
                Ok(SignupResponse::ValidationError(x)) => format!("Problem while validating: {x}"),
                Ok(SignupResponse::CreateUserError(x)) => x,
                Ok(SignupResponse::Success) => {
                    username.set(crate::auth::get_username());
                    request_animation_frame(move || {
                        use_navigate()("/", NavigateOptions::default()).unwrap();
                    });
                    String::new()
                }
                Err(_) => "There was a problem, try again later".into(),
            })
            .unwrap_or_default()
    };

    view! {
        <Title text="Signup"/>
        <div class="auth-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Sign up"</h1>
                        <p class="text-xs-center">
                            <A href="/login">"Have an account?"</A>
                        </p>

                        <p class="error-messages text-xs-center">
                            {error_cb}
                        </p>

                        <ActionForm action=signup_server_action on:submit=move |ev| {
                            let Ok(data) = SignupAction::from_event(&ev) else {
                                return ev.prevent_default();
                            };
                            if let Err(x) = validate_signup(data.username, data.email, data.password) {
                                result_of_call.set(Some(Ok(SignupResponse::ValidationError(x))));
                                ev.prevent_default();
                            }
                        }>
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
