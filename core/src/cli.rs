use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version = "", about)]
pub struct Args {
    #[clap(name = "file")]
    pub files: Vec<PathBuf>,
}
