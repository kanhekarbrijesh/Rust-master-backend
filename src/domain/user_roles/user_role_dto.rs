use serde::Deserialize;

#[derive(Debug, sqlx::FromRow, Deserialize)] // 👈 Add sqlx::FromRow here
pub struct IUserRoleCreateDto {
    pub role_name: String,
    // pub weight: i32,
}

#[derive(Debug, sqlx::FromRow, Deserialize)] // 👈 Add sqlx::FromRow here
pub struct IUserRoleUpdateDto {
    pub role_name: String,
    pub weight: i32,
}
