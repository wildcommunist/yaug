use std::fmt::Formatter;
use actix_web::error::InternalError;
use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use actix_web::web::{Data, Form};
use secrecy::{Secret};
use serde::Deserialize;
use sqlx::PgPool;
use crate::authentication::{YaugSession};
use crate::domain::{LoginCredentials, LoginEmail, LoginPassword};
use crate::utils::error_chain_fmt;

#[derive(Deserialize)]
pub struct LoginFormData {
    pub email: String,
    pub password: Secret<String>,
}

impl TryFrom<LoginFormData> for LoginCredentials {
    type Error = String;
    fn try_from(value: LoginFormData) -> Result<Self, Self::Error> {
        let email = LoginEmail::parse(value.email)?;
        let password = LoginPassword::parse(value.password)?;
        Ok(LoginCredentials {
            email,
            password,
        })
    }
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("{0}")]
    ValidationError(String),
    #[error("Authentication failed")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error("Account not activated")]
    AccountNotActivated(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[source] anyhow::Error),
}

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            LoginError::ValidationError(_) => StatusCode::BAD_REQUEST,
            LoginError::InvalidCredentials(_) => StatusCode::UNAUTHORIZED,
            LoginError::AccountNotActivated(_) => StatusCode::UNAUTHORIZED,
            LoginError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(&self, f)
    }
}

#[tracing::instrument(
name = "Post login",
skip(data, pool, session),
fields(email = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub async fn post_login(
    data: Form<LoginFormData>,
    pool: Data<PgPool>,
    session: YaugSession,
) -> Result<HttpResponse, LoginError> {
    let login_credentials: LoginCredentials = data.0.try_into().map_err(LoginError::ValidationError)?;

    tracing::Span::current().record("email", &tracing::field::display(&login_credentials.email));


    todo!()
}