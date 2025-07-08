use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("JSON deserialization error: {0}")]
    Json(#[from] serde_json::Error), // serde_json::Error から自動変換
}
