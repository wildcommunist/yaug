use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web::web::Data;
use tera::{Context, Tera};
use crate::utils::e500;

pub async fn get_home_page(
    tpl: Data<Tera>,
) -> Result<HttpResponse, actix_web::Error> {
    let ctx = Context::new();

    Ok(
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(
                tpl.render("home.html", &ctx).map_err(e500)?
            )
    )
}