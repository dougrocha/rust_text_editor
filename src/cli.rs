use std::path::PathBuf;

use clap::Parser;

use crate::utils::version;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Args {
    #[clap(name = "file")]
    pub files: Vec<PathBuf>,
}
