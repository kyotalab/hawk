pub mod arg;
pub mod error;
pub mod executor;
pub mod filter;
pub mod parser;
pub mod utils;

use std::io::{self, Read};

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
   //  let reader: Box<dyn BufRead> = if let Some(path) = args.path {
   //      Box::new(BufReader::new(File::open(path)?))
   //  } else {
   //      Box::new(BufReader::new(io::stdin()))
   //  };

    let content = if let Some(path) = args.path {
        std::fs::read_to_string(path)?
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    let input_format = detect_input_format(&content);

    let data= parse_content(&content, input_format)?;
    let query = args.query;

    let format = args.format.parse::<OutputFormat>()
        .map_err(|e| Error::InvalidFormat(e.to_string()))?;


    // debug
    // debug_json_order(&json);
    Ok((data, query, format))
}

#[derive(Debug)]
enum InputFormat {
    Json,
    Yaml,
    // Csv, // 将来追加
}
fn detect_input_format(content: &str) -> InputFormat {
    let trimmed = content.trim();

    // デバッグ情報を出力
   //  println!("=== Format Detection Debug ===");
   //  println!("Content length: {}", content.len());
   //  println!("First 200 chars: {}", trimmed.chars().take(200).collect::<String>());

    // YAML判定を先に行う（より具体的な条件）
    if trimmed.contains("apiVersion:") ||
        trimmed.contains("kind:") ||
        trimmed.contains("metadata:") ||
        trimmed.contains("spec:") ||
        (trimmed.contains(":\n") || trimmed.contains(": ")) {  // YAML特有のkey: value形式
        return InputFormat::Yaml;
    }

    // JSON判定（厳密にチェック）
    if (trimmed.starts_with('{') && trimmed.ends_with('}')) ||
        (trimmed.starts_with('[') && trimmed.ends_with(']')) {
        // さらに、全体がJSONとして有効かチェック
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return InputFormat::Json;
        }
    }

    // 最後の手段: コロンがあればYAML、なければJSON
    if trimmed.contains(':') {
        println!("Detected format: YAML (fallback)");
        InputFormat::Yaml
    } else {
        println!("Detected format: JSON (default)");
        InputFormat::Json
    }
}

fn parse_content(content: &str, format: InputFormat) -> Result<Value, Error> {
    match format {
        InputFormat::Json => {
            serde_json::from_str(content).map_err(Error::Json)
        }
        InputFormat::Yaml => {
            // 複数ドキュメントに対応
            if content.contains("---") {
                parse_multi_document_yaml(content)
            } else {
                serde_yaml::from_str(content).map_err(Error::Yaml)
            }
        }
    }
}

fn parse_multi_document_yaml(content: &str) -> Result<Value, Error> {
    let documents: Vec<&str> = content
        .split("---")
        .map(|doc| doc.trim())
        .filter(|doc| !doc.is_empty())
        .collect();

    let mut parsed_docs = Vec::new();

    for doc in documents {
        let parsed: Value = serde_yaml::from_str(doc).map_err(Error::Yaml)?;
        parsed_docs.push(parsed);
    }

    // 複数ドキュメントを配列として返す
    Ok(Value::Array(parsed_docs))
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
