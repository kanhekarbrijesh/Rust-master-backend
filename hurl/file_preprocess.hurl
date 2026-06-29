# File Preprocessing E2E Hurl workflow
# Tests that the file preprocessing pipeline works end-to-end
# through the gallery upload endpoint (which uses image preprocessing).
#
# Prerequisites:
#   - Server running on localhost:8080
#   - Test images at tests/test-files/gallery_test_image.png
#   - Run with: hurl --test --file-root . hurl\file_preprocess.hurl

# ─── Step 1: Upload a gallery image (triggers preprocessing) ─────────────────
POST http://localhost:8080/api/private/v1/gallery
[Multipart]
status: active
image: file,tests/test-files/gallery_test_image.png; image/png
HTTP/1.1 201
[Captures]
gallery_id: jsonpath "$.id"
[Asserts]
jsonpath "$.id" exists

# ─── Step 2: Verify the preprocessed image (should be .webp) ─────────────────
GET http://localhost:8080/api/private/v1/gallery/{{gallery_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.id" == "{{gallery_id}}"
# The stored URL should end in .webp (preprocessing converts to WebP)
jsonpath "$.url" matches ".webp$"
jsonpath "$.status" == "active"

# ─── Step 3: Upload an unsupported file type (should fail at MIME check) ─────
POST http://localhost:8080/api/private/v1/gallery
[Multipart]
status: active
image: file,tests/test-files/test.txt; text/plain
HTTP/1.1 400
[Asserts]
jsonpath "$.error" == "Bad Request"
jsonpath "$.details" exists

# ─── Step 4: Clean up test data ──────────────────────────────────────────────
DELETE http://localhost:8080/api/private/v1/gallery/{{gallery_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.message" == "Gallery item deleted successfully"
