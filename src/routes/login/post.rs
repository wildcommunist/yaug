use std::fmt::Formatter;
use actix_web::error::InternalError;
use actix_web::{HttpResponse, ResponseError};
use actix_web::http::header::LOCATION;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Form};
use actix_web_flash_messages::FlashMessage;
use secrecy::{Secret};
use serde::Deserialize;
use sqlx::PgPool;
use crate::authentication::{AuthenticationError, Credentials, validate_login_credentials, YaugSession};
use crate::utils::{error_chain_fmt, see_other};

#[derive(Deserialize)]
pub struct LoginFormData {
    pub email: String,
    pub password: Secret<String>,
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
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credentials = Credentials {
        email: data.0.email,
        password: data.0.password,
    };
    tracing::Span::current().record("email", &tracing::field::display(&credentials.email));

    match validate_login_credentials(credentials, &pool).await {
        Ok(user_id) => {
            session.renew();
            session.insert_user_id(user_id)
                .map_err(|e| login_redirect(LoginError::UnexpectedError(e.into())))?;
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            Ok(see_other("/account"))
        }
        Err(e) => {
            let e = match e {
                AuthenticationError::InvalidCredentials(_) => LoginError::InvalidCredentials(e.into()),
                AuthenticationError::UnexpectedError(_) => LoginError::UnexpectedError(e.into())
            };
            Err(login_redirect(e))
        }
    }
}

fn login_redirect(e: LoginError) -> InternalError<LoginError> {
    FlashMessage::error(e.to_string()).send();
    let response = HttpResponse::SeeOther()
        .insert_header((LOCATION, "/login"))
        .finish();
    InternalError::from_response(e, response)
}