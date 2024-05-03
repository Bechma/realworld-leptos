use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserPreview {
    pub username: String,
    pub image: Option<String>,
    pub following: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct User {
    username: String,
    #[cfg_attr(feature = "hydrate", allow(dead_code))]
    #[serde(skip_serializing)]
    password: Option<String>,
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
        self.password = Some(password);
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

    pub fn set_bio(mut self, bio: String) -> Result<Self, String> {
        static BIO_MIN: usize = 10;
        if bio.is_empty() {
            self.bio = None;
        } else if bio.len() < BIO_MIN {
            return Err("bio too short, at least 10 characters".into());
        } else {
            self.bio = Some(bio);
        }
        Ok(self)
    }

    #[inline]
    pub fn set_image(mut self, image: String) -> Result<Self, String> {
        if image.is_empty() {
            self.image = None;
            // TODO: This is incorrect! changeme in the future for a proper validation
        } else if !image.starts_with("http") {
            return Err("Invalid image!".into());
        } else {
            self.image = Some(image);
        }
        Ok(self)
    }

    #[cfg(feature = "ssr")]
    pub async fn get(username: String) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "SELECT username, email, bio, image, NULL as password FROM users WHERE username=$1",
            username
        )
            .fetch_one(crate::database::get_db())
            .await
    }

    #[cfg(feature = "ssr")]
    pub async fn get_email(email: String) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "SELECT username, email, bio, image, NULL as password FROM users WHERE email=$1",
            email
        )
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
    password=CASE WHEN $5 THEN crypt($6, gen_salt('bf')) ELSE password END
WHERE username=$1",
            self.username,
            self.image,
            self.bio,
            self.email,
            self.password.is_some(),
            self.password,
        )
            .execute(crate::database::get_db())
            .await
    }
}
