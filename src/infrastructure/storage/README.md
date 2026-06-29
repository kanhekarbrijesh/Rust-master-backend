# Storage Infrastructure

This module provides a pluggable storage and file preprocessing architecture for the application.

## Architecture Overview

```
infrastructure/storage/
├── mod.rs                        # Module declarations + re-exports
├── storage_types.rs              # Core StorageProvider trait + shared types
├── storage_util.rs               # Centralized multipart upload helpers
├── image_preprocessing.rs        # [LEGACY] Backward-compat wrapper
├── file_preprocess/
│   ├── mod.rs                    # FilePreprocessor trait + shared types
│   ├── file_preprocess_local.rs  # Full local image processing
│   ├── file_preprocess_cf_worker.rs  # Cloudflare Worker (pass-through)
│   └── file_preprocess_aws_lambda.rs # AWS Lambda (same as local)
├── localstorage/
│   └── mod.rs                    # Local filesystem storage provider
├── aws/
│   └── mod.rs                    # AWS S3 storage provider
├── cloudflare/
│   └── mod.rs                    # Cloudflare R2 storage provider
└── README.md                     # This file
```

## Storage Provider (backends)

The `StorageProvider` trait in `storage_types.rs` defines 4 methods:

| Method | Purpose |
|--------|---------|
| `store_file(input)` | Store a file, return key + URL |
| `get_file_url(key)` | Get public URL for a stored file |
| `delete_file(key)` | Delete a stored file |
| `file_exists(key)` | Check if a file exists |

**Currently implemented backends:**
- `LocalStorage` — Filesystem storage
- `AwsS3Storage` — AWS S3
- `CloudflareR2Storage` — Cloudflare R2 (S3-compatible)

Switching between backends is **env-driven** via `STORAGE_PROVIDER` env var.

---

## File Preprocessors (swappable at code level)

### Why code-level swapping?

Cloudflare Workers run in a V8 isolate where native Rust crates like `image`
cannot be used directly. AWS Lambda runs in a full micro-VM where they work
fine. By swapping at **code instantiation time** (not env vars), you get
compile-time guarantees that only the right code is compiled for each target.

### Available preprocessors

| Preprocessor | Location | Runtime | Image processing | Dependencies |
|---|---|---|---|---|
| `LocalFilePreprocessor` | `file_preprocess_local.rs` | Native binary | ✅ Resize + WebP | `image` crate |
| `CfWorkerFilePreprocessor` | `file_preprocess_cf_worker.rs` | V8 isolate | ❌ Pass-through only | None (std) |
| `AwsLambdaFilePreprocessor` | `file_preprocess_aws_lambda.rs` | Firecracker VM | ✅ Resize + WebP | `image` crate |

### How to swap

Edit `src/infrastructure/app_state.rs`, find the `FilePreprocessor` setup:

```rust
// 🔁 SWAP HERE: Change the concrete type to switch preprocessing.
let file_preprocessor: Arc<dyn FilePreprocessor> = Arc::new(LocalFilePreprocessor::new());
```

Change `LocalFilePreprocessor::new()` to:

```rust
// For Cloudflare Worker:
Arc::new(CfWorkerFilePreprocessor::new())

// For AWS Lambda:
Arc::new(AwsLambdaFilePreprocessor::new())
```

### Using the preprocessor in controllers

**New code** (trait-based, recommended):

```rust
use crate::infrastructure::storage::storage_util;

let result = storage_util::handle_multipart_upload_with_preprocessor(
    &*state.storage,
    &*state.file_preprocessor,
    "gallery",
    multipart,
).await?;
```

**Legacy code** (still works via backward-compat wrapper):

```rust
let result = storage_util::handle_multipart_upload_with_preprocessing(
    &*state.storage,
    "gallery",
    multipart,
).await?;
```

Both approaches produce the same result. The legacy functions use
`LocalFilePreprocessor` internally.

## Upload helpers (`storage_util.rs`)

| Helper | File required | Preprocessing |
|---|---|---|
| `handle_multipart_upload` | ✅ Yes | ❌ No |
| `handle_multipart_upload_with_preprocessing` | ✅ Yes | ✅ Legacy (image crate) |
| `handle_multipart_upload_optional` | ❌ No | ❌ No |
| `handle_multipart_upload_optional_with_preprocessing` | ❌ No | ✅ Legacy (image crate) |
| `handle_multipart_upload_with_preprocessor` | ✅ Yes | ✅ Trait-based (swappable) |
| `handle_multipart_upload_optional_with_preprocessor` | ❌ No | ✅ Trait-based (swappable) |

Plus:
- `spawn_orphan_cleanup()` — Fire-and-forget file delete on DB failure
- `delete_file_quietly()` — Best-effort file deletion
- `storage_key_from_url()` — Extract storage key from URL
- `resolve_file_url_on_update()` — Handle old-file cleanup on update

## PreprocessingConfig

Both trait-based and legacy preprocessors accept a `PreprocessingConfig`:

```rust
PreprocessingConfig {
    max_dimension: 512,       // Max width/height (0 = no resize)
    convert_to_webp: true,    // Convert to WebP
    allowed_mime_types: vec![ // Empty = allow all
        "image/jpeg".into(),
        "image/png".into(),
        "image/webp".into(),
        "image/gif".into(),
    ],
}
```

## Testing

```bash
# Run unit tests for all preprocessors
cargo test --lib

# Run integration tests (requires running server)
cargo run
# In another terminal:
hurl --test --file-root . hurl\file_preprocess.hurl
hurl --test --file-root . hurl\gallery.hurl
```

## Deployment

### Local (EC2 / VPS / desktop)
Just `cargo run` — uses `LocalFilePreprocessor` + `LocalStorage`.
Works on any x86_64 Linux/Windows/Mac system.

### AWS Lambda

**Two approaches:**

**A) Integrated (same binary as API server)**
   In `app_state.rs`, swap to `AwsLambdaFilePreprocessor`:
   ```rust
   let fp: Arc<dyn FilePreprocessor> = Arc::new(AwsLambdaFilePreprocessor::new());
   ```
   Build with `cargo build --release --target x86_64-unknown-linux-musl`
   Package and deploy as a single Lambda function.

**B) Standalone Lambda function (recommended for offloading)**
   See `aws-lambda/` at the project root — a fully self-contained Rust project.
   ```
   cd aws-lambda
   cargo build --release --target x86_64-unknown-linux-musl
   cp target/.../aws-lambda-preprocess ./bootstrap
   zip lambda.zip bootstrap
   aws lambda create-function ...
   ```
   The Lambda can be triggered by API Gateway, S3 events, or direct SDK invocation.
   See `aws-lambda/README.md` for full details.

### Cloudflare Worker

**YES — you can write the Worker in Rust!** Using `workers-rs` crate (compiles to WASM).

See `cloudflare-worker/` at the project root — a fully self-contained Rust project:

```bash
cd cloudflare-worker
rustup target add wasm32-unknown-unknown
npx wrangler deploy
```

The Worker in `cloudflare-worker/src/lib.rs`:
- Written entirely in Rust using the `worker` crate (not JS/TS!)
- Validates MIME type and file size
- Optionally preprocesses images (using `photon-rs` for WASM compatibility)
- Stores files in R2 bucket
- Returns public URLs

**How it connects to the Rust API:**
1. Deploy the Worker
2. Point your domain to the Worker
3. In `app_state.rs`, swap to `CfWorkerFilePreprocessor`
4. The Worker handles uploads, the Rust API handles everything else

## Project Structure

```
project-root/
├── src/
│   └── infrastructure/storage/
│       ├── file_preprocess/
│       │   ├── mod.rs                        # FilePreprocessor trait
│       │   ├── file_preprocess_local.rs      # Integrated: native binary (EC2, VPS)
│       │   ├── file_preprocess_cf_worker.rs  # Integrated: behind CF Worker
│       │   └── file_preprocess_aws_lambda.rs # Integrated: in Lambda/EC2
│       └── ...
├── cloudflare-worker/                        # STANDALONE Rust CF Worker project
│   ├── Cargo.toml
│   ├── wrangler.toml
│   └── src/lib.rs
├── aws-lambda/                               # STANDALONE Rust Lambda project
│   ├── Cargo.toml
│   ├── deploy.ps1
│   └── src/main.rs
└── ...
```
