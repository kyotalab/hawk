use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid output format: {0}")]
    InvalidFormat(String),

    #[error("File not found: {0}")]
    FileNotFound(#[from] std::io::Error), // std::io::Error から自動変換

    #[error("JSON deserialization error: {0}")]
    Json(#[from] serde_json::Error), // serde_json::Error から自動変換

    #[error("YAML deserialization error: {0}")]
    Yaml(#[from] serde_yaml::Error), // serde_yaml::Error から自動変換

    #[error("str parse int error: {0}")]
    StrToInt(#[from] std::num::ParseIntError), // std::num::ParseIntError からの自動変換

    #[error("Invalid query format: {0}")]
    InvalidQuery(String),

    #[error("Array index out of bounds: {0}")]
    IndexOutOfBounds(usize),
}
