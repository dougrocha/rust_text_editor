pub mod app;
pub mod buffer;
pub mod cli;
pub mod components;
pub mod config;
pub mod cursor;
pub mod editor;
pub mod mode;
pub mod movements;
pub mod prompt;
pub mod terminal;
pub mod utils;
pub mod window;

use app::App;
use clap::Parser;
use cli::Args;
use color_eyre::eyre::Result;
use utils::{setup_logging, setup_panic_handler};

async fn tokio_main() -> Result<()> {
    setup_logging()?;
    setup_panic_handler()?;

    let args = Args::parse();

    let mut app = App::new(args)?;
    app.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = tokio_main().await {
        eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
        Err(e)
    } else {
        Ok(())
    }
}
