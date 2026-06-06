# required dependencies
[dependencies]
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "macros"] }
dotenvy = "0.15"

