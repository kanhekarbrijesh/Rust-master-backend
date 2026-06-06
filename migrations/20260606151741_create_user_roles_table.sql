-- Add migration script here
CREATE TABLE user_roles (
    id SERIAL PRIMARY KEY,
    role_name VARCHAR(255) NOT NULL
);