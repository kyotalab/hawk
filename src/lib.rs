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

pub fn setup() -> Result<(Value, String, OutputFormat), Error> {
    let args = Args::parse();
    let reader: Box<dyn BufRead> = if let Some(path) = args.path {
        Box::new(BufReader::new(File::open(path)?))
    } else {
        Box::new(BufReader::new(io::stdin()))
    };

    let json = serde_json::from_reader(reader)?;
    let query = args.query;

    let format = args.format.parse::<OutputFormat>()
        .map_err(|e| Error::InvalidFormat(e.to_string()))?;


    // debug
    // debug_json_order(&json);
    Ok((json, query, format))
}

pub fn debug_json_order(json: &Value) {
    println!("=== Original JSON field order ===");
    
    // ルートレベル
    if let Value::Object(obj) = json {
        println!("Root fields:");
        for key in obj.keys() {
            println!("  {}", key);
        }
        
        // users配列の最初の要素のフィールド順序
        if let Some(Value::Array(users)) = obj.get("users") {
            if let Some(Value::Object(first_user)) = users.get(0) {
                println!("First user fields:");
                for key in first_user.keys() {
                    println!("  {}", key);
                }
            }
        }
    }
}
