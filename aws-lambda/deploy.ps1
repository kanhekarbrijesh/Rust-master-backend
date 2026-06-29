# ─── Deploy AWS Lambda Image Preprocessor ──────────────────────────────────
# PowerShell deployment script
# ============================================================================

$ErrorActionPreference = "Stop"

# ─── Config ──────────────────────────────────────────────────────────────
$FUNCTION_NAME = "image-preprocessor"
$LAMBDA_ROLE = "arn:aws:iam::<account-id>:role/<lambda-execution-role>"  # ⚠️ CHANGE THIS
$BUCKET_NAME = "processed-uploads"
$REGION = "us-east-1"

Write-Host "🔨 Building Lambda binary..." -ForegroundColor Yellow
cargo build --release --target x86_64-unknown-linux-musl
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "📦 Packaging..." -ForegroundColor Yellow
cp "target/x86_64-unknown-linux-musl/release/aws-lambda-preprocess" ./bootstrap
if (Test-Path lambda.zip) { Remove-Item lambda.zip }
Compress-Archive -Path bootstrap -DestinationPath lambda.zip

# Check if function already exists
$EXISTS = aws lambda get-function --function-name $FUNCTION_NAME --region $REGION 2>$null
if ($EXISTS) {
    Write-Host "🔄 Updating existing Lambda function: $FUNCTION_NAME" -ForegroundColor Green
    aws lambda update-function-code `
        --function-name $FUNCTION_NAME `
        --zip-file fileb://lambda.zip `
        --region $REGION
} else {
    Write-Host "🚀 Creating new Lambda function: $FUNCTION_NAME" -ForegroundColor Green
    aws lambda create-function `
        --function-name $FUNCTION_NAME `
        --runtime provided.al2 `
        --role $LAMBDA_ROLE `
        --handler bootstrap `
        --zip-file fileb://lambda.zip `
        --region $REGION `
        --environment Variables="{DESTINATION_BUCKET=$BUCKET_NAME}"
}

Write-Host "✅ Done! Lambda function: $FUNCTION_NAME" -ForegroundColor Green
Write-Host ""
Write-Host "🔗 Test with:"
Write-Host "  aws lambda invoke --function-name $FUNCTION_NAME --payload '{\"image_base64\":\"...\"}' response.json"
