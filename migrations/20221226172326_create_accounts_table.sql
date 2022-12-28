-- 20221226172326_create_accounts_table.sql
CREATE TABLE accounts
(
    user_id       uuid PRIMARY KEY,
    email         TEXT        NOT NULL UNIQUE,
    password_hash TEXT        NOT NULL,
    created_at    timestamptz NOT NULL
);