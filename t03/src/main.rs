use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = dotenvy::dotenv() {
        eprintln!(".env file: {err}");
        return ExitCode::FAILURE;
    }

    if let Err(err) = t03::run().await {
        eprintln!("{err:?}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
