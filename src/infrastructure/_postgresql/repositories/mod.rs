use std::pin::Pin;
use thiserror::Error;

pub mod user_roles_psql_repo;

#[derive(Error, Debug)]
pub enum PsqlAppError {
    #[error("Database error occurred: {0}")]
    DatabaseError(#[from] sqlx::Error), // Automatically converts SQLx errors to AppError

    #[error("User role already exists")]
    AlreadyExists,

    #[error("Internal server error: {0}")]
    Internal(String),
}

// A convenient type alias to keep your code clean
pub type PsqlRepoFuture<T> = Pin<Box<dyn Future<Output = Result<T, PsqlAppError>> + Send>>;
