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

    let data = parse_content(&content, input_format)?;
    let query = args.query;

    let format = args
        .format
        .parse::<OutputFormat>()
        .map_err(|e| Error::InvalidFormat(e.to_string()))?;

    // debug
    // debug_json_order(&json);
    Ok((data, query, format))
}

#[derive(Debug)]
enum InputFormat {
    Json,
    Yaml,
    Csv, // 将来追加
}
fn detect_input_format(content: &str) -> InputFormat {
    let trimmed = content.trim();

    // デバッグ情報を出力
    //  println!("=== Format Detection Debug ===");
    //  println!("Content length: {}", content.len());
    //  println!("First 200 chars: {}", trimmed.chars().take(200).collect::<String>());

    // CSV判定を最初に行う（シンプルな形式から）
    if is_likely_csv(trimmed) {
        return InputFormat::Csv;
    }

    // YAML判定を先に行う（より具体的な条件）
    if trimmed.contains("apiVersion:")
        || trimmed.contains("kind:")
        || trimmed.contains("metadata:")
        || trimmed.contains("spec:")
        || (trimmed.contains(":\n") || trimmed.contains(": "))
    {
        // YAML特有のkey: value形式
        return InputFormat::Yaml;
    }

    // JSON判定（厳密にチェック）
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        // さらに、全体がJSONとして有効かチェック
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return InputFormat::Json;
        }
    }

    // 最後の手段: コロンがあればYAML、なければJSON
    if trimmed.contains(':') {
        InputFormat::Yaml
    } else {
        InputFormat::Json
    }
}

fn parse_content(content: &str, format: InputFormat) -> Result<Value, Error> {
    match format {
        InputFormat::Json => serde_json::from_str(content).map_err(Error::Json),
        InputFormat::Yaml => {
            // 複数ドキュメントに対応
            if content.contains("---") {
                parse_multi_document_yaml(content)
            } else {
                serde_yaml::from_str(content).map_err(Error::Yaml)
            }
        }
        InputFormat::Csv => parse_csv_to_json(content),
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

fn is_likely_csv(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().take(5).collect();

    if lines.is_empty() {
        return false;
    }

    // 最初の行をヘッダーとして想定
    let first_line = lines[0];
    let comma_count = first_line.matches(',').count();

    // カンマが1個以上あり、他の行も同じような構造
    if comma_count > 0 {
        // 他の行も同じようなカンマ数か確認
        lines.iter().skip(1).all(|line| {
            let line_comma_count = line.matches(',').count();
            (line_comma_count as i32 - comma_count as i32).abs() <= 1
        })
    } else {
        false
    }
}
fn parse_csv_to_json(content: &str) -> Result<Value, Error> {
    let mut reader = csv::Reader::from_reader(content.as_bytes());

    // ヘッダーを取得
    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| Error::Csv(e))?
        .iter()
        .map(|h| h.trim().to_string())
        .collect();

    let mut records = Vec::new();

    for result in reader.records() {
        let record = result.map_err(|e| Error::Csv(e))?;
        let mut object = serde_json::Map::new();

        for (i, field) in record.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                let value = infer_value_type(field.trim());
                object.insert(header.clone(), value);
            }
        }

        records.push(Value::Object(object));
    }

    // 直接配列を返す（二重配列にしない）
    Ok(Value::Array(records))
}

fn infer_value_type(field: &str) -> Value {
    // 空文字チェック
    if field.is_empty() {
        return Value::Null;
    }

    // 真偽値判定
    match field.to_lowercase().as_str() {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _ => {}
    }

    // 整数判定
    if let Ok(int_val) = field.parse::<i64>() {
        return Value::Number(serde_json::Number::from(int_val));
    }

    // 浮動小数点数判定
    if let Ok(float_val) = field.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(float_val) {
            return Value::Number(num);
        }
    }

    // デフォルトは文字列
    Value::String(field.to_string())
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
