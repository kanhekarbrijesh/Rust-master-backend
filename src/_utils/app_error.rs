use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Conflict(String),
    BadRequest(String),
    DatabaseError(mongodb::error::Error),
    SqlxError(sqlx::Error),
    Internal(String),
}

// =========================================================================
// 1. HTTP RESPONSE TRANSFORMATION (Axum Integration)
// =========================================================================
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "Not Found", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "Conflict", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "Bad Request", msg),

            AppError::DatabaseError(err) => {
                error!("MongoDB System Failure: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database Failure",
                    "An unexpected error occurred on the MongoDB storage engine level.".to_string(),
                )
            }

            AppError::SqlxError(err) => {
                error!("PostgreSQL System Failure: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database Failure",
                    "An unexpected error occurred on the PostgreSQL database level.".to_string(),
                )
            }

            AppError::Internal(msg) => {
                error!("Internal Core System Error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                    msg,
                )
            }
        };

        // Standardized JSON payload structure across your entire API ecosystem
        (
            status,
            Json(serde_json::json!({
                "error": error_message,
                "details": details
            })),
        )
            .into_response()
    }
}

// =========================================================================
// 2. MONGODB ERROR PARSING & CONVERSION
// =========================================================================
impl From<mongodb::error::Error> for AppError {
    fn from(err: mongodb::error::Error) -> Self {
        // Extract database code across standard Command responses and inline Write failures
        let code = match *err.kind {
            mongodb::error::ErrorKind::Command(ref reply) => Some(reply.code),
            mongodb::error::ErrorKind::Write(mongodb::error::WriteFailure::WriteError(ref e)) => Some(e.code),
            _ => None,
        };

        if let Some(c) = code {
            match c {
                // Code 11000 = Unique Constraint Failure (Duplicate Key)
                11000 => return AppError::Conflict(
                    "The value provided breaks a unique field constraint in the document collection.".to_string()
                ),
                // Code 121 = Schema Document Validation Failure
                121 => return AppError::BadRequest(
                    "Schema validation failed on the MongoDB engine level. Check data formats.".to_string()
                ),
                _ => {}
            }
        }

        AppError::DatabaseError(err)
    }
}

// =========================================================================
// 3. POSTGRESQL (SQLX) ERROR PARSING & CONVERSION
// =========================================================================
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            // Explicitly map a missing query result straight to a 404 NotFound
            sqlx::Error::RowNotFound => {
                AppError::NotFound("The requested relational database record could not be found.".to_string())
            }
            
            // Inspect underlying driver/database engine errors
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        // Class 23 — Integrity Constraint Violation
                        "23505" => AppError::Conflict(
                            "Duplicate key entry violation. A record with these unique details already exists.".to_string()
                        ),
                        "23503" => AppError::Conflict(
                            "Foreign key constraint violation. Referenced parent or child record does not exist.".to_string()
                        ),
                        "23502" => AppError::BadRequest(
                            "Not null constraint violation. A required database field was missing or null.".to_string()
                        ),
                        "23514" => AppError::BadRequest(
                            "Database check constraint violation. Input data values fail domain business logic restrictions.".to_string()
                        ),
                        
                        // Class 22 — Data Exception
                        "22001" => AppError::BadRequest(
                            "String data right truncation. Provided input value exceeds maximum column character length.".to_string()
                        ),
                        "22P02" => AppError::BadRequest(
                            "Invalid text representation format. Invalid type conversion format parsed on data layer.".to_string()
                        ),
                        
                        _ => AppError::Internal(format!("Unmapped Postgres Error (Code: {}): {}", code, db_err.message())),
                    }
                } else {
                    AppError::Internal(format!("Database driver exception: {}", db_err.message()))
                }
            }
            
            // Catch pooling/connection dropouts cleanly as system internal issues
            sqlx::Error::PoolTimedOut => AppError::Internal("Database connection pool timeout. Server overloaded.".to_string()),
            sqlx::Error::Io(io_err) => AppError::Internal(format!("Database network IO transmission breakdown: {}", io_err)),
            
            // Fallthrough for configuration or structural compilation/type-decoding failures
            _ => AppError::SqlxError(err),
        }
    }
}

// =========================================================================
// 4. UTILITY STRING CONVERSIONS (Enables rapid inline error bubbling via `?`)
// =========================================================================

// Automatically converts runtime `String` allocations into Internal errors
impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Internal(err)
    }
}

// Automatically converts compile-time static `&str` text literals into Internal errors
impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::Internal(err.to_string())
    }
}