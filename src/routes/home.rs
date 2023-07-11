use crate::components::ArticlePreview;
use leptos::*;
use leptos_meta::*;

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
    Resource<(String, bool, u32, u32), Result<Vec<crate::models::ArticlePreview>, ServerFnError>>;

/// Renders the home page of your application.
#[component]
pub fn HomePage(cx: Scope, username: RwSignal<Option<String>>) -> impl IntoView {
    let tag = create_rw_signal(cx, String::new());
    let my_feed = create_rw_signal(cx, false);
    let page = create_rw_signal(cx, 0u32);
    let amount = create_rw_signal(cx, 10u32);

    let articles: ArticlesType = create_resource(
        cx,
        move || (tag.get(), my_feed.get(), page.get(), amount.get()),
        move |(tag, my_feed, page, amount)| async move {
            home_articles(cx, page, amount, tag, my_feed).await
        },
    );

    let class_my_feed = move || {
        tracing::debug!("set class_my_feed");
        format!(
            "nav-link {}",
            if username.with(|x| x.is_none()) {
                my_feed.set(false);
                "disabled"
            } else if my_feed.get() {
                "active"
            } else {
                ""
            }
        )
    };
    let class_global_feed =
        move || format!("nav-link {}", if !my_feed.get() { "active" } else { "" });

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
                                    <a class={class_my_feed} href="" on:click=move |_| {if !my_feed.get() && username.with(|x| x.is_some()) {my_feed.set(true)}}>"Your Feed"</a>
                                </li>
                                <li class="nav-item">
                                    <a class={class_global_feed} href="" on:click=move |_| if my_feed.get() {my_feed.set(false)}>"Global Feed"</a>
                                </li>
                                <li class="nav-item pull-xs-right">
                                    <div style="display: inline-block;">
                                        //<input type="hidden" name="page" value="{{params.page}}">
                                        //{% if params.myfeed %}<input type="hidden" name="myfeed" value="true">{% endif %}
                                        //{% if params.tag %}<input type="hidden" name="tag" value="{{params.tag}}">{% endif %}
                                        <button type="submit" class="btn btn-sm btn-outline-primary">
                                            "Articles to display"
                                        </button>
                                        <input style="width: 4em" type="number" name="amount" prop:value=move || amount.get() />
                                    </div>
                                </li>
                            </ul>
                        </div>

                        // {% for a in articles %}
                        // {{macros::preview(article=a)}}
                        // {% endfor %}
                        <ArticlePreviewList username=username articles=articles/>
                    </div>

                    <div class="col-md-3">
                        <div class="sidebar">
                            <h4>"Popular Tags"</h4>
                            <TagList tag=tag />
                        </div>
                    </div>
                    <ul class="pagination">
                        <Show
                            when=move || {page.get() > 0}
                            fallback=|cx| view!{cx, <p>"hello guys"</p>}
                        >
                            <li class="page-item">
                                <a class="btn btn-primary" on:click=move |_| page.update(|x| *x -= 1)>
                                    "<< Previous page"
                                </a>
                            </li>
                        </Show>
                        <Show
                            when=move || {articles.with(cx, |x| x.as_ref().map(|y| y.len()).unwrap_or_default()).unwrap_or_default() < amount.get() as usize}
                            fallback=|_| ()
                        >
                            <li class="page-item">
                                <a class="btn btn-primary" on:click=move |_| page.update(|x| *x += 1)>
                                    "Next page >>"
                                </a>
                            </li>
                        </Show>
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
fn TagList(cx: Scope, tag: RwSignal<String>) -> impl IntoView {
    let tag_list = create_resource(cx, || (), |_| async { get_tags().await });

    // TODO: Wonder if it's possible to reduce reduce the 2x clone
    // TODO: Every click on any tag will trigger the whole For... I just want to re-render for 1 element
    let tag_view = move || {
        let tag_elected = tag.get();
        tag_list.with(cx, move |ts| {
            ts.clone().map(move |tags| {
                view! { cx,
                    <For
                        each=move || tags.clone().into_iter().enumerate()
                        key=|(i, _)| *i
                        view=move |cx, (_, t): (usize, String)| {
                            let class = if t == tag_elected {"tag-pill tag-default tag-primary"} else {"tag-pill tag-default"};
                            let t2 = t.to_string();
                            view!{cx, <a href="" class=class  on:click=move |_| {
                                tag.update(|current_tag| {
                                    tracing::debug!("current_tag={current_tag},new_tag={t}");
                                    *current_tag = if current_tag == &t {
                                        String::new()
                                    } else {
                                        t.to_string()
                                    }
                                })
                            }>{t2}</a>}
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
