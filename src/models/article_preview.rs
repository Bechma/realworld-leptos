use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ArticlePreview {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub created_at: String,
    pub favorites_count: Option<i64>,
    pub author: User,
    pub fav: bool,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct User {
    pub username: String,
    pub image: Option<String>,
    pub following: bool,
}

impl ArticlePreview {
    #[cfg(feature = "ssr")]
    pub async fn for_home_page(
        cx: leptos::Scope,
        page: i64,
        amount: i64,
        tag: String,
        my_feed: bool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let username = crate::auth::get_username(cx).unwrap_or_default();
        sqlx::query!(
            "
SELECT 
    a.slug,
    a.title,
    a.description,
    a.created_at,
    (SELECT COUNT(*) FROM FavArticles WHERE article=a.slug) as favorites_count,
    u.username, u.image,
    EXISTS(SELECT 1 FROM FavArticles WHERE article=a.slug and username=$5) as fav,
    EXISTS(SELECT 1 FROM Follows WHERE follower=$5 and influencer=u.username) as following,
    (SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as tag_list
FROM Articles as a
    JOIN Users as u ON a.author = u.username
WHERE
    CASE WHEN $3!='' THEN a.slug in (SELECT distinct article FROM ArticleTags WHERE tag=$3)
    ELSE 1=1
    END
    AND
    CASE WHEN $4 THEN u.username in (SELECT influencer FROM Follows WHERE follower=$5)
    ELSE 1=1
    END
ORDER BY a.created_at desc
LIMIT $1 OFFSET $2",
            amount,
            page * amount,
            tag,
            my_feed,
            username,
        )
        .map(|x| Self {
            slug: x.slug,
            title: x.title,
            fav: x.fav.unwrap_or_default(),
            description: x.description,
            created_at: x.created_at.format("%d/%m/%Y %H:%M").to_string(),
            favorites_count: x.favorites_count,
            author: User {
                username: x.username,
                image: x.image,
                following: x.following.unwrap_or_default(),
            },
            tags: x
                .tag_list
                .unwrap_or_default()
                .split(' ')
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
        })
        .fetch_all(crate::database::get_db())
        .await
    }
}
