mod auth;
mod config;
mod db;
mod frontend;
mod http;
mod templates;

use anyhow::Context;
use clap::Parser;
use log::info;
use sqlx::postgres::PgPoolOptions;

use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // This returns an error if the `.env` file doesn't exist, but that's not what we want
    // since we're not going to use a `.env` file if we deploy this application.
    dotenv::dotenv().ok();

    // Initialize the logger.
    env_logger::init();

    let config = Config::parse();

    // We create a single connection pool for SQLx that's shared across the whole application.
    // This saves us from opening a new connection for every API call, which is wasteful.
    let db = PgPoolOptions::new()
        .connect(&config.database_url)
        .await
        .context("could not connect to database_url")?;

    // This embeds database migrations in the application binary so we can ensure the database
    // is migrated correctly on startup
    info!("Applying database migrations...");
    sqlx::migrate!().run(&db).await?;
    info!("All un-applied migrations have been successfully executed!");

    http::serve(config, db).await?;

    Ok(())
}
