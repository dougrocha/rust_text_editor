pub mod action;
pub mod buffer;
pub mod cli;
pub mod components;
pub mod config;
pub mod cursor;
pub mod editor;
pub mod mode;
pub mod text;
pub mod tui;
pub mod utils;
pub mod window;

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::Result;

use crate::{
    editor::Editor,
    utils::{initialize_logging, initialize_panic_handler},
};

async fn tokio_main() -> Result<()> {
    initialize_logging()?;

    initialize_panic_handler()?;

    let args = Cli::parse();

    let current_dir = std::env::current_dir()?;

    let mut app = Editor::new(current_dir, args.files)?;
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
