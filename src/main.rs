use std::{fs::File, io::{self, BufRead, BufReader}};

use anyhow::Result;
use clap::Parser;
use hawk::{execute_query, Args, Error};
use serde_json::Value;

fn main() -> Result<(), Error>{
    // Load CLI arguments and file
    let (json, query) = setup()?;

    let result = execute_query(&json, &query)?;

    println!("{:?}", result);

    Ok(())
}


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




