# Gallery E2E Hurl workflow
# Tests full lifecycle: create (upload), read, update, delete (file cleanup)
#
# Prerequisites:
#   - Server running on localhost:8080
#   - Test image at tests/test-files/gallery_test_image.png
#   - Run with: hurl --test --file-root . hurl\gallery.hurl

# ─── Step 1: Create a gallery item with image upload ─────────────────────────
POST http://localhost:8080/api/private/v1/gallery
[Multipart]
status: active
image: file,tests/test-files/gallery_test_image.png; image/png
HTTP/1.1 201
[Captures]
gallery_id: jsonpath "$.id"
[Asserts]
jsonpath "$.id" exists

# ─── Step 2: Read all gallery items ──────────────────────────────────────────
GET http://localhost:8080/api/private/v1/gallery
HTTP/1.1 200
[Asserts]
jsonpath "$[0].id" exists
jsonpath "$[0].url" exists
jsonpath "$[0].status" exists

# ─── Step 3: Read gallery item by ID ─────────────────────────────────────────
GET http://localhost:8080/api/private/v1/gallery/{{gallery_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.id" == "{{gallery_id}}"
jsonpath "$.url" exists
jsonpath "$.status" == "active"

# ─── Step 4: Update gallery item status ──────────────────────────────────────
PUT http://localhost:8080/api/private/v1/gallery/{{gallery_id}}
Content-Type: application/json
{
  "id": "{{gallery_id}}",
  "status": "inactive"
}
HTTP/1.1 200
[Asserts]
jsonpath "$.message" == "Gallery item updated successfully"

# ─── Step 5: Verify the update via GET ───────────────────────────────────────
GET http://localhost:8080/api/private/v1/gallery/{{gallery_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.status" == "inactive"

# ─── Step 6: Delete gallery item (also cleans up the stored file) ────────────
DELETE http://localhost:8080/api/private/v1/gallery/{{gallery_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.message" == "Gallery item deleted successfully"

# ─── Step 7: Verify it's gone (should return 404) ────────────────────────────
GET http://localhost:8080/api/private/v1/gallery/{{gallery_id}}
HTTP/1.1 404
[Asserts]
jsonpath "$.error" == "Not Found"
