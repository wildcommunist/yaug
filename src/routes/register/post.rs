use actix_web::HttpResponse;
use actix_web::web::{Data, Form};
use secrecy::Secret;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub password: Secret<String>,
    pub password_check: Secret<String>,
}

pub async fn post_register(
    data: Form<FormData>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    todo!()
}