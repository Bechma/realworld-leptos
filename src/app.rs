use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet};
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::path;

use crate::components::NavItems;
use crate::routes::{Article, Editor, HomePage, Login, Profile, ResetPassword, Settings, Signup};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <link rel="shortcut icon" type="image/ico" href="/favicon.ico"/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[tracing::instrument]
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let username: crate::auth::UsernameSignal = RwSignal::new(None);

    let logout: crate::auth::LogoutSignal = ServerAction::<crate::auth::LogoutAction>::new();
    let login: crate::auth::LoginSignal = ServerAction::<crate::auth::LoginAction>::new();
    let signup: crate::auth::SignupSignal = ServerAction::<crate::auth::SignupAction>::new();

    let (logout_version, login_version, signup_version) =
        (logout.version(), login.version(), signup.version());

    let user = Resource::new(
        move || {
            (
                logout_version.get(),
                login_version.get(),
                signup_version.get(),
            )
        },
        move |_| {
            tracing::debug!("fetch user");
            crate::auth::current_user()
        },
    );

    Effect::new(move |_| {
        if let Some(user_result) = user.get() {
            let next_username = user_result.ok().map(|user| user.username());
            if username.get_untracked() != next_username {
                username.set(next_username);
            }
        }
    });

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet href="https://code.ionicframework.com/ionicons/2.0.1/css/ionicons.min.css"/>
        <Stylesheet href="https://fonts.googleapis.com/css?family=Titillium+Web:700|Source+Serif+Pro:400,700|Merriweather+Sans:400,700|Source+Sans+Pro:400,300,600,700,300italic,400italic,600italic,700italic"/>
        <Stylesheet href="https://demo.productionready.io/main.css"/>
        <Stylesheet href="/pkg/realworld-leptos.css"/>

        <Router>
            <nav class="navbar navbar-light">
                <div class="container">
                    <A href="/" exact=true><span class="navbar-brand">"conduit"</span></A>
                    <ul class="nav navbar-nav pull-xs-right">
                        <Transition fallback=|| view!{<p>"Loading Navigation bar"</p>}>
                        {move || user.get().map(move |_| {
                            view! {
                                <NavItems logout username />
                            }
                        })}
                        </Transition>
                    </ul>
                </div>
            </nav>
            <main>
                <Routes fallback=|| ()>
                    <Route path=path!("/") view=move || view! {
                        <Transition fallback=|| view!{<p>"Loading HomePage"</p>}>
                        {move || user.get().map(move |_| {
                            view! {
                                <HomePage username/>
                            }
                        })}
                        </Transition>
                    }/>
                    <Route path=path!("/login") view=move || view! { <Login login/> }/>
                    <Route path=path!("/reset_password") view=move || view! { <ResetPassword/> }/>
                    <Route path=path!("/signup") view=move || view! { <Signup signup/> }/>
                    <Route path=path!("/settings") view=move || view! { <Settings logout /> }/>
                    <Route path=path!("/editor/:slug?") view=|| view! { <Editor/> }/>
                    <Route path=path!("/article/:slug") view=move || view! {
                        <Transition fallback=|| view!{<p>"Loading Article"</p>}>
                        {move || user.get().map(move |_| {
                            view! {
                                <Article username/>
                            }
                        })}
                        </Transition>
                    }/>
                    <Route path=path!("/profile/:user") view=move || view! {
                        <Transition fallback=|| view!{<p>"Loading Profile"</p>}>
                        {move || user.get().map(move |_| {
                            view! {
                                <Profile username/>
                            }
                        })}
                        </Transition>
                    }/>
                </Routes>
            </main>
            <footer>
                <div class="container">
                    <A href="/"><span class="logo-font">"conduit"</span></A>
                    <span class="attribution">
                        "An interactive learning project from "
                        <a href="https://thinkster.io">"Thinkster"</a>
                        ". Code &amp; design licensed under MIT."
                    </span>
                </div>
            </footer>
        </Router>
    }
}
