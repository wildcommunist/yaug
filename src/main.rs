use std::fmt::{Debug, Display};
use tokio::task::JoinError;
use yaug::configuration::{get_configuration, Settings};
use yaug::startup::Application;
use yaug::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("yaug".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    tracing::info!("Fetching configuration");
    let settings = get_configuration().expect("Failed to get application configuration");
    tracing::info!("Building application");
    let app = Application::build(settings.clone()).await?;
    tracing::info!("Spawning threads");
    let app_task = tokio::spawn(app.run_until_stopped());

    tracing::info!(
        "Starting application on {}:{}. Environment: {}",
        &settings.app.host, &settings.app.port, Settings::current_environment().as_str()
    );

    tokio::select! {
        o = app_task => report_exit("API", o)
    }

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed to complete",
                task_name
            )
        }
    }
}