use crate::components::ArticlePreview;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[server(HomeAction, "/api", "GetJson")]
async fn home_articles(
    cx: Scope,
    page: u32,
    amount: u32,
    tag: String,
    my_feed: bool,
) -> Result<Vec<crate::models::ArticlePreview>, ServerFnError> {
    let page = i64::from(page);
    let amount = i64::from(amount);

    Ok(
        crate::models::ArticlePreview::for_home_page(cx, page, amount, tag, my_feed)
            .await
            .map_err(|x| {
                tracing::error!("problem while fetching home articles: {x:?}");
                ServerFnError::ServerError("Problem while fetching home articles".into())
            })?,
    )
}

#[server(GetTagsAction, "/api", "GetJson")]
async fn get_tags() -> Result<Vec<String>, ServerFnError> {
    sqlx::query!("SELECT DISTINCT tag FROM ArticleTags")
        .map(|x| x.tag)
        .fetch_all(crate::database::get_db())
        .await
        .map_err(|x| {
            tracing::error!("problem while fetching tags: {x:?}");
            ServerFnError::ServerError("Problem while fetching tags".into())
        })
}

type ArticlesType =
    Resource<crate::models::Pagination, Result<Vec<crate::models::ArticlePreview>, ServerFnError>>;

/// Renders the home page of your application.
#[component]
pub fn HomePage(cx: Scope, username: RwSignal<Option<String>>) -> impl IntoView {
    let pagination = use_query::<crate::models::Pagination>(cx);

    let articles: ArticlesType = create_resource(
        cx,
        move || pagination.get().unwrap_or_default(),
        move |pagination| async move {
            tracing::debug!("making another request: {pagination:?}");
            home_articles(
                cx,
                pagination.get_page(),
                pagination.get_amount(),
                pagination.get_tag().to_string(),
                pagination.get_my_feed(),
            )
            .await
        },
    );

    let your_feed_href = move || {
        if username.with(|x| x.is_some())
            && !pagination.with(|x| x.as_ref().map(|x| x.get_my_feed()).unwrap_or_default())
        {
            pagination
                .get()
                .unwrap_or_default()
                .set_my_feed(true)
                .to_string()
        } else {
            "".into()
        }
    };
    let your_feed_class = move || {
        tracing::debug!("set class_my_feed");
        format!(
            "nav-link {}",
            if username.with(|x| x.is_none()) {
                "disabled"
            } else if pagination.with(|x| x.as_ref().map(|x| x.get_my_feed()).unwrap_or_default()) {
                "active"
            } else {
                ""
            }
        )
    };

    view! { cx,
        <Title text="Home"/>

        <div class="home-page">
            <div class="banner">
                <div class="container">
                    <h1 class="logo-font">conduit</h1>
                    <p>"A place to share your knowledge."</p>
                </div>
            </div>

            <div class="container page">
                <div class="row">
                    <div class="col-md-9">
                        <div class="feed-toggle">
                            <ul class="nav nav-pills outline-active">
                                <li class="nav-item">
                                    <a href=your_feed_href class=your_feed_class>
                                        "Your Feed"
                                    </a>
                                </li>
                                <li class="nav-item">
                                    <a class="nav-link"
                                    class:active=move || !pagination.with(|x| x.as_ref().map(|x| x.get_my_feed()).unwrap_or_default())
                                    href=move || pagination.get().unwrap_or_default().set_my_feed(false).to_string()>
                                        "Global Feed"
                                    </a>
                                </li>
                                <li class="nav-item pull-xs-right">
                                    <div style="display: inline-block;">
                                        "Articles to display | "
                                        <a href=move || pagination.get().unwrap_or_default().set_amount(1).to_string() class="btn btn-primary">"1"</a>
                                        <a href=move || pagination.get().unwrap_or_default().set_amount(20).to_string() class="btn btn-primary">"20"</a>
                                        <a href=move || pagination.get().unwrap_or_default().set_amount(50).to_string() class="btn btn-primary">"50"</a>
                                    </div>
                                </li>
                            </ul>
                        </div>

                        <ArticlePreviewList username=username articles=articles/>
                    </div>

                    <div class="col-md-3">
                        <div class="sidebar">
                            <h4>"Popular Tags"</h4>
                            <TagList />
                        </div>
                    </div>

                    <ul class="pagination">
                        <Show
                            when=move || {pagination.with(|x| x.as_ref().map(|y| y.get_page()).unwrap_or_default()) > 0}
                            fallback=|_| ()
                        >
                            <li class="page-item">
                                <a class="btn btn-primary" href=move || pagination.get().unwrap_or_default().previous_page().to_string()>
                                    "<< Previous page"
                                </a>
                            </li>
                        </Show>
                        <Suspense fallback=|| ()>
                            <Show
                                // TODO: fix this dummy logic
                                when=move || {
                                    articles.with(cx, |x| x.as_ref().map(|y| y.len()).unwrap_or_default()).unwrap_or_default()
                                    >=
                                    pagination.with(|x| x.as_ref().map(|y| y.get_page()).unwrap_or_default()) as usize}
                                fallback=|_| ()
                            >
                                <li class="page-item">
                                    <a class="btn btn-primary" href=move || pagination.get().unwrap_or_default().next_page().to_string()>
                                        "Next page >>"
                                    </a>
                                </li>
                            </Show>
                        </Suspense>
                    </ul>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ArticlePreviewList(
    cx: Scope,
    username: RwSignal<Option<String>>,
    articles: ArticlesType,
) -> impl IntoView {
    // TODO: When the user logouts in the main screen, there's a request to articles... WHY?
    let articles_view = move || {
        articles.with(cx, move |x| {
            x.clone().map(move |res| {
                view! {cx,
                    <For
                        each=move || res.clone().into_iter().enumerate()
                        key=|(i, _)| *i
                        view=move |cx, (_, article): (usize, crate::models::ArticlePreview)| {
                            let article = create_rw_signal(cx, article);
                            view! {cx,
                                <ArticlePreview article=article username=username />
                            }
                        }
                    />
                }
            })
        })
    };

    view! {cx,
        <Suspense fallback=move || view! {cx, <p>"Loading Articles"</p> }>
            <ErrorBoundary fallback=|cx, _| {
                view! { cx,
                    <ul class="error-messages">
                        <li>"Something went wrong."</li>
                    </ul>
                }
            }>
                {articles_view}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn TagList(cx: Scope) -> impl IntoView {
    let pagination = use_query::<crate::models::Pagination>(cx);
    let tag_list = create_resource(cx, || (), |_| async { get_tags().await });

    // TODO: Wonder if it's possible to reduce reduce the 2x clone
    // TODO: Every click on any tag will trigger the whole For... I just want to re-render for 1 element
    let tag_view = move || {
        let tag_elected = pagination.with(|x| {
            x.as_ref()
                .map(|y| y.get_tag())
                .unwrap_or_default()
                .to_string()
        });
        tag_list.with(cx, move |ts| {
            ts.clone().map(move |tags| {
                view! { cx,
                    <For
                        each=move || tags.clone().into_iter().enumerate()
                        key=|(i, _)| *i
                        view=move |cx, (_, t): (usize, String)| {
                            let class = if t == tag_elected {"tag-pill tag-default tag-primary"} else {"tag-pill tag-default"};
                            let t2 = t.to_string();
                            view!{cx,
                                <A class=class href=move || pagination.with(|x| x.clone().unwrap_or_default().set_tag(&t).to_string())>
                                    {t2}
                                </A>
                            }
                        }
                    />
                }
            })
        })
    };

    view! { cx,
        <div class="tag-list">
            <Suspense fallback=move || view! {cx, <p>"Loading Tags"</p> }>
                <ErrorBoundary fallback=|cx, _| {
                    view! { cx,
                        <ul class="error-messages">
                            <li>"Something went wrong."</li>
                        </ul>
                    }
                }>
                    {tag_view}
                </ErrorBoundary>
            </Suspense>
        </div>
    }
}
