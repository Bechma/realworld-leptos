use crate::auth::{validate_signup, SignupAction, SignupResponse};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[tracing::instrument]
#[component]
pub fn Signup(cx: Scope, username: RwSignal<Option<String>>) -> impl IntoView {
    let (error, set_error) = create_signal(cx, view! {cx, <ul></ul>});

    let signup_server_action = create_server_action::<SignupAction>(cx);
    let result_of_call = signup_server_action.value();

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
                Ok(SignupResponse::Success) => {
                    username.set(crate::auth::get_username(cx));
                    use_navigate(cx)("/", NavigateOptions::default()).unwrap();
                }
                Err(_) => set_error.set(view! {cx,
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

                        <ActionForm action=signup_server_action on:submit=move |ev| {
                            let Ok(data) = SignupAction::from_event(&ev) else {
                                return ev.prevent_default();
                            };
                            if let Err(x) = validate_signup(data.username, data.email, data.password) {
                                set_error.set(view! {cx,
                                    <ul class="error-messages">
                                        <li>"Problem while validating: "{x}</li>
                                    </ul>
                                });
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
