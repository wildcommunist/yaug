use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use tera::{Context, Tera};
use crate::utils::e500;

pub async fn get_login_form(
    flash_messages: IncomingFlashMessages,
    tpl: Data<Tera>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut flash: Vec<String> = Vec::new();

    //for m in flash_messages.iter().filter(|m| m.level() == Level::Error) {
    // TODO: Sort by level (info warning error etc)
    for fm in flash_messages.iter() {
        flash.push(format!("<p><i>{}</i></p>", fm.content()))
    }

    flash.push(format!("<p><b>{}</b></p><br />It HURT!", "I got hit by a bus!"));

    let mut ctx = Context::new();
    ctx.insert("flash", &flash);

    Ok(
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(
                tpl.render("login/form.html", &ctx).map_err(e500)?
            )
    )
}