use std::fmt::{Debug, Formatter};
use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use actix_web::web::{Data, Form};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::{AccountCredentials, AccountPassword, AccountEmail};
use crate::helpers::generate_subscription_token;
use crate::store::{get_account_by_email, store_user_account, store_user_activation_email_job, store_user_activation_token};
use crate::utils::{error_chain_fmt, see_other};

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub password: Secret<String>,
    pub password_check: Secret<String>,
}

impl TryFrom<FormData> for AccountCredentials {
    type Error = String;
    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        if value.password_check.expose_secret() != value.password.expose_secret() {
            return Err(format!("Password mismatch"));
        }

        let email = AccountEmail::parse(value.email)?;
        let password = AccountPassword::parse(value.password)?;
        Ok(AccountCredentials {
            email,
            password,
        })
    }
}

#[derive(thiserror::Error)]
pub enum RegistrationError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl Debug for RegistrationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for RegistrationError {
    fn status_code(&self) -> StatusCode {
        match self {
            RegistrationError::ValidationError(_) => StatusCode::BAD_REQUEST,
            RegistrationError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[tracing::instrument(
name = "Account registration",
skip(data, pool),
fields(
account_email = % data.email
)
)]
pub async fn post_register(
    data: Form<FormData>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, RegistrationError> {
    let new_account: AccountCredentials = data.0.try_into().map_err(RegistrationError::ValidationError)?;
    if let Some(_u) = get_account_by_email(&pool, &new_account.email)
        .await
        .map_err(RegistrationError::UnexpectedError)?
    {
        // we already have a user that is registered
        FlashMessage::error("This email address is already registered").send();
        return Ok(see_other("/register"));
    }

    let mut tx = pool.begin()
        .await
        .context("Failed to get pool transaction lock")?;

    // insert user into accounts
    let account_id =
        store_user_account(
            &mut tx,
            Uuid::new_v4(),
            &new_account.email,
            new_account.password.compute_hash().await?,
        ).await.context("Failed to store new user account")?;

    // store activation token
    let token = generate_subscription_token(32);

    // store job to send activation email
    store_user_activation_token(&mut tx, account_id, &token)
        .await
        .context("Failed to store account activation token")?;

    // queue up activation email job
    store_user_activation_email_job(&mut tx, &new_account.email, "") //TODO: content
        .await
        .context("Failed to save activation email job")?;

    tx.commit()
        .await
        .context("Failed to commit account creation transaction")?;

    FlashMessage::success(
        "Your account has been created, please check you email for further instructions."
    ).send();
    Ok(see_other("/login"))
}
