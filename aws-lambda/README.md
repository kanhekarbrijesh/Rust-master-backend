# AWS Lambda — Image Preprocessor

Standalone Rust AWS Lambda function for image preprocessing (resize + WebP conversion).

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Three Deployment Modes                         │
├──────────────┬──────────────────────┬──────────────────────────────┤
│  Mode 1      │  Mode 2              │  Mode 3                      │
│  API Gateway │  S3 Event Trigger    │  Direct SDK Invocation       │
│              │                      │                              │
│  Client      │  Raw S3 Bucket       │  Main Rust Backend           │
│    ↓         │    ↓ (upload)        │    ↓ (invoke)                │
│  API Gateway │  S3 Event → Lambda   │  Lambda (this)               │
│    ↓         │    ↓ (process)       │    ↓ (store)                 │
│  Lambda      │  Processed S3 Bucket │  S3 Bucket                   │
│    ↓         │                      │                              │
│  S3 Bucket   │                      │                              │
└──────────────┴──────────────────────┴──────────────────────────────┘
```

## Prerequisites

- Rust: `rustup target add x86_64-unknown-linux-musl`
- AWS CLI configured with appropriate credentials
- An S3 bucket for storing processed images

## Build & Deploy

### 1. Build for Lambda

```bash
cd aws-lambda
cargo build --release --target x86_64-unknown-linux-musl
```

### 2. Package

```bash
cp target/x86_64-unknown-linux-musl/release/aws-lambda-preprocess ./bootstrap
zip lambda.zip bootstrap
```

> **Note:** The binary must be named `bootstrap` for the AWS Lambda `provided.al2` runtime.

### 3. Create Lambda Function

```bash
aws lambda create-function \
  --function-name image-preprocessor \
  --runtime provided.al2 \
  --role arn:aws:iam::<account-id>:role/<lambda-execution-role> \
  --handler bootstrap \
  --zip-file fileb://lambda.zip \
  --environment Variables={DESTINATION_BUCKET=processed-uploads}
```

Or update an existing function:

```bash
aws lambda update-function-code \
  --function-name image-preprocessor \
  --zip-file fileb://lambda.zip
```

### 4. Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `DESTINATION_BUCKET` | No | `processed-uploads` | S3 bucket for processed images |
| `AWS_REGION` | No | `us-east-1` | AWS region |
| `RUST_LOG` | No | `info` | Log level |

### 5. IAM Role Permissions

The Lambda execution role needs:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:PutObject",
        "s3:GetObject"
      ],
      "Resource": "arn:aws:s3:::<your-bucket>/*"
    },
    {
      "Effect": "Allow",
      "Action": [
        "logs:CreateLogGroup",
        "logs:CreateLogStream",
        "logs:PutLogEvents"
      ],
      "Resource": "*"
    }
  ]
}
```

## Integration with Main Backend

### Option A: Integrated (same binary)

In `src/infrastructure/app_state.rs`:

```rust
let file_preprocessor: Arc<dyn FilePreprocessor> =
    Arc::new(AwsLambdaFilePreprocessor::new());
```

This uses the same compiled binary. Works on EC2, ECS, EKS, or even Lambda
(if you bundle the whole API server as a Lambda).

### Option B: Standalone Lambda (this project)

The main backend calls this Lambda via AWS SDK:

```rust
use aws_sdk_lambda::Client as LambdaClient;
use serde_json::json;

async fn invoke_preprocessor(
    client: &LambdaClient,
    image_bytes: &[u8],
    content_type: &str,
    file_name: &str,
) -> Result<String, AppError> {
    use base64::Engine;

    let payload = json!({
        "image_base64": base64::engine::general_purpose::STANDARD.encode(image_bytes),
        "content_type": content_type,
        "file_name": file_name,
        "destination_bucket": "processed-uploads",
        "directory": "uploads"
    });

    let result = client
        .invoke()
        .function_name("image-preprocessor")
        .payload(aws_sdk_lambda::primitives::Blob::new(
            serde_json::to_string(&payload).unwrap(),
        ))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Lambda invoke failed: {e}")))?;

    let bytes = result.payload().unwrap().as_ref();
    let response: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| AppError::Internal(format!("Lambda response parse failed: {e}")))?;

    Ok(response["url"].as_str().unwrap_or("").to_string())
}
```

## API Gateway Setup (Mode 1)

1. Create a REST API in API Gateway
2. Create a `POST /upload` resource
3. Set integration type to Lambda Proxy
4. Deploy the API
5. Client sends `POST https://api-gateway-url/upload` with:
   - Headers: `Content-Type: image/jpeg` (or image/png, image/webp)
   - Query params: `?filename=photo.jpg&directory=gallery`
   - Body: raw image bytes

## S3 Event Setup (Mode 2)

1. Create an S3 bucket (e.g., `raw-uploads`)
2. In bucket properties → Event Notifications → Create:
   - Event types: `s3:ObjectCreated:*`
   - Destination: Lambda function `image-preprocessor`
3. Processed files appear under `processed/` prefix in the same bucket

## Cost Considerations

| Resource | Estimated Cost |
|---|---|
| Lambda (128 MB, 1s execution) | ~$0.20 per million invocations |
| S3 storage | ~$0.023/GB/month |
| API Gateway | ~$3.50 per million requests |
