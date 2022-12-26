-- 20221226172326_create_accounts_table.sql
CREATE TABLE accounts
(
    account_id uuid PRIMARY KEY,
    password   TEXT        NOT NULL,
    created_at timestamptz NOT NULL
);