# Orders E2E Hurl workflow

# Create order
POST http://localhost:3000/private/v1/orders
Content-Type: application/json

{
  "order_number": "ORD-1001",
  "user_id": "user123",
  "items": [{"sku": "SKU1", "quantity": 2, "price_in_cents": 500}],
  "total_in_cents": 1000,
  "status": "new",
  "created_at": "2026-01-01T00:00:00Z"
}

> {"id": "$id"}

# Get all orders
GET http://localhost:3000/private/v1/orders

> [200]

# Get created order by id
GET http://localhost:3000/private/v1/orders/{{id}}

> [200]

# Update order
PUT http://localhost:3000/private/v1/orders/{{id}}
Content-Type: application/json

{
  "order_number": "ORD-1001",
  "user_id": "user123",
  "items": [{"sku": "SKU1", "quantity": 3, "price_in_cents": 500}],
  "total_in_cents": 1500,
  "status": "updated",
  "created_at": "2026-01-01T00:00:00Z"
}

> [200]

# Delete order
DELETE http://localhost:3000/private/v1/orders/{{id}}

> [200]
