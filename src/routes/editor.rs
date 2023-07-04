use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum EditorResponse {
    ValidationError(String),
    UpdateError,
    Unauthenticated,
    Success(String),
}

#[cfg(feature = "ssr")]
#[derive(Debug)]
struct ArticleUpdate {
    title: String,
    description: String,
    body: String,
    tag_list: std::collections::HashSet<String>,
}

#[cfg(feature = "ssr")]
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
            .execute(&mut transaction)
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
        .execute(&mut transaction)
        .await?.rows_affected(),
        slug)
    };
    if rows_affected != 1 {
        // We are going to modify just one row, otherwise something funky is going on
        tracing::error!("no rows affected");
        return Err(sqlx::Error::RowNotFound);
    }
    sqlx::query!("DELETE FROM ArticleTags WHERE article=$1", slug)
        .execute(&mut transaction)
        .await?;
    if !article.tag_list.is_empty() {
        let mut qb = sqlx::QueryBuilder::new("INSERT INTO ArticleTags(article, tag) ");
        qb.push_values(
            article.tag_list.clone().into_iter().take(BIND_LIMIT / 2),
            |mut b, tag| {
                b.push_bind(slug.clone()).push_bind(tag);
            },
        );
        qb.build().execute(&mut transaction).await?;
    }

    transaction.commit().await?;
    Ok(slug)
}

#[tracing::instrument]
#[server(EditorAction, "/api")]
pub async fn editor_action(
    cx: Scope,
    title: String,
    description: String,
    body: String,
    tag_list: String,
    slug: String,
) -> Result<EditorResponse, ServerFnError> {
    let Some(author) = super::get_username(cx) else {
        return Ok(EditorResponse::Unauthenticated);
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
pub fn Editor(cx: Scope) -> impl IntoView {
    let (error, set_error) = create_signal(cx, view! {cx, <ul></ul>});

    let editor_server_action = create_server_action::<EditorAction>(cx);
    let result_of_call = editor_server_action.value();

    let params = use_params_map(cx);
    let slug = params
        .get()
        .get("slug")
        .map(|x| {
            view! {cx,
                <input name="slug" type="hidden" value=x />
            }
        })
        .unwrap_or(view! {cx, <input name="slug" type="hidden" value="" />});

    create_effect(cx, move |_| {
        let r = result_of_call.get();
        let navigate = use_navigate(cx);
        request_animation_frame(move || {
            if super::get_username(cx).is_none() {
                navigate("/login", NavigateOptions::default()).unwrap();
                tracing::debug!("You need to login");
                return;
            }
            if let Some(msg) = r {
                match msg {
                    Ok(EditorResponse::ValidationError(x)) => set_error.set(view! {cx,
                        <ul class="error-messages">
                            <li>"Problem while validating: "{x}</li>
                        </ul>
                    }),
                    Ok(EditorResponse::UpdateError) => set_error.set(view! {cx,
                        <ul class="error-messages">
                            <li>"Error while updating the article, please, try again later"</li>
                        </ul>
                    }),
                    Ok(EditorResponse::Unauthenticated) => {
                        tracing::debug!("You need to login");
                        navigate("/login", NavigateOptions::default()).unwrap()
                    }
                    Ok(EditorResponse::Success(x)) => {
                        navigate(&format!("/article/{}", x), NavigateOptions::default()).unwrap()
                    }
                    Err(x) => set_error.set(view! {cx,
                        <ul class="error-messages">
                            <li>"Unexpected error: "{x.to_string()}</li>
                        </ul>
                    }),
                }
            }
        });
        tracing::debug!("Editor Effect!");
    });

    view! { cx,
        <Title text="Editor"/>
        <div class="editor-page">
            <div class="container page">
                <div class="row">
                    {error}
                    <div class="col-md-10 offset-md-1 col-xs-12">
                        <ActionForm action=editor_server_action>
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
                                <button class="btn btn-lg pull-xs-right btn-primary" type="submit">
                                    "Publish Article"
                                </button>
                                {slug}
                            </fieldset>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
