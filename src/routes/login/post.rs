use actix_web::error::InternalError;
use actix_web::HttpResponse;
use actix_web::web::{Data, Form};
use secrecy::Secret;
use sqlx::PgPool;
use crate::authentication::YaugSession;

#[derive(Deserialize)]
struct LoginFormData {
    pub username: String,
    pub password: Secret<String>,
}

pub enum LoginError{

}

#[tracing::instrument(
    name="Post login",
    skip(data,pool,sesion),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn post_login(
    data: Form<LoginFormData>,
    pool: Data<PgPool>,
    session: YaugSession,
) -> Result<HttpResponse, InternalError<LoginError>> {}