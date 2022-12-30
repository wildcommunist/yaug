use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::AccountEmail;

#[tracing::instrument(
name = "Get account by email",
skip(pool)
)]
pub async fn get_account_by_email(
    pool: &PgPool,
    email: &AccountEmail,
) -> Result<Option<Uuid>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT user_id, email, password_hash, created_at
        FROM accounts
        WHERE email = $1
        "#,
        email.as_ref()
    )
        .fetch_optional(pool)
        .await
        .context("Failed to execute account fetch query")?
        .map(|r| r.user_id);
    Ok(row)
}

#[tracing::instrument(
name = "Get account by user id",
skip(pool)
)]
pub async fn get_account_by_user_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<Uuid>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT user_id, email, password_hash, created_at
        FROM accounts
        WHERE user_id = $1
        "#,
        user_id
    )
        .fetch_optional(pool)
        .await
        .context("Failed to execute account fetch query")?
        .map(|r| r.user_id);
    Ok(row)
}