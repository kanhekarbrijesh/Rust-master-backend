use sqlx::PgPool;

use crate::{_utils::app_error::AppError, domain::users::user_type::User};

/// Pure repository functions for the `users` table.
/// Uses raw SQL via `sqlx::query_as!` for compile-time query checking.
pub struct UserRepo;

impl UserRepo {
    pub async fn create(
        db: &PgPool,
        name: &str,
        profile_image: &str,
        role_id: i32,
    ) -> Result<User, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (name, profile_image, role_id)
            VALUES ($1, $2, $3)
            RETURNING id, name, profile_image, role_id
            "#,
            name,
            profile_image,
            role_id,
        )
        .fetch_one(db)
        .await?;

        Ok(user)
    }

    pub async fn find_all(db: &PgPool) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, name, profile_image, role_id
            FROM users
            ORDER BY id ASC
            "#,
        )
        .fetch_all(db)
        .await?;

        Ok(users)
    }

    pub async fn find_by_id(db: &PgPool, id: i32) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, name, profile_image, role_id
            FROM users
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(db)
        .await?;

        Ok(user)
    }

    pub async fn update(
        db: &PgPool,
        id: i32,
        name: &str,
        profile_image: &str,
        role_id: i32,
    ) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET name = $1, profile_image = $2, role_id = $3
            WHERE id = $4
            RETURNING id, name, profile_image, role_id
            "#,
            name,
            profile_image,
            role_id,
            id,
        )
        .fetch_optional(db)
        .await?;

        Ok(user)
    }

    pub async fn delete(db: &PgPool, id: i32) -> Result<bool, AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
            id,
        )
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
