use anyhow::{anyhow, Context};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use uuid::Uuid;
use crate::telemetry::spawn_blocking_with_tracing;

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthenticationError {
    #[error("Invalid credentials")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[tracing::instrument(
name = "Validate credentials",
skip(credentials, pool)
)]
pub async fn validate_login_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<Uuid, AuthenticationError> {
    let mut user_id = None;
    // The reason we set some random password hash is so that we dont get a user enumeration attack
    // so that we dont have few ms diff between existing user password match and no user
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$gZiV/M1gPc22ElAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno".to_string(),
    );

    if let Some((stored_user_id, stored_password_hash)) = get_stored_credentials(
        pool, &credentials.username,
    ).await.map_err(AuthenticationError::UnexpectedError)?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    // We are spawning comparison in another thread as not to block the main thread executor
    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
        .await
        .context("Failed to spawn password validation thread")
        .map_err(AuthenticationError::UnexpectedError)??;

    user_id.ok_or_else(|| AuthenticationError::InvalidCredentials(anyhow!("Invalid username or password.")))
}

#[tracing::instrument(
name = "get store credentials",
skip(pool, email)
)]
pub async fn get_stored_credentials(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(Uuid, Secret<String>)>, anyhow::Error> {
    let row = sqlx::query!(
       r#"
       SELECT user_id, password_hash
       FROM accounts
       WHERE email = $1
       "#,
       email
   ).fetch_optional(pool)
        .await
        .context("Failed to perform query to get stored credentials")?
        .map(|r| (r.user_id, Secret::new(r.password_hash)));

    Ok(row)
}

#[tracing::instrument(
name = "Verify password hash",
skip(expected, given)
)]
fn verify_password_hash(
    expected: Secret<String>,
    given: Secret<String>,
) -> Result<(), AuthenticationError> {
    let expected_hash = PasswordHash::new(expected.expose_secret())
        .context("Failed to parse given password as PHC string")
        .map_err(AuthenticationError::UnexpectedError)?;

    Argon2::default()
        .verify_password(
            given.expose_secret().as_bytes(),
            &expected_hash,
        )
        .context("Invalid password")
        .map_err(AuthenticationError::InvalidCredentials)
}