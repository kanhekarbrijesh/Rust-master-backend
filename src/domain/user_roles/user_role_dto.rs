#[derive(Debug, sqlx::FromRow)] // 👈 Add sqlx::FromRow here
pub struct IUserRoleCreateDto {
    pub role_name: String,
    pub weight: i32,
}
