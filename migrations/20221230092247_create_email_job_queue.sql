-- 20221230092247_create_email_job_queue.sql
CREATE TABLE email_queue
(
    id         uuid PRIMARY KEY,
    email      varchar(32) NOT NULL UNIQUE,
    content    TEXT        NOT NULL,
    created_at timestamptz NOT NULL
);
