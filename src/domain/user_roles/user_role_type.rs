#[derive(Debug, sqlx::FromRow)] // 👈 Add sqlx::FromRow here
pub struct IUserRole {
    pub id: i32,
    pub role_name: String,
    pub weight: i32,
}
