use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::{models::*, error::AppError};

pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, user: &CreateUserRequest, password_hash: String) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            "
            INSERT INTO users (id, username, email, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "
        )
        .bind(Uuid::new_v4())
        .bind(&user.username)
        .bind(&user.email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?

        Ok(user)
    }

    pub async fn create_link(&self, user_id: Uuid, link: &CreateLinkRequest) -> Result<Link, AppError> {
        let mut tx = self.pool.begin().await?

        let link = sqlx::query_as::<_, Link>(
            "
            INSERT INTO links (id, user_id, url)
            VALUES ($1, $2, $3)
            RETURNING *
            "
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(&link.url)
        .fetch_one(&mut *tx)
        .await?

        if let Some(tags) = &link.tags {
            for tag_name in tags {
                let tag = sqlx::query_as::<_, Tag>(
                    "
                    INSERT INTO tags (id, name)
                    VALUES ($1, $2)
                    ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
                    RETURNING *
                    "
                )
                .bind(Uuid::new_v4())
                .bind(tag_name)
                .fetch_one(&mut *tx)
                .await?

                sqlx::query(
                    "
                    INSERT INTO link_tags (link_id, tag_id)
                    VALUES ($1, $2)
                    "
                )
                .bind(link.id)
                .bind(tag.id)
                .execute(&mut *tx)
                .await?
            }
        }

        tx.commit().await?

        Ok(link)
    }

    pub async fn get_link(&self, id: Uuid) -> Result<LinkResponse, AppError> {
        let link = sqlx::query_as::<_, Link>(
            "SELECT * FROM links WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?

        let tags = sqlx::query_scalar::<_, String>(
            "
            SELECT t.name
            FROM tags t
            JOIN link_tags lt ON lt.tag_id = t.id
            WHERE lt.link_id = $1
            "
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?

        Ok(LinkResponse { link, tags })
    }

    pub async fn get_user_links(&self, user_id: Uuid) -> Result<Vec<LinkResponse>, AppError> {
        let links = sqlx::query_as::<_, Link>(
            "SELECT * FROM links WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?

        let mut responses = Vec::new();
        for link in links {
            let tags = sqlx::query_scalar::<_, String>(
                "
                SELECT t.name
                FROM tags t
                JOIN link_tags lt ON lt.tag_id = t.id
                WHERE lt.link_id = $1
                "
            )
            .bind(link.id)
            .fetch_all(&self.pool)
            .await?

            responses.push(LinkResponse { link, tags });
        }

        Ok(responses)
    }

    pub async fn update_link(&self, id: Uuid, update: &UpdateLinkRequest) -> Result<LinkResponse, AppError> {
        let mut tx = self.pool.begin().await?

        sqlx::query("DELETE FROM link_tags WHERE link_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?

        for tag_name in &update.tags {
            let tag = sqlx::query_as::<_, Tag>(
                "
                INSERT INTO tags (id, name)
                VALUES ($1, $2)
                ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
                RETURNING *
                "
            )
            .bind(Uuid::new_v4())
            .bind(tag_name)
            .fetch_one(&mut *tx)
            .await?

            sqlx::query(
                "
                INSERT INTO link_tags (link_id, tag_id)
                VALUES ($1, $2)
                "
            )
            .bind(id)
            .bind(tag.id)
            .execute(&mut *tx)
            .await?
        }

        let link = sqlx::query_as::<_, Link>(
            "SELECT * FROM links WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?

        tx.commit().await?

        Ok(LinkResponse {
            link,
            tags: update.tags.clone(),
        })
    }

    pub async fn delete_link(&self, id: Uuid) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?

        sqlx::query("DELETE FROM link_tags WHERE link_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?

        sqlx::query("DELETE FROM links WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?

        tx.commit().await?

        Ok(())
    }
}