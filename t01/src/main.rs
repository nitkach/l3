use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = t01::run().await {
        eprintln!("{err:?}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
