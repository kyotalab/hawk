use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid output format: {0}")]
    InvalidFormat(String),

    #[error("File not found: {0}")]
    FileNotFound(#[from] std::io::Error),

    #[error("JSON deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML deserialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),

    #[error("str parse int error: {0}")]
    StrToInt(#[from] std::num::ParseIntError),

    #[error("Invalid query format: {0}")]
    InvalidQuery(String),

    #[error("Array index out of bounds: {0}")]
    IndexOutOfBounds(usize),
}
