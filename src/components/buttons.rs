use leptos::*;
use leptos_router::*;

#[server(FollowAction, "/api")]
#[tracing::instrument]
pub async fn follow_action(other_user: String) -> Result<bool, ServerFnError> {
    let Some(username) = crate::auth::get_username() else {
        return Err(ServerFnError::ServerError("You need to be authenticated".into()));
    };
    toggle_follow(username, other_user).await.map_err(|x| {
        tracing::error!("problem while updating the database: {x:?}");
        ServerFnError::ServerError("error while updating the follow".into())
    })
}

#[cfg(feature = "ssr")]
#[tracing::instrument]
async fn toggle_follow(current: String, other: String) -> Result<bool, sqlx::Error> {
    let db = crate::database::get_db();
    match sqlx::query!(
        "SELECT * FROM Follows WHERE follower=$1 and influencer=$2",
        current,
        other
    )
    .fetch_one(db)
    .await
    {
        Ok(_) => sqlx::query!(
            "DELETE FROM Follows WHERE follower=$1 and influencer=$2",
            current,
            other
        )
        .execute(db)
        .await
        .map(|_| false),
        Err(sqlx::error::Error::RowNotFound) => sqlx::query!(
            "INSERT INTO Follows(follower, influencer) VALUES ($1, $2)",
            current,
            other
        )
        .execute(db)
        .await
        .map(|_| true),
        Err(x) => Err(x),
    }
}

#[component]
pub fn ButtonFollow(
    logged_user: crate::auth::UsernameSignal,
    author: ReadSignal<String>,
    following: bool,
) -> impl IntoView {
    let follow = create_server_action::<FollowAction>();
    let result_call = follow.value();
    let follow_cond = move || {
        if let Some(x) = result_call.get() {
            match x {
                Ok(x) => x,
                Err(err) => {
                    tracing::error!("problem while following {err:?}");
                    following
                }
            }
        } else {
            following
        }
    };

    view! {
        <Show
            when=move || logged_user.get().unwrap_or_default() != author.get()
            fallback=|| ()
        >
            <ActionForm action=follow class="inline pull-xs-right">
                <input type="hidden" name="other_user" value=move || author.get() />
                <button type="submit" class="btn btn-sm btn-outline-secondary">
                    <Show
                        when=follow_cond
                        fallback=|| view!{<i class="ion-plus-round"></i>" Follow "}
                    >
                        <i class="ion-close-round"></i>" Unfollow "
                    </Show>
                    {move || author.get()}
                </button>
            </ActionForm>
        </Show>
    }
}

#[server(FavAction, "/api")]
#[tracing::instrument]
pub async fn fav_action(slug: String) -> Result<bool, ServerFnError> {
    let Some(username) = crate::auth::get_username() else {
        return Err(ServerFnError::ServerError("You need to be authenticated".into()));
    };
    toggle_fav(slug, username).await.map_err(|x| {
        tracing::error!("problem while updating the database: {x:?}");
        ServerFnError::ServerError("error while updating the follow".into())
    })
}

#[cfg(feature = "ssr")]
#[tracing::instrument]
async fn toggle_fav(slug: String, username: String) -> Result<bool, sqlx::Error> {
    let db = crate::database::get_db();
    match sqlx::query!(
        "SELECT * FROM FavArticles WHERE article=$1 and username=$2",
        slug,
        username
    )
    .fetch_one(db)
    .await
    {
        Ok(_) => sqlx::query!(
            "DELETE FROM FavArticles WHERE article=$1 and username=$2",
            slug,
            username
        )
        .execute(db)
        .await
        .map(|_| false),
        Err(sqlx::error::Error::RowNotFound) => sqlx::query!(
            "INSERT INTO FavArticles(article, username) VALUES ($1, $2)",
            slug,
            username
        )
        .execute(db)
        .await
        .map(|_| true),
        Err(x) => Err(x),
    }
}

#[component]
pub fn ButtonFav(
    username: crate::auth::UsernameSignal,
    article: super::article_preview::ArticleSignal,
) -> impl IntoView {
    let make_fav = create_server_action::<FavAction>();
    let result_make_fav = make_fav.value();
    let fav_count = move || {
        if let Some(x) = result_make_fav.get() {
            match x {
                Ok(result) => {
                    article.update(move |x| {
                        x.fav = !x.fav;
                        x.favorites_count =
                            (x.favorites_count + if result { 1 } else { -1 }).max(0);
                    });
                }
                Err(err) => {
                    tracing::error!("problem while fav {err:?}");
                }
            }
        }
        article.with(|x| x.favorites_count)
    };

    view! {
        <Show
            when=move || username.with(Option::is_some)
            fallback=move || view!{
                <button class="btn btn-sm btn-outline-primary pull-xs-right">
                    <i class="ion-heart"></i>
                    <span class="counter">" ("{fav_count}")"</span>
                </button>
            }
        >
            <ActionForm action=make_fav class="inline pull-xs-right">
                <input type="hidden" name="slug" value=move || article.with(|x| x.slug.to_string()) />
                <button type="submit" class="btn btn-sm btn-outline-primary">
                <Show
                    when=move || article.with(|x| x.fav)
                    fallback=move || {view!{<i class="ion-heart"></i>" Fav "}}
                >
                    <i class="ion-heart-broken"></i>" Unfav "
                </Show>
                <span class="counter">"("{fav_count}")"</span></button>
            </ActionForm>
        </Show>
    }
}
