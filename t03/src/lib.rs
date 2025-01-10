use anyhow::Result;

mod app;
mod dto;
mod error;
mod model;
mod repository;

pub async fn run() -> Result<()> {
    let app = app::App::initialize().await?;

    app.run().await?;

    Ok(())
}
