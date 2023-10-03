use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum EditorResponse {
    ValidationError(String),
    UpdateError,
    Successful(String),
}

#[cfg_attr(feature = "hydrate", allow(dead_code))]
#[derive(Debug)]
struct ArticleUpdate {
    title: String,
    description: String,
    body: String,
    tag_list: std::collections::HashSet<String>,
}

const TITLE_MIN_LENGTH: usize = 4;
const DESCRIPTION_MIN_LENGTH: usize = 4;
const BODY_MIN_LENGTH: usize = 10;

#[cfg(feature = "ssr")]
#[tracing::instrument]
fn validate_article(
    title: String,
    description: String,
    body: String,
    tag_list: String,
) -> Result<ArticleUpdate, String> {
    if title.len() < TITLE_MIN_LENGTH {
        return Err("You need to provide a title with at least 4 characters".into());
    }

    if description.len() < DESCRIPTION_MIN_LENGTH {
        return Err("You need to provide a description with at least 4 characters".into());
    }

    if body.len() < BODY_MIN_LENGTH {
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
        let slug = article
            .title
            .chars()
            .map(|c| {
                let c = c.to_ascii_lowercase();
                if c == ' ' {
                    '-'
                } else {
                    c
                }
            })
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect::<String>();
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
        return Ok(EditorResponse::ValidationError(
            "you should be authenticated".to_string(),
        ));
    };
    let article = match validate_article(title, description, body, tag_list) {
        Ok(x) => x,
        Err(x) => return Ok(EditorResponse::ValidationError(x)),
    };
    match update_article(author, slug, article).await {
        Ok(x) => {
            leptos_axum::redirect(&format!("/article/{x}"));
            Ok(EditorResponse::Successful(x))
        }
        Err(x) => {
            tracing::error!("EDITOR ERROR: {}", x.to_string());
            Ok(EditorResponse::UpdateError)
        }
    }
}

#[tracing::instrument]
#[component]
pub fn Editor() -> impl IntoView {
    let editor_server_action = create_server_action::<EditorAction>();
    let result = editor_server_action.value();
    let error = move || {
        result.with(|x| {
            x.as_ref().map_or(true, |y| {
                y.is_err() || !matches!(y, Ok(EditorResponse::Successful(_)))
            })
        })
    };

    let params = use_params_map();
    let article_res = create_resource(
        move || params.get(),
        |slug| async move {
            if let Some(s) = slug.get("slug") {
                super::get_article(s.to_string()).await
            } else {
                Ok(super::ArticleResult::default())
            }
        },
    );

    view! {
        <Title text="Editor"/>
        <div class="editor-page">
            <div class="container page">
                <div class="row">
                    <p class="text-xs-center"
                        class:text-success=move || !error()
                        class:error-messages=error
                    >
                        <strong>
                            {move || result.with(|x| {
                                let Some(x) = x else {
                                    return String::new();
                                };
                                match x {
                                    Ok(EditorResponse::ValidationError(x)) => {
                                        format!("Problem while validating: {x}")
                                    }
                                    Ok(EditorResponse::UpdateError) => {
                                        "Error while updating the article, please, try again later".into()
                                    }
                                    Ok(EditorResponse::Successful(_)) => {
                                        String::new()
                                    }
                                    Err(x) => format!("Unexpected error: {x}"),
                                }
                            })}
                        </strong>
                    </p>

                    <div class="col-md-10 offset-md-1 col-xs-12">
                        <ActionForm action=editor_server_action>
                        <Suspense fallback=move || view! {<p>"Loading Tags"</p> }>
                            <ErrorBoundary fallback=|_| {
                                view! { <p class="error-messages text-xs-center">"Something went wrong."</p>}
                            }>
                                {move || article_res.get().map(move |x| x.map(move |a| {
                                    view! {
                                        <fieldset>
                                            <fieldset class="form-group">
                                                <input name="title" type="text" class="form-control form-control-lg" minlength=TITLE_MIN_LENGTH
                                                    placeholder="Article Title" value=a.article.title />
                                            </fieldset>
                                            <fieldset class="form-group">
                                                <input name="description" type="text" class="form-control" minlength=DESCRIPTION_MIN_LENGTH
                                                    placeholder="What's this article about?" value=a.article.description />
                                            </fieldset>
                                            <fieldset class="form-group">
                                                <textarea name="body" class="form-control" rows="8"
                                                    placeholder="Write your article (in markdown)" minlength=BODY_MIN_LENGTH
                                                    prop:value=a.article.body.unwrap_or_default()></textarea>
                                            </fieldset>
                                            <fieldset class="form-group">
                                                <input name="tag_list" type="text" class="form-control"
                                                    placeholder="Enter tags(space separated)" value=a.article.tag_list.join(" ") />
                                            </fieldset>
                                            <input name="slug" type="hidden" value=a.article.slug />
                                            <button class="btn btn-lg pull-xs-right btn-primary" type="submit">
                                                "Publish Article"
                                            </button>
                                        </fieldset>
                                    }
                                }))}
                            </ErrorBoundary>
                        </Suspense>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
