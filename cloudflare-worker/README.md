# Cloudflare Worker — Image Preprocessor (Rust)

A Cloudflare Worker written entirely in **Rust** using the `workers-rs` crate.
It compiles to WASM and runs inside Cloudflare's V8 isolate.

## Architecture

```
Client → Cloudflare Worker (this) → R2 Bucket
```

The Worker:
1. Receives multipart upload requests
2. Validates MIME type and file size
3. Optionally preprocesses images (resize + format conversion)
4. Stores the file in R2
5. Returns the public URL

## Prerequisites

- Node.js and `npm` (for `wrangler`)
- Rust: `rustup target add wasm32-unknown-unknown`
- A Cloudflare account with R2 enabled

## Setup

```bash
npm install -g wrangler
npx wrangler login
```

Create the R2 bucket:

```bash
npx wrangler r2 bucket create rust-tut-uploads
```

## Build & Deploy

```bash
cd cloudflare-worker
npx wrangler deploy
```

## Configuration

Edit `wrangler.toml`:

- `bucket_name` — Your R2 bucket name
- `routes` — (Optional) Custom domain route
- `PUBLIC_URL_BASE` — (Optional) env var for public URL base

## Integration with Main Backend

The main Rust backend uses `CfWorkerFilePreprocessor` when running behind
this Worker. It validates MIME + file size and passes through the file.

In `src/infrastructure/app_state.rs`:

```rust
let file_preprocessor: Arc<dyn FilePreprocessor> =
    Arc::new(CfWorkerFilePreprocessor::new());
```

The Worker is the upload endpoint. The Rust API handles everything else.

## Image Processing

For full image preprocessing inside the Worker (resize + WebP),
add `photon-rs` to `Cargo.toml` and uncomment the preprocessing
code in `src/lib.rs`.

The `image` crate's `webp` feature has limited WASM support,
so `photon-rs` is recommended for WASM environments.

## API Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/` | Health check |
| `POST` | `/upload` | Upload + preprocess image |
| `DELETE` | `/{key}` | Delete image by key |

### Upload

```bash
curl -X POST https://worker.yourdomain.workers.dev/upload \
  -F "image=@photo.jpg" \
  -F "directory=gallery"
```

### Delete

```bash
curl -X DELETE https://worker.yourdomain.workers.dev/uploads/uuid-photo.jpg
```
