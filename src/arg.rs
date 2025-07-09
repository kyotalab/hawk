use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    pub query: String,
    pub path: Option<PathBuf>,
}
