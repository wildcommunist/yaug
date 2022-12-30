-- 20221230085347_create_account_activation_table.sql
CREATE TABLE activation_token
(
    user_id    uuid        NOT NULL REFERENCES accounts (user_id),
    token      varchar(32) NOT NULL UNIQUE,
    created_at timestamptz NOT NULL,
    PRIMARY KEY (token)
);