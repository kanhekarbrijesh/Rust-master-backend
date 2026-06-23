// src/services/domain_services/user_role_services.rs
use crate::{
    domain::user_roles::{
        user_role_dto::{IUserRoleCreateDto, IUserRoleUpdateDto},
        user_role_type::IUserRole,
    },
    infrastructure::db::postgresql::psql_app_error::PsqlAppError,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct UserRolesPsqlService;

impl UserRolesPsqlService {
    pub async fn create(
        db: &PgPool,
        payload: IUserRoleCreateDto,
    ) -> Result<IUserRole, PsqlAppError> {
        let role = sqlx::query_as!(
            IUserRole,
            r#"
            INSERT INTO user_roles (role_name) 
            VALUES ($1) 
            RETURNING id, role_name, weight
            "#,
            payload.role_name
        )
        .fetch_one(db)
        .await?;

        Ok(role)
    }

    pub async fn find_by_id(db: &PgPool, id: i32) -> Result<Option<IUserRole>, PsqlAppError> {
        let role = sqlx::query_as!(
            IUserRole,
            r#"
            SELECT id, role_name, weight 
            FROM user_roles 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(db)
        .await?;

        Ok(role)
    }

    // pub async fn find_all(db: &PgPool) -> Result<Vec<IUserRole>, PsqlAppError> {
    //     let roles = sqlx::query_as!(
    //         IUserRole,
    //         r#"
    //         SELECT id, role_name, weight
    //         FROM user_roles
    //         ORDER BY weight DESC, role_name ASC
    //         "#
    //     )
    //     .fetch_all(db)
    //     .await?;

    //     Ok(roles)
    // }

    pub async fn update(
        db: &PgPool,
        id: i32,
        payload: IUserRoleUpdateDto,
    ) -> Result<Option<IUserRole>, PsqlAppError> {
        let role = sqlx::query_as!(
            IUserRole,
            r#"
            UPDATE user_roles 
            SET role_name = $1, weight = $2 
            WHERE id = $3 
            RETURNING id, role_name, weight
            "#,
            payload.role_name,
            payload.weight,
            id
        )
        .fetch_optional(db)
        .await?;

        Ok(role)
    }

    // pub async fn delete(db: &PgPool, id: i32) -> Result<bool, PsqlAppError> {
    //     let result = sqlx::query!(
    //         r#"
    //         DELETE FROM user_roles
    //         WHERE id = $1
    //         "#,
    //         id
    //     )
    //     .execute(db)
    //     .await?;

    //     Ok(result.rows_affected() > 0)
    // }
}
