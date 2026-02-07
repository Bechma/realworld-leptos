use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{components::A, hooks::use_params_map};

use crate::components::ArticleMeta;

#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
pub struct ArticleResult {
    pub(super) article: crate::models::Article,
    pub(super) logged_user: Option<crate::models::User>,
}

#[server(GetArticleAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn get_article(slug: String) -> Result<Option<ArticleResult>, ServerFnError> {
    let article = match crate::models::Article::for_article(slug).await {
        Ok(article) => article,
        Err(sqlx::Error::RowNotFound) => return Ok(None),
        Err(x) => {
            let err = format!("Error while getting article: {x:?}");
            tracing::error!("{err}");
            return Err(ServerFnError::new(
                "Could not retrieve article, try again later",
            ));
        }
    };

    Ok(Some(ArticleResult {
        article,
        logged_user: crate::auth::current_user().await.ok(),
    }))
}

#[tracing::instrument]
#[component]
pub fn Article(username: crate::auth::UsernameSignal) -> impl IntoView {
    let params = use_params_map();
    let article = Resource::new(
        move || params.get().get("slug").clone().unwrap_or_default(),
        |slug| async { get_article(slug).await },
    );

    let title = RwSignal::new(String::from("Loading"));

    view! {
        <Title text=move || title.get()/>

        <Suspense fallback=move || view! { <p>"Loading Article"</p> }>
            <ErrorBoundary fallback=|_| {
                view! { <p class="error-messages text-xs-center">"Something went wrong, please try again later."</p>}
            }>
                {move || {
                    article.get().map(move |x| {
                        x.map(move |article_result| match article_result {
                            Some(article_result) => {
                                title.set(article_result.article.slug.clone());
                                view! {
                                    <ArticlePage username result=article_result />
                                }
                                .into_any()
                            }
                            None => {
                                title.set("Article not found".to_string());
                                view! { <ArticleNotFound/> }.into_any()
                            }
                        })
                    })
                }}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn ArticleNotFound() -> impl IntoView {
    view! {
        <div class="container page">
            <p class="error-messages text-xs-center">"Article not found."</p>
        </div>
    }
}

#[component]
fn ArticlePage(username: crate::auth::UsernameSignal, result: ArticleResult) -> impl IntoView {
    let article_signal = RwSignal::new(result.article.clone());
    let user_signal = RwSignal::new(result.logged_user);
    let tag_list = result.article.tag_list;

    view! {
        <div class="article-page">
            <div class="banner">
                <div class="container">
                    <h1>{result.article.title}</h1>
                    <ArticleMeta username article=article_signal is_preview=false />
                </div>
            </div>

            <div class="container page">
                <div class="row article-content">
                    <div class="col-md-12">
                        <p>{result.article.body}</p>
                    </div>
                </div>

                <ul class="tag-list">
                    <For
                        each=move || tag_list.clone().into_iter()
                        key=|tag| tag.clone()
                        children=|tag: String| {view!{<li class="tag-default tag-pill tag-outline">{tag}</li>}}
                    />
                </ul>

                <hr />

                <div class="article-actions">
                    <div class="row" style="justify-content: center;">
                        <ArticleMeta username article=article_signal is_preview=false />
                    </div>
                </div>

                <div class="row">
                    <CommentSection username article=article_signal user=user_signal />
                </div>
            </div>
        </div>
    }
}

#[server(PostCommentAction, "/api")]
#[tracing::instrument]
pub async fn post_comment(slug: String, body: String) -> Result<(), ServerFnError> {
    let Some(logged_user) = crate::auth::get_username() else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };

    crate::models::Comment::insert(slug, logged_user, body)
        .await
        .map(|_| ())
        .map_err(|x| {
            let err = format!("Error while posting a comment: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not post a comment, try again later".into())
        })
}

#[server(GetCommentsAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn get_comments(slug: String) -> Result<Vec<crate::models::Comment>, ServerFnError> {
    crate::models::Comment::get_all(slug).await.map_err(|x| {
        let err = format!("Error while posting a comment: {x:?}");
        tracing::error!("{err}");
        ServerFnError::ServerError("Could not post a comment, try again later".into())
    })
}

#[server(DeleteCommentsAction, "/api")]
#[tracing::instrument]
pub async fn delete_comment(id: i32) -> Result<(), ServerFnError> {
    let Some(logged_user) = crate::auth::get_username() else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };

    crate::models::Comment::delete(id, logged_user)
        .await
        .map(|_| ())
        .map_err(|x| {
            let err = format!("Error while posting a comment: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not post a comment, try again later".into())
        })
}

#[component]
fn CommentSection(
    username: crate::auth::UsernameSignal,
    article: crate::components::ArticleSignal,
    user: RwSignal<Option<crate::models::User>>,
) -> impl IntoView {
    let comments_action = ServerAction::<PostCommentAction>::new();
    let result = comments_action.version();
    let reset_comment = RwSignal::new("");
    let comments = Resource::new(
        move || (result.get(), article.with(|a| a.slug.clone())),
        move |(_, a)| async move {
            reset_comment.set("");
            get_comments(a).await.unwrap_or_else(|_| vec![])
        },
    );

    view! {
        <div class="col-xs-12 col-md-8 offset-md-2">
            <Show when=move || username.with(Option::is_some) fallback=|| ()>
                <div class="card comment-form">
                <ActionForm action=comments_action>
                    <input name="slug" type="hidden" value=move || article.with(|x| x.slug.clone()) />
                    <div class="card-block">
                        <textarea name="body" prop:value=move || reset_comment.get() class="form-control" placeholder="Write a comment..." rows="3"></textarea>
                    </div>
                    <div class="card-footer">
                        <img src=move || user.with(|x| x.as_ref().map(crate::models::User::image).unwrap_or_default()) class="comment-author-img" />
                        <button class="btn btn-sm btn-primary" type="submit">
                            "Post Comment"
                        </button>
                    </div>
                </ActionForm>
                </div>
            </Show>
            <Suspense fallback=move || view! {<p>"Loading Comments from the article"</p> }>
                <ErrorBoundary fallback=|_| {
                    view! { <p class="error-messages text-xs-center">"Something went wrong."</p>}
                }>
                    {move || comments.get().map(move |c| {
                        view! {
                            <For each=move || c.clone().into_iter()
                                key=|comment| comment.id
                                children=move |comment: crate::models::Comment| {
                                    let comment = RwSignal::new(comment);
                                    view!{<Comment username comment comments />}
                                }/>
                        }
                    })}
                </ErrorBoundary>
            </Suspense>
        </div>
    }
}

#[component]
fn Comment(
    username: crate::auth::UsernameSignal,
    comment: RwSignal<crate::models::Comment>,
    comments: Resource<Vec<crate::models::Comment>>,
) -> impl IntoView {
    let user_link = move || format!("/profile/{}", comment.with(|x| x.username.clone()));
    let user_image = move || comment.with(|x| x.user_image.clone().unwrap_or_default());
    let delete_c = ServerAction::<DeleteCommentsAction>::new();
    let delete_result = delete_c.value();

    Effect::new(move |_| {
        if let Some(Ok(())) = delete_result.get() {
            tracing::info!("comment deleted!");
            comments.refetch();
        }
    });

    view! {
        <div class="card">
            <div class="card-block">
                <p class="card-text">{move || comment.with(|x| x.body.clone())}</p>
            </div>
            <div class="card-footer">
                <A href=user_link><span class="comment-author">
                    <img src=user_image class="comment-author-img" />
                </span></A>
                " "
                <A href=user_link><span  class="comment-author">{move || comment.with(|x| x.username.clone())}</span></A>
                <span class="date-posted">{move || comment.with(|x| x.created_at.clone())}</span>
                <Show
                    when=move || {username.get().unwrap_or_default() == comment.with(|x| x.username.clone())}
                    fallback=|| ()>
                    <div  class="comment-author">
                    <ActionForm action=delete_c>
                        <input type="hidden" name="id" value=move || comment.with(|x| x.id) />
                        <button class="btn btn-sm" type="submit"><i class="ion-trash-b"></i></button>
                    </ActionForm>
                    </div>
                </Show>
            </div>
        </div>
    }
}
