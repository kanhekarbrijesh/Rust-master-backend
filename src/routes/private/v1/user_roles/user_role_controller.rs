// src/routes/private/v1/user_roles/user_role_controller.rs

use crate::{
    domain::user_roles::{
        user_role_dto::{IUserRoleCreateDto, IUserRoleUpdateDto},
        user_role_type::IUserRole,
    },
    infrastructure::app_state::AppState,
    services::domain_services::user_roles_psql_services::UserRolesPsqlService,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

// POST /roles
#[axum::debug_handler]
pub async fn create_role_handler(
    State(state): State<AppState>,
    Json(payload): Json<IUserRoleCreateDto>,
) -> Result<Json<IUserRole>, StatusCode> {
    // 👈 Pass a reference to the pool. No more .clone()!
    let role = UserRolesPsqlService::create(&state.psql_pool, payload)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(role))
}

// GET /roles/:id
#[axum::debug_handler]
pub async fn get_role_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<IUserRole>, StatusCode> {
    let role = UserRolesPsqlService::find_by_id(&state.psql_pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(role))
}

// PUT /roles/:id
#[axum::debug_handler]
pub async fn update_role_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<IUserRoleUpdateDto>,
) -> Result<Json<IUserRole>, StatusCode> {
    let updated_role = UserRolesPsqlService::update(&state.psql_pool, id, payload)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(updated_role))
}
