use crate::auth::{LoginMessages, LoginSignal};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn Login(login: LoginSignal) -> impl IntoView {
    let result_of_call = login.value();

    let error = move || {
        result_of_call.with(|msg| {
            msg.as_ref()
                .map(|inner| match inner {
                    Ok(LoginMessages::Unsuccessful) => "Incorrect user or password",
                    // We don't receive the success when it's redirecting... https://github.com/leptos-rs/leptos/issues/1513
                    Ok(LoginMessages::Successful) | Err(ServerFnError::Deserialization(_)) => {
                        unreachable!("login success!")
                    }
                    Err(x) => {
                        tracing::error!("Problem during login: {x:?}");
                        "There was a problem, try again later"
                    }
                })
                .unwrap_or_default()
        })
    };

    view! {
        <Title text="Login"/>
        <div class="auth-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Login"</h1>

                        <p class="error-messages text-xs-center">
                            {error}
                        </p>

                        <ActionForm action=login>
                            <fieldset class="form-group">
                                <input name="username" class="form-control form-control-lg" type="text"
                                    placeholder="Your Username" />
                            </fieldset>
                            <fieldset class="form-group">
                                <input name="password" class="form-control form-control-lg" type="password"
                                    placeholder="Password" />
                            </fieldset>
                            <button class="btn btn-lg btn-primary pull-xs-right">"Sign in"</button>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
