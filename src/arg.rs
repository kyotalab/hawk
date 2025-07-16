use std::path::PathBuf;

use clap::Parser;

use crate::Error;

/// hawk - Modern data analysis tool for structured data (JSON, YAML, CSV)
///
/// hawk combines the simplicity of awk with the power of pandas for data exploration.
/// Perfect for analyzing JSON APIs, YAML configs, and CSV datasets.
#[derive(Debug, Parser)]
#[command(name = "hawk")]
#[command(version = "0.2.0")]
#[command(about = "Modern data analysis tool for structured data and text files")]
#[command(long_about = "
hawk is a command-line data analysis tool that brings pandas-like functionality
to your terminal. It supports JSON, YAML, CSV, and plain text formats with automatic
detection, powerful filtering, grouping, aggregation, and string manipulation capabilities.

EXAMPLES:
# Basic field access
    hawk ‘.users[0].name’ data.json
    hawk ‘.users.name’ data.csv


# Text processing (NEW in v0.2.0!)
    hawk '. | select(. | contains(\"ERROR\"))' app.log
    hawk '. | map(. | trim | upper)' data.txt
    hawk '. | map(. | substring(0, 19))' access.log

# String operations
    hawk '. | map(. | replace(\"old\", \"new\"))' text.txt
    hawk '. | map(. | split(\",\") | join(\" | \"))' csv_lines.txt

# Filtering and aggregation
    hawk '.users[] | select(.age > 30)' data.yaml
    hawk '.sales | group_by(.region) | avg(.amount)' sales.csv

# Statistical analysis (NEW!)
    hawk '. | unique | sort' numbers.txt
    hawk '.scores[] | median(.value)' scores.json
    hawk '.data[] | stddev(.measurement)' sensor_data.csv

# Complex pipelines
    hawk '. | select(. | contains(\"WARN\")) | map(. | substring(11, 8)) | unique' app.log
    hawk '.users[] | map(.email | lower | trim) | select(. | ends_with(\".com\"))' users.csv

# Data exploration
    hawk '. | info' data.json
    hawk '.users | count' data.csv
    hawk '. | length' any_file.txt


SUPPORTED FORMATS:
    JSON, YAML, CSV, Plain Text (automatically detected)

QUERY SYNTAX:
    # Field Access
    .field                    - Access field
    .array[0]                 - Access array element
    .array[]                  - Access all array elements


# Text Processing (NEW!)
    . | map(. | upper)        - Convert to uppercase
    . | map(. | lower)        - Convert to lowercase
    . | map(. | trim)         - Remove whitespace
    . | map(. | length)       - Get string length
    . | map(. | reverse)      - Reverse string

# String Manipulation
    . | map(. | replace(\"a\", \"b\"))  - Replace text
    . | map(. | substring(0, 5))      - Extract substring
    . | map(. | split(\",\"))          - Split by delimiter
    .array[] | join(\", \")            - Join array elements

# String Filtering
    . | select(. | contains(\"text\"))     - Contains pattern
    . | select(. | starts_with(\"pre\"))   - Starts with pattern
    . | select(. | ends_with(\"suf\"))     - Ends with pattern

# Statistical Functions (NEW!)
    . | unique                - Remove duplicates
    . | sort                  - Sort values
    . | median                - Calculate median
    . | stddev                - Calculate standard deviation
    . | length                - Get array/text length

# Filtering & Aggregation
    . | select(.field > 10)   - Filter data
    . | group_by(.category)   - Group data
    . | count/sum/avg/min/max - Aggregate functions

# Data Transformation
    . | map(.field | operation) - Transform data with string operations


OUTPUT FORMATS:
    –format table           - Colored table output (default for structured data)
    –format json            - JSON output with syntax highlighting
    –format list            - Simple list output
    –format auto            - Smart format detection (default)

COLORED OUTPUT:
    Automatic color detection (TTY), respects NO_COLOR environment variable
")]

pub struct Args {
    /// JSONPath-style query to execute
    ///
    /// Examples:
    ///
    ///   .users[0].name              - Get first user's name
    ///   
    ///   .users | select(.age > 30)  - Filter users by age
    ///   
    ///   . | group_by(.department)   - Group by department
    pub query: String,

    /// Input file path (JSON, YAML, or CSV)
    ///
    /// If not provided, reads from stdin.
    /// File format is automatically detected.
    pub path: Option<PathBuf>,

    /// Output format
    ///
    ///    auto: Smart detection (table for arrays, list for values, json for complex)
    ///
    ///    table: Force tabular output
    /// 
    ///    json: Force JSON output
    /// 
    ///    list: Force list output
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
