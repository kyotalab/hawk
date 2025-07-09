pub mod arg;
pub mod error;
pub mod executor;
pub mod filter;
pub mod parser;
pub mod utils;

use std::{fs::File, io::{self, BufRead, BufReader}};

pub use arg::*;
use clap::Parser;
pub use error::*;
pub use executor::*;
pub use filter::*;
pub use parser::*;
use serde_json::Value;
pub use utils::*;

pub fn setup() -> Result<(Value, String), Error> {
    let args = Args::parse();
    let reader: Box<dyn BufRead> = if let Some(path) = args.path {
        Box::new(BufReader::new(File::open(path)?))
    } else {
        Box::new(BufReader::new(io::stdin()))
    };

    let json: Value = serde_json::from_reader(reader)?;
    let query = args.query;

    Ok((json, query))
}

