use std::sync::Arc;

use mongodb::Database;
use sqlx::PgPool;

use crate::{
    configuration::config::Configs,
    infrastructure::{
        db::mongodb::mongodb_connection::mongodb_connection,
        db::postgresql::psql_connction::psql_connection, storage::storage_types::StorageProvider,
    },
    services::domain_services::user_roles_psql_services::UserRolesPsqlService,
};

#[derive(Clone)]
#[allow(dead_code)] // 👈 Temporary muzzle for the compiler warning
pub struct AppState {
    pub db: Database, // The live database instance
    pub mongodb_collections:
        crate::infrastructure::db::mongodb::mongodb_collections::MongodbCollections,
    // postgresql pgpool
    pub psql_pool: PgPool,
    pub user_role_repo: UserRolesPsqlService,
    // ─── Storage ─────────────────────────────────────────────────────────
    /// The active storage provider (Local, S3, R2, etc.)
    pub storage: Arc<dyn StorageProvider>,
    /// The URL prefix used to serve stored files (e.g. "/uploads").
    /// This is injected from config and used by controllers & services to
    /// reconstruct storage keys from URLs — keeping it **fully decoupled**
    /// from any single backend implementation.
    pub storage_serve_prefix: String,
}

impl AppState {
    pub async fn new(config: Configs) -> Self {
        // --------------------------------------------------------------- start : mongodb setup --------
        // mongodb setup
        let mongodb_uri = &config.mongo_uri.clone();
        let db = mongodb_connection(mongodb_uri).await;
        let mongodb_collections =
            crate::infrastructure::db::mongodb::mongodb_collections::MongodbCollections::new(
                db.clone(),
            );
        // --------------------------------------------------------------- end : mongodb setup --------

        // --------------------------------------------------------------- start : postgresql setup --------
        // Create a connection pool
        let postgresql_url = &config.postgresql_neon_pool_url;

        let psql_pool = psql_connection(postgresql_url)
            .await
            .expect("Failed to connect to postgresql");

        let user_role_repo = UserRolesPsqlService;
        // let user_role_repo2 = UserRolesPsqlService;
        // --------------------------------------------------------------- start : postgresql setup --------

        // --------------------------------------------------------------- start : storage setup --------
        let (storage, storage_serve_prefix): (Arc<dyn StorageProvider>, String) =
            match config.storage_provider.as_str() {
                "aws" => {
                    let s3_url_base = format!(
                        "https://{}.s3.{}.amazonaws.com",
                        config.aws_bucket, config.aws_region
                    );
                    (
                        Arc::new(
                            crate::infrastructure::storage::aws::AwsS3Storage::new(
                                &config.aws_bucket,
                                &config.aws_region,
                            )
                            .await,
                        ),
                        s3_url_base,
                    )
                }
                "cloudflare" => {
                    // When r2_public_url is set, it's used as the public-facing
                    // URL base. Otherwise we fall back to the raw S3 endpoint
                    // URL which includes the full key (with prefix).
                    let has_public_url = !config.r2_public_url.is_empty();
                    let public_url = if has_public_url {
                        config.r2_public_url.clone()
                    } else {
                        String::new()
                    };
                    // storage_serve_prefix must match the URL structure so that
                    // storage_key_from_url() can extract the key during delete.
                    // With public URL:   {public_url}/{raw_key}
                    // Without public URL: {endpoint}/{full_key}  (includes prefix)
                    // We store the serve prefix that matches.
                    let serve_prefix = if has_public_url {
                        public_url.clone()
                    } else {
                        config.r2_endpoint.clone()
                    };
                    (
                        Arc::new(
                            crate::infrastructure::storage::cloudflare::CloudflareR2Storage::new(
                                &config.r2_access_key,
                                &config.r2_secret_key,
                                &config.r2_endpoint,
                                &config.r2_bucket,
                                &config.r2_key_prefix,
                                &public_url,
                            )
                            .await,
                        ),
                        serve_prefix,
                    )
                }
                _ => (
                    Arc::new(
                        crate::infrastructure::storage::localstorage::LocalStorage::new(
                            &config.storage_local_path,
                            &config.storage_local_serve_prefix,
                        ),
                    ),
                    config.storage_local_serve_prefix.clone(),
                ),
            };
        // --------------------------------------------------------------- end : storage setup --------

        Self {
            db,
            mongodb_collections,
            psql_pool,
            user_role_repo,
            storage,
            storage_serve_prefix,
        }
    }
}
