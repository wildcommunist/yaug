use secrecy::{ExposeSecret, Secret};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;
use crate::domain::AccountEmail;

#[tracing::instrument(
name = "Store user account",
skip(tx, hash)
)]
pub async fn store_user_account(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    email: &AccountEmail,
    hash: Secret<String>,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO accounts (user_id, email, password_hash, created_at)
        VALUES ($1, $2, $3, now())
        "#,
        user_id, email.as_ref(), hash.expose_secret()
    ).execute(tx)
        .await?;
    Ok(user_id)
}

#[tracing::instrument(
name = "Store user activation token",
skip(tx, token)
)]
pub async fn store_user_activation_token(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    token: &str,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO activation_token (user_id, token, created_at)
        VALUES ($1, $2, now())
        "#,
        user_id, token
    ).execute(tx)
        .await?;
    Ok(user_id)
}

#[tracing::instrument(
name = "Store user activation email",
skip(tx, content)
)]
pub async fn store_user_activation_email_job(
    tx: &mut Transaction<'_, Postgres>,
    recipient: &AccountEmail,
    content: &str,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO email_queue (id, email, content, created_at)
        VALUES ($1, $2, $3, now())
        "#,
        id, recipient.as_ref(), content
    ).execute(tx).await?;
    Ok(id)
}