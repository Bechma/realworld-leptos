use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::NavItems;
use crate::routes::*;

#[tracing::instrument]
#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    let username = create_rw_signal(cx, crate::auth::get_username(cx));
    let logout = create_server_action::<crate::auth::LogoutAction>(cx);

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet href="https://code.ionicframework.com/ionicons/2.0.1/css/ionicons.min.css"/>
        <Stylesheet href="https://fonts.googleapis.com/css?family=Titillium+Web:700|Source+Serif+Pro:400,700|Merriweather+Sans:400,700|Source+Sans+Pro:400,300,600,700,300italic,400italic,600italic,700italic"/>
        <Stylesheet href="https://demo.productionready.io/main.css"/>
        <Stylesheet id="leptos" href="/pkg/realworld-leptos.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <nav class="navbar navbar-light">
                <div class="container">
                    <A class="navbar-brand".to_string() href="/" exact=true>"conduit"</A>
                    <ul class="nav navbar-nav pull-xs-right">
                        <li class="nav-item">
                            <A class="nav-link".to_string() href="/" exact=true><i class="ion-home"></i>" Home"</A>
                        </li>
                        <NavItems logout=logout username=username />
                    </ul>
                </div>
            </nav>
            <main>
                <Routes>
                    <Route path="/" view=move |cx| view! { cx, <HomePage username=username/> }/>
                    <Route path="/login" view=move |cx| view! { cx, <Login username=username/> }/>
                    <Route path="/signup" view=move |cx| view! { cx, <Signup username=username/> }/>
                    <Route path="/settings" view=move |cx| view! { cx, <Settings logout=logout /> }/>
                    <Route path="/editor/:slug?" view=|cx| view! { cx, <Editor/> }/>
                </Routes>
            </main>
            <footer>
                <div class="container">
                    <a href="/" class="logo-font">"conduit"</a>
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
