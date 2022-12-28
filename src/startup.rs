use actix_web::dev::Server;
use std::net::TcpListener;
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_web::{App, HttpServer, web};
use actix_web::cookie::Key;
use actix_web::web::Data;
use actix_web_flash_messages::FlashMessagesFramework;
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_lab::middleware::from_fn;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use tera::Tera;
use tracing_actix_web::TracingLogger;
use crate::authentication::reject_anonymous_users;
use crate::configuration::Settings;
use crate::routes::{get_account_home, get_home_page, get_login_form, post_login, get_register_form, post_register};

//region Application & impl
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, anyhow::Error> {
        let listen_address = format!(
            "{}:{}",
            config.app.host, config.app.port
        );

        let tcp_listener = TcpListener::bind(&listen_address)?;
        let local_port = tcp_listener.local_addr().unwrap().port(); // We are needing this for testing suites where ports are dynamic

        let mut tera = config.get_template_engine();
        tera.autoescape_on(vec![]);

        let server = run(
            tcp_listener,
            config.db.get_connection_pool(),
            config.app.redis_uri,
            config.app.cookie_secret,
            tera,
        ).await?;

        Ok(Self { port: local_port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
//endregion

pub async fn run(
    listener: TcpListener,
    pool: PgPool,
    redis_uri: Secret<String>,
    cookie_secret: Secret<String>,
    template_engine: Tera,
) -> Result<Server, anyhow::Error> {
    let connection_pool = Data::new(pool);
    let template = Data::new(template_engine);
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret())
        .await?;
    let secret_key = Key::from(cookie_secret.expose_secret().as_bytes());
    let message_store = CookieMessageStore::builder(secret_key.clone())
        .build();
    let message_framework = FlashMessagesFramework::builder(
        message_store
    ).build();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(message_framework.clone())
            .wrap(
                SessionMiddleware::new(
                    redis_store.clone(),
                    secret_key.clone(),
                )
            )
            .service(
                actix_files::Files::new("/static", "public/static")
                    .use_last_modified(true)
            )//Maybe move this all to static domain in teh future?
            .route("/", web::get().to(get_home_page))
            .route("/login", web::get().to(get_login_form))
            .route("/login", web::post().to(post_login))
            .route("/register", web::get().to(get_register_form))
            .route("/register", web::post().to(post_register))
            .service(
                // Logged in routes
                web::scope("")
                    .wrap(from_fn(reject_anonymous_users))
                    .route("/account", web::get().to(get_account_home))
            )
            .app_data(connection_pool.clone())
            .app_data(template.clone())
    })
        .listen(listener)?
        .run();
    Ok(server)
}