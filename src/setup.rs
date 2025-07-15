use std::io::{self, Read};

use clap::Parser;
use serde_json::Value;

use crate::{Args, Error, OutputFormat};

pub fn setup() -> Result<(Value, String, OutputFormat), Error> {
    let args = Args::parse();

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
    Csv,
    Text,
}

fn detect_input_format(content: &str) -> InputFormat {
    let trimmed = content.trim();

    // CSV判定を最初に行う（シンプルな形式から）
    if is_likely_csv(trimmed) {
        return InputFormat::Csv;
    }

    // JSON判定（厳密にチェック）- YAMLより先に判定
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        // さらに、全体がJSONとして有効かチェック
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return InputFormat::Json;
        }
    }

    // YAML判定 - より厳格な条件に変更
    if is_structured_yaml(trimmed) {
        return InputFormat::Yaml;
    }

    // 上記のいずれにも該当しない場合はText
    InputFormat::Text
}

// 構造化されたYAMLかどうかを厳格に判定
fn is_structured_yaml(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.is_empty() {
        return false;
    }

    // Kubernetes/Docker Compose等の明確なYAMLマーカー
    if content.contains("apiVersion:") || content.contains("kind:") 
       || content.contains("version:") || content.contains("services:") {
        return true;
    }

    let mut yaml_indicators = 0;
    let mut total_meaningful_lines = 0;

    for line in lines {
        let trimmed = line.trim();
        
        // 空行やコメントは除外
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        total_meaningful_lines += 1;

        // YAML構造の特徴を検出
        if is_valid_yaml_line(trimmed) {
            yaml_indicators += 1;
        }
    }

    // 意味のある行が少ない場合はYAMLではない
    if total_meaningful_lines < 3 {
        return false;
    }

    // 80%以上の行がYAML構造ならYAMLと判定
    (yaml_indicators as f64 / total_meaningful_lines as f64) > 0.8
}

// 有効なYAML行かどうかを判定
fn is_valid_yaml_line(line: &str) -> bool {
    // リスト形式 (- item)
    if line.starts_with("- ") {
        return true;
    }

    // key: value 形式
    if let Some(colon_pos) = line.find(':') {
        let key_part = line[..colon_pos].trim();
        let value_part = line[colon_pos + 1..].trim();

        // キー部分の検証
        if key_part.is_empty() {
            return false;
        }

        // キーに無効な文字が含まれていない
        if key_part.contains(' ') && !key_part.starts_with('"') && !key_part.starts_with('\'') {
            return false;
        }

        // インデントされたネスト構造
        if line.starts_with("  ") || line.starts_with("\t") {
            return true;
        }

        // 値が明らかにYAML的
        if value_part.is_empty() 
           || value_part.starts_with('[') 
           || value_part.starts_with('{')
           || value_part == "true" 
           || value_part == "false" 
           || value_part.parse::<f64>().is_ok() {
            return true;
        }

        // パス、URL、タイムスタンプなどが含まれていたらYAMLではない可能性が高い
        if value_part.contains('/') && value_part.len() > 10 {
            return false;
        }

        return true;
    }

    false
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
        InputFormat::Text => parse_text_to_json(content),
    }
}

fn parse_text_to_json(content: &str) -> Result<Value, Error> {
    // テキストを行ごとに分割して配列として扱う
    let lines: Vec<Value> = content
        .lines()
        .map(|line| Value::String(line.to_string()))
        .collect();
    
    // 空のファイルの場合も配列として返す
    Ok(Value::Array(lines))
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

// テキスト処理用のヘルパー関数
pub fn text_to_json_values(content: &str) -> Result<Vec<Value>, Error> {
    let lines: Vec<Value> = content
        .lines()
        .map(|line| Value::String(line.to_string()))
        .collect();
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_parsing() {
        let content = "line1\nline2\nERROR: something happened";
        let result = parse_text_to_json(content).unwrap();
        
        if let Value::Array(lines) = result {
            assert_eq!(lines.len(), 3);
            assert_eq!(lines[0], Value::String("line1".to_string()));
            assert_eq!(lines[1], Value::String("line2".to_string()));
            assert_eq!(lines[2], Value::String("ERROR: something happened".to_string()));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_empty_text() {
        let content = "";
        let result = parse_text_to_json(content).unwrap();
        
        if let Value::Array(lines) = result {
            assert_eq!(lines.len(), 1);
            assert_eq!(lines[0], Value::String("".to_string()));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_format_detection() {
        // JSON
        assert!(matches!(detect_input_format(r#"{"key": "value"}"#), InputFormat::Json));
        assert!(matches!(detect_input_format(r#"[{"name": "Alice"}, {"name": "Bob"}]"#), InputFormat::Json));
        
        // CSV
        assert!(matches!(detect_input_format("name,age\nAlice,30\nBob,25"), InputFormat::Csv));
        
        // YAML - 明確な構造化YAML
        assert!(matches!(detect_input_format("apiVersion: v1\nkind: Pod\nmetadata:\n  name: test"), InputFormat::Yaml));
        assert!(matches!(detect_input_format("version: '3'\nservices:\n  web:\n    image: nginx"), InputFormat::Yaml));
        
        // Text - あらゆる種類のプレーンテキスト
        assert!(matches!(detect_input_format("plain text line\nanother line\nthird line"), InputFormat::Text));
        assert!(matches!(detect_input_format("2024-01-01 10:00:00 INFO Starting application\n2024-01-01 10:00:01 ERROR Something failed"), InputFormat::Text));
        assert!(matches!(detect_input_format("192.168.1.100 - - [15/Jan/2024:09:00:01 +0000] \"GET /api/users HTTP/1.1\" 200 1234"), InputFormat::Text));
        assert!(matches!(detect_input_format("# This is a comment\nSome content\n# Another comment"), InputFormat::Text));
        assert!(matches!(detect_input_format("ServerName localhost\nServerPort 8080\nDocumentRoot /var/www"), InputFormat::Text));
        assert!(matches!(detect_input_format("Random text with: colons but not YAML\nAnother line with strange: formatting"), InputFormat::Text));
    }
    
    #[test]
    fn test_yaml_detection() {
        use super::{is_structured_yaml, is_valid_yaml_line};
        
        // 明確にYAMLとして認識されるべき
        assert!(is_structured_yaml("apiVersion: v1\nkind: Pod"));
        assert!(is_structured_yaml("key: value\nother: data\nnested:\n  sub: item"));
        
        // YAMLとして認識されないべき
        assert!(!is_structured_yaml("2024-01-01 10:00:00 INFO Starting"));
        assert!(!is_structured_yaml("plain text\nwith some: colons"));
        assert!(!is_structured_yaml("ServerName: localhost\nServerPort: 8080")); // 設定ファイル風だがYAMLではない
        
        // 個別行のテスト
        assert!(is_valid_yaml_line("key: value"));
        assert!(is_valid_yaml_line("  nested: item"));
        assert!(is_valid_yaml_line("- list_item"));
        assert!(!is_valid_yaml_line("2024-01-01 10:00:00 INFO message"));
        assert!(!is_valid_yaml_line("random text line"));
    }
}
