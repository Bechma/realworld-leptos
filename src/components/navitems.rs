use leptos::*;
use leptos_router::*;

#[component]
pub(crate) fn NavItems(
    logout: crate::auth::LogoutSignal,
    username: crate::auth::UsernameSignal,
) -> impl IntoView {
    let profile_label = move || username.get().unwrap_or_default();
    let profile_href = move || format!("/profile/{}", profile_label());
    let logged_style = move || username.get().map(|_| "").unwrap_or("display: none;");
    let anonymous_style = move || username.get().map(|_| "display: none;").unwrap_or("");

    let result_of_call = logout.value();
    create_effect(move |_| {
        let res = result_of_call.get();
        tracing::debug!("Result logout: {:?}", res);
        if let Some(x) = res {
            match x {
                Ok(()) => {
                    username.set(None);
                    request_animation_frame(move || {
                        let route = use_router();
                        let path = route.pathname();
                        let path = path.get_untracked();
                        tracing::debug!("Logout request_animation_frame path: {path}");
                        if path.starts_with("/settings") || path.starts_with("/editor") {
                            use_navigate()("/login", NavigateOptions::default()).unwrap();
                        }
                    });
                }
                Err(err) => tracing::error!("Problem during logout {err:?}"),
            }
        }
    });

    view! {
        <li class="nav-item">
            <A class="nav-link".to_string() href="/" exact=true><i class="ion-home"></i>" Home"</A>
        </li>
        <li class="nav-item" style=logged_style>
            <A class="nav-link".to_string() href="/editor"><i class="ion-compose"></i>" New Article"</A>
        </li>
        <li class="nav-item" style=logged_style>
            <A class="nav-link".to_string() href="/settings"><i class="ion-gear-a"></i>" Settings"</A>
        </li>
        <li class="nav-item" style=logged_style>
            <A class="nav-link".to_string() href=profile_href><i class="ion-person"></i>" "{profile_label}</A>
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
