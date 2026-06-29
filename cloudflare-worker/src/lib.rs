// ─── CLOUDFLARE WORKER — Rust (workers-rs) ─────────────────────────────────
//
// This is a Cloudflare Worker written entirely in Rust using the `worker`
// crate. It sits in front of your R2 bucket and optionally preprocesses
// images before storing them.
//
// **Architecture:**
//   Client → Cloudflare Worker (this) → R2 Bucket
//
// **What it does:**
//   1. Receives multipart upload requests
//   2. Validates MIME type and file size
//   3. (Optional) Preprocesses images (resize + format conversion)
//   4. Stores the file in R2
//   5. Returns the public URL
//
// **Deployment:**
//   ```bash
//   cd cloudflare-worker
//   npx wrangler deploy
//   ```
//
// **Prerequisites:**
//   - `wrangler.toml` configured with R2 bucket binding
//   - Rust target: `rustup target add wasm32-unknown-unknown`
// ============================================================================

mod encryption;

use serde::{Deserialize, Serialize};
use worker::*;

// ─── Configuration ──────────────────────────────────────────────────────────

/// Allowed MIME types for image uploads.
const ALLOWED_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp", "image/gif"];

/// Maximum upload size: 10 MB.
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

// ─── Request/Response types ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct UploadQuery {
    #[serde(default)]
    directory: String,
}

#[derive(Debug, Serialize)]
struct UploadResponse {
    success: bool,
    key: String,
    url: String,
    size: u64,
    content_type: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

// ─── Worker Entrypoint ──────────────────────────────────────────────────────

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // ── Parse the request ───────────────────────────────────────────────
    let method = req.method();
    let path = req.path();
    let url = req.url()?;

    // ── CORS preflight ──────────────────────────────────────────────────
    if method == Method::Options {
        return cors_response(Response::empty()?);
    }

    // ── Route: GET / → health check ─────────────────────────────────────
    if method == Method::Get && path == "/" {
        return Response::ok(r#"{"status":"ok","worker":"rust-tut-preprocess"}"#);
    }

    // ── Route: POST /upload → handle file upload ────────────────────────
    if method == Method::Post && path == "/upload" {
        return handle_upload(req, &env).await;
    }

    // ── Route: DELETE /{key} → delete file ──────────────────────────────
    if method == Method::Delete && path.starts_with('/') {
        let key = path.trim_start_matches('/');
        return handle_delete(key, &env).await;
    }

    // ── 404 ─────────────────────────────────────────────────────────────
    Response::error("Not Found", 404)
}

// ─── Upload Handler ─────────────────────────────────────────────────────────

async fn handle_upload(mut req: Request, env: &Env) -> Result<Response> {
    // ── 1. Get R2 bucket binding ────────────────────────────────────────
    let bucket = match env.bucket("R2_BUCKET") {
        Ok(b) => b,
        Err(_) => {
            return error_response(500, "R2_BUCKET binding not configured in wrangler.toml");
        }
    };

    // ── 2. Parse multipart form data ────────────────────────────────────
    let form_data = match req.multipart().await {
        Ok(f) => f,
        Err(e) => return error_response(400, &format!("Invalid multipart: {e}")),
    };

    let mut file_data: Option<(Vec<u8>, String, String)> = None; // (bytes, filename, content_type)
    let mut directory = String::from("uploads");

    for (name, part) in form_data.into_iter() {
        match part {
            FormEntry::File(bytes) => {
                let filename = name.clone();
                let content_type = bytes
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                file_data = Some((bytes.into_bytes(), filename, content_type));
            }
            FormEntry::Field(value) => {
                if name == "directory" && !value.is_empty() {
                    directory = value;
                }
            }
        }
    }

    let (buffer, file_name, content_type) = match file_data {
        Some(d) => d,
        None => return error_response(400, "No file found in upload"),
    };

    // ── 3. Validate ─────────────────────────────────────────────────────
    if buffer.is_empty() {
        return error_response(400, "Uploaded file is empty");
    }
    if buffer.len() > MAX_FILE_SIZE {
        return error_response(
            400,
            &format!("File exceeds max size of {MAX_FILE_SIZE} bytes"),
        );
    }
    if !ALLOWED_MIME_TYPES.contains(&content_type.as_str()) {
        return error_response(
            400,
            &format!(
                "Unsupported file type '{content_type}'. Allowed: {}",
                ALLOWED_MIME_TYPES.join(", ")
            ),
        );
    }

    // ── 4. (Optional) Image preprocessing ───────────────────────────────
    // Uncomment to enable image processing via photon-rs:
    // let (processed_buffer, final_content_type) = preprocess_image(&buffer, &content_type)?;
    // let processed_file_name = ...;
    //
    // For now: pass-through (no transformation)
    let processed_buffer = buffer;
    let final_content_type = content_type.clone();
    let processed_file_name = file_name;

    // ── 4b. Encryption (if FILE_ENCRYPTION_KEY is set) ──────────────────
    let (store_buffer, store_content_type, is_encrypted) =
        if let Ok(enc_key) = encryption::load_encryption_key() {
            let encrypted = encryption::encrypt_file(&processed_buffer, &enc_key)
                .map_err(|e| error_response(500, &e).unwrap_err())?;
            (encrypted, "application/octet-stream".to_string(), true)
        } else {
            (processed_buffer, final_content_type.clone(), false)
        };

    // ── 5. Build storage key ────────────────────────────────────────────
    let uuid = uuid_v4();
    let sanitized: String = processed_file_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let key = format!("{}/{}", directory.trim_end_matches('/'), uuid);

    // ── 6. Store in R2 ──────────────────────────────────────────────────
    let mut put = bucket.put(&key, store_buffer);
    put = put.content_type(&store_content_type);
    // Store encryption marker as custom metadata so the Worker knows to decrypt on read
    put = put.custom_metadata("x-encrypted", if is_encrypted { "true" } else { "false" });
    put.execute().await?;

    let size = buffer.len() as u64;

    // ── 7. Build public URL ─────────────────────────────────────────────
    // Change this to your public domain or use the R2.dev URL
    let public_url_base = env
        .var("PUBLIC_URL_BASE")
        .unwrap_or("https://pub-<your-id>.r2.dev".into());
    let url = format!(
        "{}/{}",
        public_url_base.to_string().trim_end_matches('/'),
        key
    );

    Ok(cors_response(Response::from_json(&UploadResponse {
        success: true,
        key,
        url,
        size,
        content_type,
    })?)?)
}

// ─── Delete Handler ─────────────────────────────────────────────────────────

async fn handle_delete(key: &str, env: &Env) -> Result<Response> {
    let bucket = match env.bucket("R2_BUCKET") {
        Ok(b) => b,
        Err(_) => {
            return error_response(500, "R2_BUCKET binding not configured");
        }
    };

    bucket.delete(key).await?;
    Response::ok(r#"{"success":true}"#)
}

// ─── Image Preprocessing (WASM-compatible) ──────────────────────────────────
//
// To enable image preprocessing in the Worker, add `photon-rs` to Cargo.toml
// and uncomment this function.
//
// ```rust
// use photon_rs::transform::resize;
// use photon_rs::native::open_image_from_bytes;
//
// fn preprocess_image(
//     buffer: &[u8],
//     content_type: &str,
// ) -> Result<(Vec<u8>, String), String> {
//     let mut img = open_image_from_bytes(buffer)
//         .map_err(|e| format!("Failed to decode image: {e}"))?;
//
//     // Resize to max 512px
//     let max_dim = 512_u32;
//     if img.get_width() > max_dim || img.get_height() > max_dim {
//         let ratio = max_dim as f64 / img.get_width().max(img.get_height()) as f64;
//         let new_w = (img.get_width() as f64 * ratio).round() as u32;
//         let new_h = (img.get_height() as f64 * ratio).round() as u32;
//         img = resize(&img, new_w, new_h, photon_rs::transform::SamplingFilter::Lanczos3);
//     }
//
//     // Encode as JPEG (PNG also supported)
//     let output = photon_rs::native::open_image_bytes(img);
//
//     Ok((output, "image/jpeg".to_string()))
// }
// ```

// ─── Helpers ────────────────────────────────────────────────────────────────

fn uuid_v4() -> String {
    // Simple UUID v4 generation (or use the `uuid` crate if added)
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:032x}", nanos)
}

fn error_response(status: u16, message: &str) -> Result<Response> {
    Response::from_json(&ErrorResponse {
        success: false,
        error: message.to_string(),
    })
    .and_then(|r| r.with_status(status))
    .map(|r| cors_response(r).unwrap_or(r))
}

fn cors_response(mut resp: Response) -> Result<Response> {
    let headers = resp.headers_mut();
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;
    Ok(resp)
}
