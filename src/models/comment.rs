#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Comment {
    pub id: i32,
    pub article: String,
    pub username: String,
    pub body: String,
    pub created_at: String,
    pub user_image: Option<String>,
}

impl Comment {
    #[cfg(feature = "ssr")]
    pub async fn insert(
        article: String,
        username: String,
        body: String,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO Comments(article, username, body) VALUES ($1, $2, $3)",
            article,
            username,
            body
        )
        .execute(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn get_all(article: String) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query!(
            "
        SELECT c.*, u.image FROM Comments as c
            JOIN Users as u ON u.username=c.username
        WHERE c.article=$1
        ORDER BY c.created_at",
            article
        )
        .map(|x| Self {
            id: x.id,
            article: x.article,
            username: x.username,
            body: x.body,
            created_at: x.created_at.format(super::DATE_FORMAT).to_string(),
            user_image: x.image,
        })
        .fetch_all(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn delete(
        id: i32,
        user: String,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!("DELETE FROM Comments WHERE id=$1 and username=$2", id, user)
            .execute(crate::database::get_db())
            .await
    }
}
