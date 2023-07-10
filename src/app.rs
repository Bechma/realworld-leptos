use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::routes::*;

#[tracing::instrument]
#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    let username = create_rw_signal(cx, crate::auth::get_username(cx));
    let logout = create_server_action::<LogoutAction>(cx);

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

#[component]
fn NavItems(
    cx: Scope,
    logout: Action<LogoutAction, Result<(), ServerFnError>>,
    username: RwSignal<Option<String>>,
) -> impl IntoView {
    let profile_href = move || {
        username
            .get()
            .map(|x| format!("/profile/{x}"))
            .unwrap_or_default()
    };
    let profile_label = move || username.get().map(|x| format!(" {x}")).unwrap_or_default();
    let logged_style = move || username.get().map(|_| "").unwrap_or("display: none;");
    let anonymous_style = move || username.get().map(|_| "display: none;").unwrap_or("");

    let result_of_call = logout.value();
    create_effect(cx, move |_| {
        let res = result_of_call.get();
        tracing::debug!("Result logout: {:?}", res);
        if let Some(x) = res {
            match x {
                Ok(()) => {
                    username.set(None);
                    request_animation_frame(move || {
                        let route = use_router(cx);
                        let path = route.pathname();
                        let path = path.get_untracked();
                        tracing::debug!("Logout request_animation_frame path: {path}");
                        if path.starts_with("/settings") || path.starts_with("/editor") {
                            use_navigate(cx)("/login", NavigateOptions::default()).unwrap()
                        }
                    })
                }
                Err(err) => tracing::error!("Problem during logout {err:?}"),
            }
        }
    });

    view! {cx,
        <li class="nav-item" style=logged_style>
            <A class="nav-link".to_string() href="/editor"><i class="ion-compose"></i>" New Article"</A>
        </li>
        <li class="nav-item" style=logged_style>
            <A class="nav-link".to_string() href="/settings"><i class="ion-gear-a"></i>" Settings"</A>
        </li>
        <li class="nav-item" style=logged_style>
            <A class="nav-link".to_string() href=profile_href><i class="ion-person"></i>{profile_label}</A>
        </li>
        <li class="nav-item" style=logged_style>
            <ActionForm action=logout>
                <button type="submit" class="nav-link" style="background: none; border: none;">
                    <i class="ion-log-out"></i>" Logout"
                </button>
            </ActionForm>
        </li>
        <li class="nav-item" style=anonymous_style>
            <A class="nav-link".to_string() href="/signup"><i class="ion-plus-round"></i>" Sign up"</A>
        </li>
        <li class="nav-item" style=anonymous_style>
            <A class="nav-link".to_string() href="/login"><i class="ion-log-in"></i>" Login"</A>
        </li>
    }
}
