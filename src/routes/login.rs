use crate::auth::{LoginAction, LoginMessages};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn Login(cx: Scope, username: RwSignal<Option<String>>) -> impl IntoView {
    let login_server_action = create_server_action::<LoginAction>(cx);

    let result_of_call = login_server_action.value();

    let error = move || {
        result_of_call.with(|msg| {
            let err = if let Some(msg) = msg {
                match msg {
                    Ok(LoginMessages::Unsuccessful) => "Incorrect user or password",
                    Ok(LoginMessages::Successful) => {
                        username.set(crate::auth::get_username(cx));
                        request_animation_frame(move || {
                            use_navigate(cx)("/", NavigateOptions::default()).unwrap()
                        });
                        ""
                    }
                    Err(_) => "There was a problem, try again later",
                }
            } else {
                ""
            };
            view! {cx, <li>{err}</li>}
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
                            {error}
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
