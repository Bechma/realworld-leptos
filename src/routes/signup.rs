use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::auth::{SignupAction, SignupResponse, SignupSignal, validate_signup};

#[component]
pub fn Signup(signup: SignupSignal) -> impl IntoView {
    let result_of_call = signup.value();

    let error_cb = move || {
        result_of_call
            .get()
            .map(|msg| match msg {
                Ok(SignupResponse::ValidationError(x)) => format!("Problem while validating: {x}"),
                Ok(SignupResponse::CreateUserError(x)) => {
                    format!("Problem while creating user: {x}")
                }
                Ok(SignupResponse::Success) => {
                    tracing::info!("Signup success! redirecting");
                    "Done".into()
                }
                Err(x) => {
                    tracing::error!("Problem during signup: {x:?}");
                    "There was a problem, try again later".into()
                }
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

                        <ActionForm action=signup on:submit=move |ev| {
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
                            <button class="btn btn-lg btn-primary pull-xs-right">"Sign up"</button>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
