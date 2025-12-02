use crate::auth::{LogoutSignal, UsernameSignal};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub(crate) fn NavItems(logout: LogoutSignal, username: UsernameSignal) -> impl IntoView {
    let profile_label = move || username.get().unwrap_or_default();
    let profile_href = move || format!("/profile/{}", profile_label());

    view! {
        <li class="nav-item">
            <A href="/" exact=true><span class="nav-link"><i class="ion-home"></i>" Home"</span></A>
        </li>
        <Show when=move || username.with(Option::is_none) fallback=move || {
            view!{
                <li class="nav-item">
                    <A href="/editor"><span class="nav-link"><i class="ion-compose"></i>" New Article"</span></A>
                </li>
                <li class="nav-item">
                    <A href="/settings"><span class="nav-link"><i class="ion-gear-a"></i>" Settings"</span></A>
                </li>
                <li class="nav-item">
                    <A href=profile_href><span class="nav-link"><i class="ion-person"></i>" "{profile_label}</span></A>
                </li>
                <li class="nav-item">
                    <ActionForm action=logout>
                        <button class="nav-link" style="background: none; border: none;">
                            <i class="ion-log-out"></i>" Logout"
                        </button>
                    </ActionForm>
                </li>
            }
        }>
            <li class="nav-item">
                <A href="/signup"><span class="nav-link"><i class="ion-plus-round"></i>" Sign up"</span></A>
            </li>
            <li class="nav-item">
                <A href="/login"><span class="nav-link"><i class="ion-log-in"></i>" Login"</span></A>
            </li>
        </Show>
    }
}
