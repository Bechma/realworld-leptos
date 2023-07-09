use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct User {
    username: String,
    #[serde(skip_deserializing)]
    password: String,
    email: String,
    bio: Option<String>,
    image: Option<String>,
}

#[cfg(feature = "ssr")]
static EMAIL_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

impl User {
    #[inline]
    pub fn username(&self) -> String {
        self.username.to_string()
    }
    #[inline]
    pub fn email(&self) -> String {
        self.email.to_string()
    }
    #[inline]
    pub fn bio(&self) -> Option<String> {
        self.bio.clone()
    }
    #[inline]
    pub fn image(&self) -> Option<String> {
        self.image.clone()
    }

    pub fn set_password(mut self, password: String) -> Result<Self, String> {
        if password.len() < 4 {
            return Err("You need to provide a stronger password".into());
        }
        self.password = password;
        Ok(self)
    }

    pub fn set_username(mut self, username: String) -> Result<Self, String> {
        if username.len() < 4 {
            return Err(format!(
                "Username {username} is too short, at least 4 characters"
            ));
        }
        self.username = username;
        Ok(self)
    }

    #[cfg(feature = "ssr")]
    fn validate_email(email: &str) -> bool {
        EMAIL_REGEX
            .get_or_init(|| regex::Regex::new(r"^[\w\-\.]+@([\w-]+\.)+\w{2,4}$").unwrap())
            .is_match(email)
    }

    #[cfg(not(feature = "ssr"))]
    fn validate_email(email: &str) -> bool {
        crate::emailRegex(email)
    }

    pub fn set_email(mut self, email: String) -> Result<Self, String> {
        if !Self::validate_email(&email) {
            return Err(format!(
                "The email {email} is invalid, provide a correct one"
            ));
        }
        self.email = email;
        Ok(self)
    }

    #[inline]
    pub fn set_bio(mut self, bio: String) -> Self {
        self.bio = Some(bio);
        self
    }

    #[inline]
    pub fn set_image(mut self, image: String) -> Self {
        self.image = Some(image);
        self
    }

    #[cfg(feature = "ssr")]
    pub async fn get(username: String) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(Self, "SELECT * FROM users WHERE username=$1", username)
            .fetch_one(crate::database::get_db())
            .await
    }

    #[cfg(feature = "ssr")]
    pub async fn insert(&self) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO Users(username, email, password) VALUES ($1, $2, crypt($3, gen_salt('bf')))",
            self.username,
            self.email,
            self.password,
        )
        .execute(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn update(&self) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "
UPDATE Users SET
    image=$2,
    bio=$3,
    email=$4,
    password=CASE WHEN $5 IS TRUE THEN crypt($6, gen_salt('bf')) ELSE password END
WHERE username=$1",
            self.username,
            self.image,
            self.bio,
            self.email,
            !self.password.is_empty(),
            self.password
        )
        .execute(crate::database::get_db())
        .await
    }
}
