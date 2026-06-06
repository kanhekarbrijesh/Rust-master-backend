use sqlx::PgPool;

use crate::{
    domain::user_roles::{user_role_dto::IUserRoleCreateDto, user_role_type::IUserRole},
    infrastructure::_postgresql::repositories::{PsqlAppError, PsqlRepoFuture},
};

pub struct UserRolesPsqlRepo {
    pub create: fn(user_role: IUserRoleCreateDto, db: PgPool) -> PsqlRepoFuture<IUserRole>,
    // pub find_one: fn(id: Uuid, db: PgPool) -> PsqlRepoFuture<IUserRole>,
    // pub delete: fn(id: Uuid, db: PgPool) -> PsqlRepoFuture<bool>,
}

impl UserRolesPsqlRepo {
    pub fn new() -> Self {
        Self {
            create: |user_role, db| Box::pin(Self::create_impl(user_role, db)),
            // find_one: |id, db| Box::pin(Self::crate_impl(id, db)),
            // delete: |id, db| Box::pin(Self::crate_impl(id, db)),
        }
    }

    async fn create_impl(
        user_role: IUserRoleCreateDto,
        db: PgPool,
    ) -> Result<IUserRole, PsqlAppError> {
        let role = sqlx::query_as!(
            IUserRole,
            "INSERT INTO user_roles (role_name) VALUES ($1) RETURNING id, role_name, weight", // 👈 Added weight here
            user_role.role_name
        )
        .fetch_one(&db)
        .await?;

        Ok(role)
    }
}
