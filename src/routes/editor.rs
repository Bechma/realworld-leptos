use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum EditorResponse {
    ValidationError(String),
    UpdateError,
    Success(String),
}

#[cfg_attr(feature = "hydrate", allow(dead_code))]
#[derive(Debug)]
struct ArticleUpdate {
    title: String,
    description: String,
    body: String,
    tag_list: std::collections::HashSet<String>,
}

#[tracing::instrument]
fn validate_article(
    title: String,
    description: String,
    body: String,
    tag_list: String,
) -> Result<ArticleUpdate, String> {
    if title.len() < 4 {
        return Err("You need to provide a title with at least 4 characters".into());
    }

    if description.len() < 4 {
        return Err("You need to provide a description with at least 4 characters".into());
    }

    if body.len() < 10 {
        return Err("You need to provide a body with at least 10 characters".into());
    }

    let tag_list = tag_list
        .trim()
        .split_ascii_whitespace()
        .filter(|x| !x.is_empty())
        .map(str::to_string)
        .collect::<std::collections::HashSet<String>>();
    Ok(ArticleUpdate {
        title,
        description,
        body,
        tag_list,
    })
}

#[cfg(feature = "ssr")]
#[tracing::instrument]
async fn update_article(
    author: String,
    slug: String,
    article: ArticleUpdate,
) -> Result<String, sqlx::Error> {
    static BIND_LIMIT: usize = 65535;
    let mut transaction = crate::database::get_db().begin().await?;
    let (rows_affected, slug) = if !slug.is_empty() {
        (
            sqlx::query!(
                "UPDATE Articles SET title=$1, description=$2, body=$3 WHERE slug=$4 and author=$5",
                article.title,
                article.description,
                article.body,
                slug,
                author,
            )
            .execute(transaction.as_mut())
            .await?
            .rows_affected(),
            slug.to_string(),
        )
    } else {
        // The slug is derived from the title
        let slug = article.title.to_lowercase().replace(' ', "-");
        (sqlx::query!(
            "INSERT INTO Articles(slug, title, description, body, author) VALUES ($1, $2, $3, $4, $5)",
            slug,
            article.title,
            article.description,
            article.body,
            author
        )
        .execute(transaction.as_mut())
        .await?.rows_affected(),
        slug)
    };
    if rows_affected != 1 {
        // We are going to modify just one row, otherwise something funky is going on
        tracing::error!("no rows affected");
        return Err(sqlx::Error::RowNotFound);
    }
    sqlx::query!("DELETE FROM ArticleTags WHERE article=$1", slug)
        .execute(transaction.as_mut())
        .await?;
    if !article.tag_list.is_empty() {
        let mut qb = sqlx::QueryBuilder::new("INSERT INTO ArticleTags(article, tag) ");
        qb.push_values(
            article.tag_list.clone().into_iter().take(BIND_LIMIT / 2),
            |mut b, tag| {
                b.push_bind(slug.clone()).push_bind(tag);
            },
        );
        qb.build().execute(transaction.as_mut()).await?;
    }

    transaction.commit().await?;
    Ok(slug)
}

#[server(EditorAction, "/api")]
#[tracing::instrument]
pub async fn editor_action(
    title: String,
    description: String,
    body: String,
    tag_list: String,
    slug: String,
) -> Result<EditorResponse, ServerFnError> {
    let Some(author) = crate::auth::get_username() else {
        leptos_axum::redirect("/login");
        return Ok(EditorResponse::ValidationError("you should be authenticated".to_string()));
    };
    let article = match validate_article(title, description, body, tag_list) {
        Ok(x) => x,
        Err(x) => return Ok(EditorResponse::ValidationError(x)),
    };
    match update_article(author, slug, article).await {
        Ok(x) => Ok(EditorResponse::Success(x)),
        Err(x) => {
            tracing::error!("EDITOR ERROR: {}", x.to_string());
            Ok(EditorResponse::UpdateError)
        }
    }
}

#[tracing::instrument]
#[component]
pub fn Editor() -> impl IntoView {
    let error = create_rw_signal(view! {<ul></ul>});

    let editor_server_action = create_server_action::<EditorAction>();
    let result_of_call = editor_server_action.value();

    let params = use_params_map();
    let slug = params.get().get("slug").cloned().unwrap_or_default();

    create_effect(move |_| {
        if let Some(msg) = result_of_call.get() {
            match msg {
                Ok(EditorResponse::ValidationError(x)) => error.set(view! {
                    <ul class="error-messages">
                        <li>"Problem while validating: "{x}</li>
                    </ul>
                }),
                Ok(EditorResponse::UpdateError) => error.set(view! {
                    <ul class="error-messages">
                        <li>"Error while updating the article, please, try again later"</li>
                    </ul>
                }),
                Ok(EditorResponse::Success(x)) => {
                    request_animation_frame(move || {
                        use_navigate()(&format!("/article/{x}"), NavigateOptions::default())
                            .unwrap();
                    });
                }
                Err(x) => error.set(view! {
                    <ul class="error-messages">
                        <li>"Unexpected error: "{x.to_string()}</li>
                    </ul>
                }),
            }
        }
        tracing::debug!("Editor Effect!");
    });

    view! {
        <Title text="Editor"/>
        <div class="editor-page">
            <div class="container page">
                <div class="row">
                    {error}
                    <div class="col-md-10 offset-md-1 col-xs-12">
                        <ActionForm action=editor_server_action on:submit=move |ev| {
                            let Ok(data) = EditorAction::from_event(&ev) else {
                                return ev.prevent_default();
                            };
                            if let Err(x) = validate_article(data.title, data.description, data.body, data.tag_list) {
                                error.set(view! {
                                    <ul class="error-messages">
                                        <li>"Problem while validating: "{format!("{x:?}")}</li>
                                    </ul>
                                });
                                ev.prevent_default();
                            }
                        }>
                            <fieldset>
                                <fieldset class="form-group">
                                    <input name="title" type="text" class="form-control form-control-lg"
                                        placeholder="Article Title" />
                                </fieldset>
                                <fieldset class="form-group">
                                    <input name="description" type="text" class="form-control"
                                        placeholder="What's this article about?" />
                                </fieldset>
                                <fieldset class="form-group">
                                    <textarea name="body" class="form-control" rows="8"
                                        placeholder="Write your article (in markdown)"></textarea>
                                </fieldset>
                                <fieldset class="form-group">
                                    <input name="tag_list" type="text" class="form-control"
                                        placeholder="Enter tags(space separated)" />
                                </fieldset>
                                <input name="slug" type="hidden" value=slug />
                                <button class="btn btn-lg pull-xs-right btn-primary" type="submit">
                                    "Publish Article"
                                </button>
                            </fieldset>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
