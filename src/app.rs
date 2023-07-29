use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::NavItems;
use crate::routes::*;

#[tracing::instrument]
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let username = create_rw_signal(crate::auth::get_username());

    let logout: crate::auth::LogoutSignal = create_server_action::<crate::auth::LogoutAction>();
    let login: crate::auth::LoginSignal = create_server_action::<crate::auth::LoginAction>();
    let signup: crate::auth::SignupSignal = create_server_action::<crate::auth::SignupAction>();

    let (logout_version, login_version, signup_version) =
        (logout.version(), login.version(), signup.version());

    create_effect(move |_| {
        logout_version.track();
        login_version.track();
        signup_version.track();
        username.set(crate::auth::get_username());
    });

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet href="https://code.ionicframework.com/ionicons/2.0.1/css/ionicons.min.css"/>
        <Stylesheet href="https://fonts.googleapis.com/css?family=Titillium+Web:700|Source+Serif+Pro:400,700|Merriweather+Sans:400,700|Source+Sans+Pro:400,300,600,700,300italic,400italic,600italic,700italic"/>
        <Stylesheet href="https://demo.productionready.io/main.css"/>
        <Stylesheet href="/pkg/realworld-leptos.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <nav class="navbar navbar-light">
                <div class="container">
                    <A class="navbar-brand" href="/" exact=true>"conduit"</A>
                    <ul class="nav navbar-nav pull-xs-right">
                        <NavItems logout username />
                    </ul>
                </div>
            </nav>
            <main>
                <Routes>
                    <Route path="/" view=move || view! { <HomePage username/> }/>
                    <Route path="/login" view=move || view! { <Login login/> }/>
                    <Route path="/signup" view=move || view! { <Signup signup/> }/>
                    <Route path="/settings" view=move || view! { <Settings logout /> }/>
                    <Route path="/editor/:slug?" view=|| view! { <Editor/> }/>
                    <Route path="/article/:slug" view=move || view! { <Article username/> }/>
                    <Route path="/profile/:user" view=move || view! { <Profile username/> }/>
                </Routes>
            </main>
            <footer>
                <div class="container">
                    <A href="/" class="logo-font">"conduit"</A>
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
