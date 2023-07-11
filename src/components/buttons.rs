use leptos::*;

#[component]
pub fn ButtonFollow(
    cx: Scope,
    username: crate::auth::UsernameSignal,
    article: super::article_preview::ArticleSignal,
) -> impl IntoView {
    view! {cx,
        <Show
            when=move || username.get() == article.with(|x| Some(x.author.username.to_string()))
            fallback=|_| ()
        >
            //<form method="post" action="{{routes.profile ~ '/' ~ user}}{% if following %}/unfollow{% else %}/follow{% endif %}"
            <form
                style="display: inline-block;" class="pull-xs-right">
                <button type="submit" class="btn btn-sm btn-outline-secondary">
                    <Show
                        when=move || article.with(|x| x.author.following)
                        fallback=|cx| view!{cx, <i class="ion-plus-round"></i>" Follow"}
                    >
                        <i class="ion-close-round"></i>" Unfollow"
                    </Show>
                    {article.with(|x| x.author.username.to_string())}
                </button>
            </form>
        </Show>
    }
}

#[component]
pub fn ButtonFav(
    cx: Scope,
    username: crate::auth::UsernameSignal,
    article: super::article_preview::ArticleSignal,
) -> impl IntoView {
    let fav = move || article.with(|x| x.fav);
    let fav_count = move || article.with(|x| x.favorites_count.unwrap_or_default());
    let heart_class = move || format!("ion-heart{}", if fav() { "-broken" } else { "" });

    view! {cx,
        <Show
            when=move || username.with(|x| x.is_some())
            fallback=move |cx| view!{cx,
                <button class="btn btn-sm btn-outline-primary pull-xs-right">
                    <i class="ion-heart"></i>
                    " "
                    <span class="counter">"("{fav_count}")"</span>
                </button>
            }
        >
            //<form method="post" action="{{routes.article ~ '/' ~ article.slug}}{% if article.fav %}/unfav{% else %}/fav{% endif %}"
            <form
                style="display: inline-block;" class="pull-xs-right">
                <button type="submit" class="btn btn-sm btn-outline-primary">
                <i class=heart_class></i>
                {move || if fav() {" Unfav "} else {" Fav "}}
                <span class="counter">"("{fav_count}")"</span></button>
            </form>
        </Show>
    }
}
