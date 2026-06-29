# Users E2E Hurl workflow
#
# Prerequisites:
#   - Server must be running on localhost:8080
#   - A user role with id=1 must exist in the database
#   - Test image at src/tests/test_files/user_placeholder_icon.png
#   - Run with: hurl --test --file-root . hurl\users.hurl

# ─── Step 1: Create a user with profile image ───────────────────────────────
POST http://localhost:8080/api/private/v1/users
[Multipart]
name: Hurl Test User
role_id: 1
profile_image: file,src/tests/test_files/user_placeholder_icon.png; image/png
HTTP/1.1 201
[Captures]
user_id: jsonpath "$.id"
[Asserts]
jsonpath "$.id" exists

# ─── Step 2: Read all users ──────────────────────────────────────────────────
GET http://localhost:8080/api/private/v1/users
HTTP/1.1 200

# ─── Step 3: Read user by ID ─────────────────────────────────────────────────
GET http://localhost:8080/api/private/v1/users/{{user_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.name" == "Hurl Test User"
jsonpath "$.role_id" == 1
jsonpath "$.profile_image" exists

# ─── Step 4: Update user WITHOUT a new image (file is optional) ──────────────
PUT http://localhost:8080/api/private/v1/users/{{user_id}}
[Multipart]
name: Hurl Test User Updated
role_id: 1
HTTP/1.1 200
[Asserts]
jsonpath "$.name" == "Hurl Test User Updated"
jsonpath "$.profile_image" exists

# ─── Step 5: Update user WITH a new image ────────────────────────────────────
PUT http://localhost:8080/api/private/v1/users/{{user_id}}
[Multipart]
name: Hurl Test User Final
role_id: 1
profile_image: file,src/tests/test_files/user_placeholder_icon.png; image/png
HTTP/1.1 200
[Asserts]
jsonpath "$.name" == "Hurl Test User Final"
jsonpath "$.profile_image" exists

# ─── Step 6: Delete user (also cleans up the stored file) ────────────────────
DELETE http://localhost:8080/api/private/v1/users/{{user_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.message" == "User deleted successfully"
