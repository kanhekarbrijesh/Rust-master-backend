# Product Categories: create only endpoint
POST http://127.0.0.1:8080/api/private/v1/product-categories
Content-Type: application/json
{
  "name": "Test Category"
}
HTTP/1.1 201
[Captures]
category_id: jsonpath "$.id"

# Products: full CRUD
POST http://127.0.0.1:8080/api/private/v1/products
Content-Type: application/json
{
  "sku": "XYZ-9876",
  "item_name": "Industrial Test Product",
  "description": "This description is intentionally long to easily satisfy Garde's minimum length constraints for description fields.",
  "quantity": 50,
  "price_in_cents": 2500,
  "discount_percent": 10,
  "supplier_email": "supplier@example.com",
  "tags": ["testingapp"],
  "created_at": "2026-06-08T12:00:00Z"
}
HTTP/1.1 201
[Captures]
product_id: jsonpath "$.id"

GET http://127.0.0.1:8080/api/private/v1/products
HTTP/1.1 200

GET http://127.0.0.1:8080/api/private/v1/products/{{product_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$._id" exists

PUT http://127.0.0.1:8080/api/private/v1/products/{{product_id}}
Content-Type: application/json
{
  "sku": "XYZ-9876",
  "item_name": "Updated Test Product",
  "description": "This description is intentionally long to easily satisfy Garde's minimum length constraints for description fields.",
  "quantity": 55,
  "price_in_cents": 3000,
  "discount_percent": 15,
  "supplier_email": "supplier@example.com",
  "tags": ["testingapp", "updated"],
  "created_at": "2026-06-08T12:00:00Z"
}
HTTP/1.1 200
[Asserts]
jsonpath "$.message" == "Product updated successfully"

DELETE http://127.0.0.1:8080/api/private/v1/products/{{product_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.message" == "Product deleted successfully"

# User Roles: create, get, update
POST http://127.0.0.1:8080/api/private/v1/user-roles
Content-Type: application/json
{
  "role_name": "tester",
  "weight": 10
}
HTTP/1.1 200
[Captures]
role_id: jsonpath "$.id"

GET http://127.0.0.1:8080/api/private/v1/user-roles/{{role_id}}
HTTP/1.1 200
[Asserts]
jsonpath "$.role_name" == "tester"

PUT http://127.0.0.1:8080/api/private/v1/user-roles/{{role_id}}
Content-Type: application/json
{
  "role_name": "tester-updated",
  "weight": 15
}
HTTP/1.1 200
[Asserts]
jsonpath "$.role_name" == "tester-updated"