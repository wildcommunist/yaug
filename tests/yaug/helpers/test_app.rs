use sqlx::{PgConnection, PgPool, Connection, Executor};
use once_cell::sync::Lazy;
use uuid::Uuid;
use yaug::configuration::{DatabaseSettings, get_configuration};
use yaug::startup::Application;
use yaug::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "testing_suite".to_string();

    /*
    If we run our test with TEST_LOG=true we will get test log tracing output as well, otherwise it will be discarded
     */
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    // we need the address for testing, so we know where to send requests to
    pub db_pool: PgPool,
    pub port: u16,
    pub api_client: reqwest::Client,
}

pub async fn spawn_test_app() -> TestApp {
    Lazy::force(&TRACING);

    let settings = {
        let mut config = get_configuration().expect("Failed to load configuration");
        config.db.database = Uuid::new_v4().to_string(); // we need a clean db each time we test
        config.app.port = 0; // Let OS assign a random, free port
        config
    };

    setup_test_database_and_migrate(&settings.db).await;

    let app = Application::build(settings.clone())
        .await
        .expect("Failed to build application");
    let port = app.port();
    let address = format!("http://127.0.0.1:{}", port);

    let _ = tokio::spawn(app.run_until_stopped());
    let api_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none()) // Prevent following those 302 redirects. we need to test em!
        .cookie_store(true)
        .build()
        .unwrap();
    let test_app = TestApp {
        address,
        db_pool: settings.db.get_connection_pool(),
        port,
        api_client,
    };

    test_app
}

async fn setup_test_database_and_migrate(db_settings: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&db_settings.get_conn_str_without_db())
        .await
        .expect("Failed to connect to the database server");

    connection
        .execute(format!(r#"CREATE DATABASE "{}""#, db_settings.database).as_str())
        .await
        .expect("Failed to create test database");

    let connection_pool = PgPool::connect_with(db_settings.get_conn_str_with_db())
        .await
        .expect("Failed to create database connection pool");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run test migrations");

    connection_pool
}