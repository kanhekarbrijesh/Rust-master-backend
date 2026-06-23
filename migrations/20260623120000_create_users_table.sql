-- Add migration script here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    profile_image VARCHAR(2048) NOT NULL DEFAULT '',
    role_id INT NOT NULL REFERENCES user_roles(id) ON DELETE RESTRICT
);
