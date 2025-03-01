use std::process::ExitCode;

use log::{error, info};

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();
    info!("Logger initialized");

    if let Err(err) = dotenvy::dotenv() {
        error!("Error with .env file: {err}");
    }

    if let Err(err) = t01::run().await {
        eprintln!("{err:?}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
