use sqlx::PgPool;
// Removed Uuid since database expects i32

use crate::{
    domain::user_roles::{
        user_role_dto::{IUserRoleCreateDto, IUserRoleUpdateDto},
        user_role_type::IUserRole,
    },
    infrastructure::_postgresql::repositories::{PsqlAppError, PsqlRepoFuture},
};

#[derive(Clone)]
pub struct UserRolesPsqlRepo {
    pub create: fn(user_role: IUserRoleCreateDto, db: PgPool) -> PsqlRepoFuture<IUserRole>,
    pub find_one: fn(id: i32, db: PgPool) -> PsqlRepoFuture<Option<IUserRole>>, // 👈 Changed to i32
    pub find_all: fn(db: PgPool) -> PsqlRepoFuture<Vec<IUserRole>>,
    pub update:
        fn(id: i32, user_role: IUserRoleUpdateDto, db: PgPool) -> PsqlRepoFuture<Option<IUserRole>>, // 👈 Changed to i32
    pub delete: fn(id: i32, db: PgPool) -> PsqlRepoFuture<bool>, // 👈 Changed to i32
}

impl UserRolesPsqlRepo {
    pub fn new() -> Self {
        Self {
            create: |user_role, db| Box::pin(Self::create_impl(user_role, db)),
            find_one: |id, db| Box::pin(Self::find_one_impl(id, db)),
            find_all: |db| Box::pin(Self::find_all_impl(db)),
            update: |id, user_role, db| Box::pin(Self::update_impl(id, user_role, db)),
            delete: |id, db| Box::pin(Self::delete_impl(id, db)),
        }
    }

    async fn create_impl(
        user_role: IUserRoleCreateDto,
        db: PgPool,
    ) -> Result<IUserRole, PsqlAppError> {
        let role = sqlx::query_as!(
            IUserRole,
            r#"
            INSERT INTO user_roles (role_name) 
            VALUES ($1) 
            RETURNING id, role_name, weight
            "#,
            user_role.role_name
        )
        .fetch_one(&db)
        .await?;

        Ok(role)
    }

    async fn find_one_impl(
        id: i32, // 👈 Changed to i32
        db: PgPool,
    ) -> Result<Option<IUserRole>, PsqlAppError> {
        let role = sqlx::query_as!(
            IUserRole,
            r#"
            SELECT id, role_name, weight 
            FROM user_roles 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&db)
        .await?;

        Ok(role)
    }

    async fn find_all_impl(db: PgPool) -> Result<Vec<IUserRole>, PsqlAppError> {
        let roles = sqlx::query_as!(
            IUserRole,
            r#"
            SELECT id, role_name, weight 
            FROM user_roles
            ORDER BY weight DESC, role_name ASC
            "#
        )
        .fetch_all(&db)
        .await?;

        Ok(roles)
    }

    async fn update_impl(
        id: i32, // 👈 Changed to i32
        user_role: IUserRoleUpdateDto,
        db: PgPool,
    ) -> Result<Option<IUserRole>, PsqlAppError> {
        let role = sqlx::query_as!(
            IUserRole,
            r#"
            UPDATE user_roles 
            SET role_name = $1, weight = $2 
            WHERE id = $3 
            RETURNING id, role_name, weight
            "#,
            user_role.role_name,
            user_role.weight,
            id
        )
        .fetch_optional(&db)
        .await?;

        Ok(role)
    }

    async fn delete_impl(
        id: i32, // 👈 Changed to i32
        db: PgPool,
    ) -> Result<bool, PsqlAppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM user_roles 
            WHERE id = $1
            "#,
            id
        )
        .execute(&db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
