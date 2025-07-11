use std::path::PathBuf;

use clap::Parser;

use crate::Error;

#[derive(Debug, Parser)]
pub struct Args {
    pub query: String,
    pub path: Option<PathBuf>,

    /// Format output (table, json, csv)
    #[arg(long, default_value = "auto")]
    pub format: String,
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Auto,
    Json,
    Table,
    List,
    // Csv, // 将来追加
}

impl std::str::FromStr for OutputFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(OutputFormat::Auto),
            "json" => Ok(OutputFormat::Json),
            "table" => Ok(OutputFormat::Table),
            "list" => Ok(OutputFormat::List),
            // "csv" => Ok(OutputFormat::Csv),
            _ => Err(Error::InvalidFormat(format!(
                "Invalid format: {}. Valid options: auto, json, table, list",
                s
            ))),
        }
    }
}
