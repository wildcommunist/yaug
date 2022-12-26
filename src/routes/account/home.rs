use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web::web::Data;
use tera::{Context, Tera};
use crate::utils::e500;

pub async fn get_account_home(
    tpl: Data<Tera>
) -> Result<HttpResponse, actix_web::Error> {
    let mut ctx = Context::new();

    Ok(
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(
                tpl.render("account/home.html", &ctx).map_err(e500)?
            )
    )
}