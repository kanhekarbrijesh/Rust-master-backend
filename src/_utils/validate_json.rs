use axum::{
    Json,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use garde::Validate;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    // 1. ✅ EXACT FIX: Tell the compiler the validation context implements Default
    <T as Validate>::Context: Default,
    S: Send + Sync,
{
    type Rejection = Response;

    // 2. ✅ EXACT FIX: Removed #[async_trait] completely. It's native now!
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Step A: Parse the JSON payload safely using Axum's built-in Json extractor logic
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "Invalid JSON format",
                        "details": rejection.body_text()
                    })),
                )
                    .into_response()
            })?;

        // Step B: Run Garde validation smoothly
        if let Err(report) = value.validate() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Validation failed",
                    "details": report.to_string()
                })),
            )
                .into_response());
        }

        Ok(ValidatedJson(value))
    }
}
