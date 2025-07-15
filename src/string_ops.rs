use serde_json::Value;
use crate::Error;

/// 文字列操作を適用する
pub fn apply_string_operation(value: &Value, operation: &str) -> Result<Value, Error> {
    match operation {
        "upper" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::String(string_val.to_uppercase()))
        },
        "lower" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::String(string_val.to_lowercase()))
        },
        "trim" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::String(string_val.trim().to_string()))
        },
        "trim_start" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::String(string_val.trim_start().to_string()))
        },
        "trim_end" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::String(string_val.trim_end().to_string()))
        },
        "length" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::Number(serde_json::Number::from(string_val.chars().count())))
        },
        "reverse" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::String(string_val.chars().rev().collect()))
        },
        op if op.starts_with("contains(") && op.ends_with(")") => {
            let string_val = extract_string_value(value)?;
            let pattern = extract_string_argument(op)?;
            Ok(Value::Bool(string_val.contains(&pattern)))
        },
        op if op.starts_with("starts_with(") && op.ends_with(")") => {
            let string_val = extract_string_value(value)?;
            let pattern = extract_string_argument(op)?;
            Ok(Value::Bool(string_val.starts_with(&pattern)))
        },
        op if op.starts_with("ends_with(") && op.ends_with(")") => {
            let string_val = extract_string_value(value)?;
            let pattern = extract_string_argument(op)?;
            Ok(Value::Bool(string_val.ends_with(&pattern)))
        },
        op if op.starts_with("replace(") && op.ends_with(")") => {
            let string_val = extract_string_value(value)?;
            let (old, new) = extract_replace_arguments(op)?;
            Ok(Value::String(string_val.replace(&old, &new)))
        },
        op if op.starts_with("substring(") && op.ends_with(")") => {
            let string_val = extract_string_value(value)?;
            let (start, length) = extract_substring_arguments(op)?;
            let result = extract_substring(string_val, start, length)?;
            Ok(Value::String(result))
        },
        op if op.starts_with("split(") && op.ends_with(")") => {
            let string_val = extract_string_value(value)?;
            let delimiter = extract_string_argument(op)?;
            let parts: Vec<Value> = string_val
                .split(&delimiter)
                .map(|s| Value::String(s.to_string()))
                .collect();
            Ok(Value::Array(parts))
        },
        op if op.starts_with("join(") && op.ends_with(")") => {
            // join操作は配列に対して適用
            apply_join_operation(value, op)
        },
        _ => Err(Error::StringOperation(format!("Unknown string operation: {}", operation))),
    }
}

/// 文字列値を抽出（エラーハンドリングを統一）
fn extract_string_value(value: &Value) -> Result<&str, Error> {
    match value {
        Value::String(s) => Ok(s),
        _ => Err(Error::StringOperation(
            format!("String operations can only be applied to string values, got: {}", get_type_name(value))
        )),
    }
}

/// join操作を配列に適用
fn apply_join_operation(value: &Value, operation: &str) -> Result<Value, Error> {
    if let Value::Array(arr) = value {
        let delimiter = extract_string_argument(operation)?;
        
        let string_parts: Result<Vec<String>, Error> = arr
            .iter()
            .map(|v| match v {
                Value::String(s) => Ok(s.clone()),
                Value::Number(n) => Ok(n.to_string()),
                Value::Bool(b) => Ok(b.to_string()),
                Value::Null => Ok("null".to_string()),
                _ => Err(Error::StringOperation("Cannot join non-primitive values".to_string())),
            })
            .collect();
        
        let parts = string_parts?;
        Ok(Value::String(parts.join(&delimiter)))
    } else {
        Err(Error::StringOperation("join can only be applied to arrays".to_string()))
    }
}

/// 値の型名を取得
fn get_type_name(value: &Value) -> &'static str {
    match value {
        Value::String(_) => "string",
        Value::Number(_) => "number",
        Value::Bool(_) => "boolean",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::Null => "null",
    }
}

/// 文字列引数を抽出（例: contains("pattern") → "pattern"）
fn extract_string_argument(operation: &str) -> Result<String, Error> {
    let start_pos = operation.find('(').ok_or_else(|| {
        Error::StringOperation("Missing opening parenthesis".to_string())
    })? + 1;
    let end_pos = operation.rfind(')').ok_or_else(|| {
        Error::StringOperation("Missing closing parenthesis".to_string())
    })?;
    
    if start_pos >= end_pos {
        return Err(Error::StringOperation("Invalid argument format".to_string()));
    }
    
    let arg = &operation[start_pos..end_pos];
    
    // 引用符を除去
    let cleaned = if (arg.starts_with('"') && arg.ends_with('"')) ||
                     (arg.starts_with('\'') && arg.ends_with('\'')) {
        &arg[1..arg.len()-1]
    } else {
        arg
    };
    
    Ok(cleaned.to_string())
}

/// replace引数を抽出（例: replace("old", "new") → ("old", "new")）
fn extract_replace_arguments(operation: &str) -> Result<(String, String), Error> {
    let start_pos = operation.find('(').ok_or_else(|| {
        Error::StringOperation("Missing opening parenthesis".to_string())
    })? + 1;
    let end_pos = operation.rfind(')').ok_or_else(|| {
        Error::StringOperation("Missing closing parenthesis".to_string())
    })?;
    
    let args_str = &operation[start_pos..end_pos];
    
    // カンマで分割
    let parts: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err(Error::StringOperation("replace requires exactly 2 arguments".to_string()));
    }
    
    let old = clean_string_argument(parts[0])?;
    let new = clean_string_argument(parts[1])?;
    
    Ok((old, new))
}

/// substring引数を抽出（例: substring(0, 5) → (0, 5)）
fn extract_substring_arguments(operation: &str) -> Result<(usize, Option<usize>), Error> {
    let start_pos = operation.find('(').ok_or_else(|| {
        Error::StringOperation("Missing opening parenthesis".to_string())
    })? + 1;
    let end_pos = operation.rfind(')').ok_or_else(|| {
        Error::StringOperation("Missing closing parenthesis".to_string())
    })?;
    
    let args_str = &operation[start_pos..end_pos];
    
    // カンマで分割
    let parts: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
    
    let start = parts[0].parse::<usize>().map_err(|_| {
        Error::StringOperation("Invalid start position for substring".to_string())
    })?;
    
    let length = if parts.len() > 1 {
        Some(parts[1].parse::<usize>().map_err(|_| {
            Error::StringOperation("Invalid length for substring".to_string())
        })?)
    } else {
        None
    };
    
    Ok((start, length))
}

/// 引用符をクリーニング
fn clean_string_argument(arg: &str) -> Result<String, Error> {
    let cleaned = if (arg.starts_with('"') && arg.ends_with('"')) ||
                     (arg.starts_with('\'') && arg.ends_with('\'')) {
        &arg[1..arg.len()-1]
    } else {
        arg
    };
    
    Ok(cleaned.to_string())
}

/// 部分文字列を抽出
fn extract_substring(text: &str, start: usize, length: Option<usize>) -> Result<String, Error> {
    let chars: Vec<char> = text.chars().collect();
    
    if start >= chars.len() {
        return Ok("".to_string());
    }
    
    let end = match length {
        Some(len) => std::cmp::min(start + len, chars.len()),
        None => chars.len(),
    };
    
    Ok(chars[start..end].iter().collect())
}

/// 複数の文字列操作をパイプラインで適用
pub fn apply_string_pipeline(value: &Value, operations: &[&str]) -> Result<Value, Error> {
    let mut current_value = value.clone();
    
    for operation in operations {
        current_value = apply_string_operation(&current_value, operation)?;
    }
    
    Ok(current_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_string_operations() {
        let value = Value::String("Hello World".to_string());
        
        // upper
        let result = apply_string_operation(&value, "upper").unwrap();
        assert_eq!(result, Value::String("HELLO WORLD".to_string()));
        
        // lower
        let result = apply_string_operation(&value, "lower").unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
        
        // trim
        let padded = Value::String("  hello  ".to_string());
        let result = apply_string_operation(&padded, "trim").unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
        
        // length
        let result = apply_string_operation(&value, "length").unwrap();
        assert_eq!(result, Value::Number(11.into()));
        
        // reverse
        let result = apply_string_operation(&value, "reverse").unwrap();
        assert_eq!(result, Value::String("dlroW olleH".to_string()));
    }

    #[test]
    fn test_string_search_operations() {
        let value = Value::String("Hello World".to_string());
        
        // contains
        let result = apply_string_operation(&value, r#"contains("World")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        let result = apply_string_operation(&value, r#"contains("xyz")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
        
        // starts_with
        let result = apply_string_operation(&value, r#"starts_with("Hello")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        // ends_with
        let result = apply_string_operation(&value, r#"ends_with("World")"#).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_replace_operation() {
        let value = Value::String("Hello World".to_string());
        
        let result = apply_string_operation(&value, r#"replace("World", "Rust")"#).unwrap();
        assert_eq!(result, Value::String("Hello Rust".to_string()));
    }

    #[test]
    fn test_substring_operation() {
        let value = Value::String("Hello World".to_string());
        
        // substring with length
        let result = apply_string_operation(&value, "substring(0, 5)").unwrap();
        assert_eq!(result, Value::String("Hello".to_string()));
        
        // substring without length (to end)
        let result = apply_string_operation(&value, "substring(6)").unwrap();
        assert_eq!(result, Value::String("World".to_string()));
    }

    #[test]
    fn test_split_operation() {
        let value = Value::String("apple,banana,cherry".to_string());
        
        let result = apply_string_operation(&value, r#"split(",")"#).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::String("apple".to_string()));
            assert_eq!(arr[1], Value::String("banana".to_string()));
            assert_eq!(arr[2], Value::String("cherry".to_string()));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_join_operation() {
        let value = Value::Array(vec![
            Value::String("apple".to_string()),
            Value::String("banana".to_string()),
            Value::String("cherry".to_string()),
        ]);
        
        let result = apply_string_operation(&value, r#"join(", ")"#).unwrap();
        assert_eq!(result, Value::String("apple, banana, cherry".to_string()));
    }

    #[test]
    fn test_string_pipeline() {
        let value = Value::String("  Hello World  ".to_string());
        let operations = vec!["trim", "upper"];
        
        let result = apply_string_pipeline(&value, &operations).unwrap();
        assert_eq!(result, Value::String("HELLO WORLD".to_string()));
    }

    #[test]
    fn test_non_string_error() {
        let value = Value::Number(42.into());
        let result = apply_string_operation(&value, "upper");
        assert!(result.is_err());
    }
}
