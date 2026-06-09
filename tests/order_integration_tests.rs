use rust_tut_day1::configuration::config::get_configurations;
use rust_tut_day1::infrastructure::app_state::AppState;

#[tokio::test]
#[ignore]
async fn orders_integration_crud_flow() {
    // This integration test is ignored by default. To run set RUN_INTEGRATION=1 and run with --ignored
    let configs = get_configurations();
    let state = AppState::new(configs).await;

    // Ensure we can call the service to fetch orders (requires a running MongoDB pointed by config)
    let res =
        rust_tut_day1::services::domain_services::order_services::get_all_orders(&state).await;
    assert!(res.is_ok());
}
