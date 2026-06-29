// ─── AWS LAMBDA — Standalone Image Preprocessing Function ───────────────────
//
// This Lambda function can be deployed in **three modes**:
//
// ─────────────────────────────────────────────────────────────────────────
// 🔹 Mode 1: API Gateway → Lambda (HTTP Upload Endpoint)
// ─────────────────────────────────────────────────────────────────────────
//   Client uploads an image → API Gateway → this Lambda → S3
//   The Lambda receives the raw image, preprocesses it (resize + WebP),
//   stores it in S3, and returns the public URL.
//
//   API Gateway setup:
//     - Route: POST /upload
//     - Integration: Lambda Proxy (passes multipart as base64)
//     - Request body: multipart/form-data with `image` field
//
// ─────────────────────────────────────────────────────────────────────────
// 🔹 Mode 2: S3 Event Trigger (Auto-process on Upload)
// ─────────────────────────────────────────────────────────────────────────
//   When a raw image is uploaded to an S3 bucket, this Lambda is triggered,
//   preprocesses the image, and stores the result in a processed bucket.
//
//   S3 Event setup:
//     - Bucket: raw-uploads
//     - Event: s3:ObjectCreated:*
//     - Destination: processed-uploads bucket
//
// ─────────────────────────────────────────────────────────────────────────
// 🔹 Mode 3: Direct SDK Invocation (from Main Backend)
// ─────────────────────────────────────────────────────────────────────────
//   The main Rust backend sends the image bytes to this Lambda via the
//   AWS SDK's Lambda::invoke() for CPU-intensive preprocessing.
//
// ─────────────────────────────────────────────────────────────────────────
// 📦 Deployment
// ─────────────────────────────────────────────────────────────────────────
// ```bash
// cd aws-lambda
// cargo build --release --target x86_64-unknown-linux-musl
// cp target/x86_64-unknown-linux-musl/release/aws-lambda-preprocess ./bootstrap
// zip lambda.zip bootstrap
// # Upload to AWS Lambda console or via AWS CLI
// aws lambda create-function \
//   --function-name image-preprocessor \
//   --runtime provided.al2 \
//   --role arn:aws:iam::<account>:role/<role> \
//   --handler bootstrap \
//   --zip-file fileb://lambda.zip
// ```
// ============================================================================

use aws_lambda_events::s3::S3Event;
use image::{DynamicImage, ImageReader, codecs::webp::WebPEncoder, imageops::FilterType};
use lambda_runtime::{Error, LambdaEvent, service_fn};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use tracing::info;

// ─── Configuration ──────────────────────────────────────────────────────────

/// Max dimension (width or height) for processed images.
const MAX_DIMENSION: u32 = 512;

/// Allowed MIME types.
const ALLOWED_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp", "image/gif"];

// ─── Request / Response Types ───────────────────────────────────────────────

/// Payload for direct invocation (Mode 3).
#[derive(Debug, Deserialize)]
struct PreprocessRequest {
    /// Raw image bytes (base64-encoded).
    image_base64: String,
    /// Original content type (e.g. "image/jpeg").
    content_type: String,
    /// Original file name.
    file_name: String,
    /// Target S3 bucket to store the result.
    destination_bucket: String,
    /// Target directory prefix in S3.
    #[serde(default = "default_directory")]
    directory: String,
}

fn default_directory() -> String {
    "processed".to_string()
}

/// Response from the Lambda (used in Mode 1 and Mode 3).
#[derive(Debug, Serialize)]
struct PreprocessResponse {
    success: bool,
    key: String,
    url: String,
    size: u64,
    content_type: String,
    format: String, // e.g. "webp"
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

// ─── Image Preprocessing Logic (reusable) ───────────────────────────────────

/// Process an image buffer: validate → resize → WebP conversion → optional encrypt.
fn preprocess_image(buffer: &[u8], content_type: &str) -> Result<(Vec<u8>, String), String> {
    // ── Validate MIME ───────────────────────────────────────────────────
    if !ALLOWED_MIME_TYPES.contains(&content_type) {
        return Err(format!(
            "Unsupported file type '{content_type}'. Allowed: {}",
            ALLOWED_MIME_TYPES.join(", ")
        ));
    }

    // ── Decode ──────────────────────────────────────────────────────────
    let img = ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()
        .map_err(|e| format!("Failed to read image: {e}"))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {e}"))?;

    // ── Resize ──────────────────────────────────────────────────────────
    let processed = if img.width() > MAX_DIMENSION || img.height() > MAX_DIMENSION {
        let ratio = (MAX_DIMENSION as f64 / img.width().max(img.height()) as f64).min(1.0);
        let new_w = (img.width() as f64 * ratio).round() as u32;
        let new_h = (img.height() as f64 * ratio).round() as u32;
        DynamicImage::ImageRgba8(image::imageops::resize(
            &img,
            new_w,
            new_h,
            FilterType::Lanczos3,
        ))
    } else {
        img
    };

    // ── Encode as WebP ──────────────────────────────────────────────────
    let mut webp_buf = Vec::new();
    {
        let encoder = WebPEncoder::new_lossless(&mut webp_buf);
        processed
            .write_with_encoder(encoder)
            .map_err(|e| format!("WebP encoding failed: {e}"))?;
    }

    // ── Encrypt (if FILE_ENCRYPTION_KEY is set) ─────────────────────────
    if let Ok(enc_key) = load_encryption_key() {
        let encrypted = encrypt_file(&webp_buf, &enc_key)?;
        return Ok((encrypted, "application/octet-stream".to_string()));
    }

    Ok((webp_buf, "image/webp".to_string()))
}

// ─── Encryption Helpers ─────────────────────────────────────────────────────

/// Load the 256-bit encryption key from FILE_ENCRYPTION_KEY env var.
fn load_encryption_key() -> Result<[u8; 32], String> {
    let key_hex = std::env::var("FILE_ENCRYPTION_KEY")
        .map_err(|_| "FILE_ENCRYPTION_KEY not set".to_string())?;
    let key_bytes = hex::decode(&key_hex)
        .map_err(|e| format!("Invalid hex: {e}"))?;
    if key_bytes.len() != 32 {
        return Err(format!("Key must be 64 hex chars, got {}", key_hex.len()));
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);
    Ok(key)
}

/// Encrypt plaintext using AES-256-GCM via `ring`.
fn encrypt_file(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
    use ring::rand::{SecureRandom, SystemRandom};

    let unbound_key =
        UnboundKey::new(&AES_256_GCM, key).map_err(|e| format!("Key init: {e}"))?;
    let sealing_key = LessSafeKey::new(unbound_key);

    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes).map_err(|e| format!("Nonce: {e}"))?;

    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = plaintext.to_vec();
    sealing_key
        .seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|e| format!("Encrypt: {e}"))?;

    let mut result = Vec::with_capacity(12 + in_out.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&in_out);
    Ok(result)
}

/// Decrypt ciphertext (nonce || encrypted || tag).
fn decrypt_file(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};

    if ciphertext.len() < 28 {
        return Err("Ciphertext too short".to_string());
    }

    let unbound_key =
        UnboundKey::new(&AES_256_GCM, key).map_err(|e| format!("Key init: {e}"))?;
    let opening_key = LessSafeKey::new(unbound_key);

    let (nonce_bytes, encrypted) = ciphertext.split_at(12);
    let nonce = Nonce::assume_unique_for_key(
        nonce_bytes.try_into().map_err(|_| "Invalid nonce".to_string())?,
    );

    let mut in_out = encrypted.to_vec();
    let plaintext = opening_key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| "Decryption failed: corrupt or wrong key".to_string())?;

    Ok(plaintext.to_vec())

// ─── S3 Helper ──────────────────────────────────────────────────────────────

/// Upload processed bytes to S3 and return the URL.
async fn upload_to_s3(
    bucket: &str,
    key: &str,
    buffer: &[u8],
    content_type: &str,
) -> Result<String, String> {
    let config = aws_config::from_env()
        .region(aws_sdk_s3::config::Region::new(
            std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".into()),
        ))
        .load()
        .await;

    let client = aws_sdk_s3::Client::new(&config);

    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(aws_sdk_s3::primitives::ByteStream::from(buffer.to_vec()))
        .content_type(content_type)
        .send()
        .await
        .map_err(|e| format!("S3 upload failed: {e}"))?;

    let url = format!("https://{bucket}.s3.amazonaws.com/{key}");
    Ok(url)
}

// ─── Handler Dispatch ───────────────────────────────────────────────────────

/// Main Lambda entrypoint.
/// Routes to the correct handler based on the event type.
async fn function_handler(
    event: LambdaEvent<serde_json::Value>,
) -> Result<serde_json::Value, Error> {
    let payload = event.payload;

    // Detect event type by checking which fields are present
    if payload.get("Records").is_some() {
        // Mode 2: S3 Event
        handle_s3_event(payload).await
    } else if payload.get("image_base64").is_some() {
        // Mode 3: Direct Invocation
        handle_direct_invoke(payload).await
    } else if payload.get("body").is_some() || payload.get("httpMethod").is_some() {
        // Mode 1: API Gateway Proxy
        handle_api_gateway(payload).await
    } else {
        Ok(serde_json::json!({
            "success": false,
            "error": "Unknown event type. Send either:\n\
                      1. API Gateway event (has 'httpMethod')\n\
                      2. S3 event (has 'Records')\n\
                      3. Direct invocation (has 'image_base64')"
        }))
    }
}

// ─── Mode 1: API Gateway Handler ────────────────────────────────────────────

async fn handle_api_gateway(event: serde_json::Value) -> Result<serde_json::Value, Error> {
    info!("Handling API Gateway request");

    // Extract the body (base64-encoded by API Gateway)
    let is_base64 = event
        .get("isBase64Encoded")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let body_str = event.get("body").and_then(|v| v.as_str()).unwrap_or("");

    let buffer = if is_base64 {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(body_str)
            .map_err(|e| format!("Base64 decode failed: {e}"))?
    } else {
        body_str.as_bytes().to_vec()
    };

    // Extract content type from headers
    let content_type = event
        .get("headers")
        .and_then(|h| h.get("content-type"))
        .or_else(|| event.get("headers").and_then(|h| h.get("Content-Type")))
        .and_then(|v| v.as_str())
        .unwrap_or("application/octet-stream")
        .to_string();

    // Extract file name from query params or body
    let file_name = event
        .get("queryStringParameters")
        .and_then(|q| q.get("filename"))
        .and_then(|v| v.as_str())
        .unwrap_or("upload")
        .to_string();

    let destination = event
        .get("queryStringParameters")
        .and_then(|q| q.get("directory"))
        .and_then(|v| v.as_str())
        .unwrap_or("uploads")
        .to_string();

    // ── Preprocess ──────────────────────────────────────────────────────
    let (processed_buf, final_ct) = match preprocess_image(&buffer, &content_type) {
        Ok(r) => r,
        Err(e) => {
            return Ok(serde_json::json!({
                "statusCode": 400,
                "headers": { "content-type": "application/json" },
                "body": serde_json::to_string(&ErrorResponse {
                    success: false,
                    error: e,
                }).unwrap()
            }));
        }
    };

    // ── Store to S3 ─────────────────────────────────────────────────────
    let bucket = std::env::var("DESTINATION_BUCKET").unwrap_or_else(|_| "processed-uploads".into());
    let sanitized: String = file_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let key = format!(
        "{}/{}-{}",
        destination.trim_end_matches('/'),
        uuid_v4(),
        sanitized
    );

    let url = upload_to_s3(&bucket, &key, &processed_buf, &final_ct)
        .await
        .map_err(|e| format!("{e}"))?;

    let size = processed_buf.len() as u64;

    Ok(serde_json::json!({
        "statusCode": 200,
        "headers": {
            "content-type": "application/json",
            "access-control-allow-origin": "*"
        },
        "body": serde_json::to_string(&PreprocessResponse {
            success: true,
            key,
            url,
            size,
            content_type: final_ct,
            format: "webp".to_string(),
        }).unwrap()
    }))
}

// ─── Mode 2: S3 Event Handler ───────────────────────────────────────────────

async fn handle_s3_event(event: serde_json::Value) -> Result<serde_json::Value, Error> {
    info!("Handling S3 event");

    let s3_event: S3Event = serde_json::from_value(event)?;

    for record in &s3_event.records {
        let bucket = record.s3.bucket.name.as_deref().unwrap_or("");
        let key = record.s3.object.key.as_deref().unwrap_or("");

        // Skip already-processed files (files in the processed prefix)
        if key.starts_with("processed/") {
            continue;
        }

        // ── Download from S3 ────────────────────────────────────────────
        let config = aws_config::from_env().load().await;
        let client = aws_sdk_s3::Client::new(&config);

        let get_output = client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| format!("S3 get failed: {e}"))?;

        let content_type = get_output
            .content_type()
            .unwrap_or("image/jpeg")
            .to_string();
        let buffer = get_output
            .body
            .collect()
            .await
            .map_err(|e| format!("Failed to read S3 body: {e}"))?
            .to_vec();

        // ── Preprocess ──────────────────────────────────────────────────
        let file_name = key.rsplit('/').next().unwrap_or("image");
        let (processed_buf, final_ct) = match preprocess_image(&buffer, &content_type) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(key = %key, error = %e, "S3 event: preprocessing failed");
                continue;
            }
        };

        // ── Store processed version ─────────────────────────────────────
        let processed_key = format!("processed/{}", key);
        let _ = upload_to_s3(bucket, &processed_key, &processed_buf, &final_ct).await;
        info!(key = %processed_key, "S3 event: processed image stored");
    }

    Ok(serde_json::json!({"success": true}))
}

// ─── Mode 3: Direct Invocation Handler ──────────────────────────────────────

async fn handle_direct_invoke(payload: serde_json::Value) -> Result<serde_json::Value, Error> {
    info!("Handling direct invocation");

    let req: PreprocessRequest = serde_json::from_value(payload)?;

    // Decode base64 image
    use base64::Engine;
    let buffer = base64::engine::general_purpose::STANDARD
        .decode(&req.image_base64)
        .map_err(|e| format!("Base64 decode failed: {e}"))?;

    // Sanitize file name + build key
    let sanitized: String = req
        .file_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let base_name = sanitized
        .rfind('.')
        .map(|dot| &sanitized[..dot])
        .unwrap_or(&sanitized);
    let key = format!(
        "{}/{}-{}.webp",
        req.directory.trim_end_matches('/'),
        uuid_v4(),
        base_name
    );

    // ── Preprocess ──────────────────────────────────────────────────────
    let (processed_buf, final_ct) = preprocess_image(&buffer, &req.content_type)?;

    // ── Store to S3 ─────────────────────────────────────────────────────
    let url = upload_to_s3(&req.destination_bucket, &key, &processed_buf, &final_ct).await?;

    let size = processed_buf.len() as u64;

    Ok(serde_json::json!(PreprocessResponse {
        success: true,
        key,
        url,
        size,
        content_type: final_ct,
        format: "webp".to_string(),
    }))
}

// ─── Utility ────────────────────────────────────────────────────────────────

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:032x}", nanos)
}

// ─── Entrypoint ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    info!("AWS Lambda image preprocessor starting");

    let func = service_fn(function_handler);
    lambda_runtime::run(func).await?;

    Ok(())
}
