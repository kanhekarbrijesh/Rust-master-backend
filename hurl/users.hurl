# Users E2E Hurl workflow
# Prerequisite: A test image file must exist at src/tests/test_files/user_placeholder_icon.png
# and a user role with id=1 must exist in the database.

# ─── Step 1: Create a user with profile image ───────────────────────────────
POST http://localhost:8080/api/private/v1/users
Content-Type: multipart/form-data; boundary=----boundary123
------boundary123
Content-Disposition: form-data; name="name"

Hurl Test User
------boundary123
Content-Disposition: form-data; name="role_id"

1
------boundary123
Content-Disposition: form-data; name="profile_image"; filename="user_placeholder_icon.png"
Content-Type: image/png

< ./src/tests/test_files/user_placeholder_icon.png
------boundary123--
HTTP/1.1 201
[Captures]
user_id: jsonpath "$.id"

# ─── Step 2: Read all users ──────────────────────────────────────────────────
GET http://localhost:8080/api/private/v1/users
HTTP/1.1 200

# ─── Step 3: Read user by ID ─────────────────────────────────────────────────
GET http://localhost:8080/api/private/v1/users/{{user_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.name" == "Hurl Test User"
jsonpath "$.role_id" == 1

# ─── Step 4: Update user with new profile image ──────────────────────────────
PUT http://localhost:8080/api/private/v1/users/{{user_id}}
Content-Type: multipart/form-data; boundary=----boundary456
------boundary456
Content-Disposition: form-data; name="name"

Hurl Test User Updated
------boundary456
Content-Disposition: form-data; name="role_id"

1
------boundary456
Content-Disposition: form-data; name="profile_image"; filename="user_placeholder_icon.png"
Content-Type: image/png

< ./src/tests/test_files/user_placeholder_icon.png
------boundary456--
HTTP/1.1 200
[Asserts]
jsonpath "$.name" == "Hurl Test User Updated"

# ─── Step 5: Delete user ─────────────────────────────────────────────────────
DELETE http://localhost:8080/api/private/v1/users/{{user_id}}
HTTP/1.1 200
