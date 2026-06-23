use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::user_type::User;

// ─── CREATE DTO ──────────────────────────────────────────────────────────────
/// Used for creating a user.
/// `profile_image` is populated by the controller after file upload.
#[derive(Debug, Serialize, Deserialize, JsonSchema, garde::Validate)]
pub struct UserCreateDto {
    #[garde(length(min = 1, max = 255))]
    pub name: String,

    #[garde(length(min = 1, max = 2048))]
    pub profile_image: String,

    #[garde(range(min = 1))]
    pub role_id: i32,
}

// ─── GET / RESPONSE DTO ──────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserDto {
    pub id: i32,
    pub name: String,
    pub profile_image: String,
    pub role_id: i32,
}

// ─── UPDATE DTO ──────────────────────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, JsonSchema, garde::Validate)]
pub struct UserUpdateDto {
    #[garde(skip)]
    pub id: i32,

    #[garde(length(min = 1, max = 255))]
    pub name: String,

    #[garde(skip)]
    pub profile_image: Option<String>, // None = keep existing

    #[garde(range(min = 1))]
    pub role_id: i32,
}

// ─── CONVERSIONS ─────────────────────────────────────────────────────────────
impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            profile_image: user.profile_image,
            role_id: user.role_id,
        }
    }
}
