-- Add migration script here
ALTER TABLE user_roles ADD COLUMN weight INT NOT NULL DEFAULT 0;