use std::sync::Arc;

use sqlx::PgPool;
use tracing::info;

use crate::{
    _utils::app_error::AppError,
    domain::users::{
        user_dto::{UserCreateDto, UserDto, UserUpdateDto},
        user_repo::UserRepo,
    },
    infrastructure::storage::{storage_types::StorageProvider, storage_util},
};

/// Pure business-logic service for User domain.
/// File handling is NOT done here — the controller handles uploads and passes
/// a ready-to-persist DTO (with `profile_image` already populated).
pub async fn create_user(db: &PgPool, payload: UserCreateDto) -> Result<UserDto, AppError> {
    let user = UserRepo::create(db, &payload.name, &payload.profile_image, payload.role_id).await?;
    info!(user_id = user.id, "User created successfully");
    Ok(user.into())
}

pub async fn get_all_users(db: &PgPool) -> Result<Vec<UserDto>, AppError> {
    let users = UserRepo::find_all(db).await?;
    info!(count = users.len(), "Fetched all users");
    Ok(users.into_iter().map(Into::into).collect())
}

pub async fn get_user_by_id(db: &PgPool, id: i32) -> Result<UserDto, AppError> {
    let user = UserRepo::find_by_id(db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    info!(user.id, "Fetched user by id");
    Ok(user.into())
}

pub async fn update_user(
    db: &PgPool,
    id: i32,
    payload: UserUpdateDto,
) -> Result<UserDto, AppError> {
    let mut tx = db
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to start transaction: {e}")))?;

    // If no new profile_image was provided, keep the existing one (inside tx)
    let profile_image = match &payload.profile_image {
        Some(img) => img.clone(),
        None => {
            let row = sqlx::query!(r#"SELECT profile_image FROM users WHERE id = $1"#, id,)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
            row.profile_image
        }
    };

    let user = sqlx::query_as!(
        crate::domain::users::user_type::User,
        r#"
        UPDATE users
        SET name = $1, profile_image = $2, role_id = $3
        WHERE id = $4
        RETURNING id, name, profile_image, role_id
        "#,
        payload.name,
        profile_image,
        payload.role_id,
        id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to commit transaction: {e}")))?;

    info!(user.id, "User updated successfully (atomic transaction)");
    Ok(user.into())
}

pub async fn delete_user(
    db: &PgPool,
    storage: Arc<dyn StorageProvider>,
    serve_prefix: &str,
    id: i32,
) -> Result<(), AppError> {
    // ── Fetch user first to get the profile_image URL for file cleanup ────
    let user = UserRepo::find_by_id(db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // ── Delete the file from storage (best-effort) ────────────────────────
    let file_key = storage_util::storage_key_from_url(&user.profile_image, serve_prefix);
    let _ = storage_util::delete_file_quietly(&*storage, &file_key).await;

    // ── Delete from DB ────────────────────────────────────────────────────
    UserRepo::delete(db, id).await?;

    info!(id, "User deleted successfully (file cleaned up)");
    Ok(())
}

// ---------------------- Unit tests ----------------------
#[cfg(test)]
mod tests {
    use crate::domain::users::user_dto::{UserCreateDto, UserUpdateDto};
    use crate::domain::users::user_type::User;

    #[test]
    fn user_dto_conversion() {
        let user = User {
            id: 1,
            name: "Alice".to_string(),
            profile_image: "http://example.com/img.jpg".to_string(),
            role_id: 2,
        };

        let dto: crate::domain::users::user_dto::UserDto = user.into();
        assert_eq!(dto.id, 1);
        assert_eq!(dto.name, "Alice");
        assert_eq!(dto.profile_image, "http://example.com/img.jpg");
        assert_eq!(dto.role_id, 2);
    }

    #[test]
    fn user_create_dto_validation_name_too_short() {
        use garde::Validate;
        let dto = UserCreateDto {
            name: "".to_string(),
            profile_image: "http://example.com/img.jpg".to_string(),
            role_id: 1,
        };
        assert!(dto.validate().is_err());
    }

    #[test]
    fn user_create_dto_validation_valid() {
        use garde::Validate;
        let dto = UserCreateDto {
            name: "Bob".to_string(),
            profile_image: "http://example.com/img.jpg".to_string(),
            role_id: 1,
        };
        assert!(dto.validate().is_ok());
    }

    #[test]
    fn user_update_dto_profile_image_optional() {
        let dto = UserUpdateDto {
            id: 1,
            name: "Bob Updated".to_string(),
            profile_image: None,
            role_id: 2,
        };
        assert!(dto.profile_image.is_none());
    }

    #[test]
    fn user_dto_rejects_name_exceeding_max_length() {
        use garde::Validate;
        let dto = UserCreateDto {
            name: "A".repeat(256),
            profile_image: "http://example.com/img.jpg".to_string(),
            role_id: 1,
        };
        assert!(dto.validate().is_err());
    }

    #[test]
    fn user_dto_rejects_profile_image_url_too_long() {
        use garde::Validate;
        let dto = UserCreateDto {
            name: "Valid Name".to_string(),
            profile_image: format!("http://example.com/{}", "a".repeat(2048)),
            role_id: 1,
        };
        assert!(dto.validate().is_err());
    }

    #[test]
    fn user_dto_rejects_zero_role_id() {
        use garde::Validate;
        let dto = UserCreateDto {
            name: "Charlie".to_string(),
            profile_image: "http://example.com/img.jpg".to_string(),
            role_id: 0,
        };
        assert!(dto.validate().is_err());
    }

    #[test]
    fn user_dto_rejects_negative_role_id() {
        use garde::Validate;
        let dto = UserCreateDto {
            name: "Charlie".to_string(),
            profile_image: "http://example.com/img.jpg".to_string(),
            role_id: -1,
        };
        assert!(dto.validate().is_err());
    }

    #[test]
    fn user_update_dto_valid_with_all_fields() {
        use garde::Validate;
        let dto = UserUpdateDto {
            id: 1,
            name: "Updated Name".to_string(),
            profile_image: Some("http://example.com/new.jpg".to_string()),
            role_id: 3,
        };
        assert!(dto.validate().is_ok());
    }

    #[test]
    fn user_update_dto_rejects_empty_name() {
        use garde::Validate;
        let dto = UserUpdateDto {
            id: 1,
            name: "".to_string(),
            profile_image: None,
            role_id: 1,
        };
        assert!(dto.validate().is_err());
    }
}
