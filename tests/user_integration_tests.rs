use rust_tut_day1::domain::users::user_type::User;

// ─── DTO CONVERSION TESTS (no DB needed) ────────────────────────────────────

#[test]
fn user_type_to_dto_mapping() {
    let user = User {
        id: 42,
        name: "Integration Tester".to_string(),
        profile_image: "http://storage.local/users/test.png".to_string(),
        role_id: 1,
    };

    let dto: rust_tut_day1::domain::users::user_dto::UserDto = user.into();

    assert_eq!(dto.id, 42);
    assert_eq!(dto.name, "Integration Tester");
    assert_eq!(dto.profile_image, "http://storage.local/users/test.png");
    assert_eq!(dto.role_id, 1);
}

#[test]
fn user_create_dto_rejects_empty_name() {
    use garde::Validate;

    let dto = rust_tut_day1::domain::users::user_dto::UserCreateDto {
        name: "".to_string(),
        profile_image: "http://example.com/pic.jpg".to_string(),
        role_id: 1,
    };

    assert!(dto.validate().is_err());
}

#[test]
fn user_create_dto_rejects_zero_role_id() {
    use garde::Validate;

    let dto = rust_tut_day1::domain::users::user_dto::UserCreateDto {
        name: "Valid Name".to_string(),
        profile_image: "http://example.com/pic.jpg".to_string(),
        role_id: 0,
    };

    assert!(dto.validate().is_err());
}

// ─── INTEGRATION TEST (requires DB, uses existing AppState) ─────────────────

#[tokio::test]
#[ignore]
async fn users_integration_crud_flow() {
    // This integration test is ignored by default.
    // To run: set RUN_INTEGRATION=1 and run with --ignored
    let configs = rust_tut_day1::configuration::config::get_configurations();
    let state = rust_tut_day1::infrastructure::app_state::AppState::new(configs).await;

    // Ensure we can call the service to fetch users (requires a running PostgreSQL)
    let res =
        rust_tut_day1::services::domain_services::user_services::get_all_users(&state.psql_pool)
            .await;
    assert!(res.is_ok());
}
