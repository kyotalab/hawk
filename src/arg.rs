use std::path::PathBuf;

use clap::Parser;

use crate::Error;

/// hawk - Modern data analysis tool for structured data (JSON, YAML, CSV)
///
/// hawk combines the simplicity of awk with the power of pandas for data exploration.
/// Perfect for analyzing JSON APIs, YAML configs, and CSV datasets.
#[derive(Debug, Parser)]
#[command(name = "hawk")]
#[command(version = "0.1.0")]
#[command(about = "Modern data analysis tool for structured data")]
#[command(long_about = "
hawk is a command-line data analysis tool that brings pandas-like functionality
to your terminal. It supports JSON, YAML, and CSV formats with automatic detection,
powerful filtering, grouping, and aggregation capabilities.

EXAMPLES:
    # Basic field access
    hawk '.users[0].name' data.json
    hawk '.users.name' data.csv

    # Filtering and aggregation
    hawk '.users[] | select(.age > 30)' data.yaml
    hawk '.sales | group_by(.region) | avg(.amount)' sales.csv

    # Data exploration
    hawk '. | info' data.json
    hawk '.users | count' data.csv

SUPPORTED FORMATS:
    JSON, YAML, CSV (automatically detected)

QUERY SYNTAX:
    .field                    - Access field
    .array[0]                 - Access array element
    .array[]                  - Access all array elements
    . | select(.field > 10)   - Filter data
    . | group_by(.category)   - Group data
    . | count/sum/avg/min/max - Aggregate functions
")]
pub struct Args {
    /// JSONPath-style query to execute
    ///
    /// Examples:
    ///   .users[0].name              - Get first user's name
    ///   .users | select(.age > 30)  - Filter users by age
    ///   . | group_by(.department)   - Group by department
    pub query: String,

    /// Input file path (JSON, YAML, or CSV)
    ///
    /// If not provided, reads from stdin.
    /// File format is automatically detected.
    pub path: Option<PathBuf>,

    /// Output format
    ///
    /// auto: Smart detection (table for arrays, list for values, json for complex)
    /// table: Force tabular output
    /// json: Force JSON output
    /// list: Force list output
    #[arg(long, default_value = "auto")]
    #[arg(value_parser = ["auto", "table", "json", "list"])]
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
