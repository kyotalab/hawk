use crate::Error;
use serde_json::Value;

/// 文字列操作を適用する
pub fn apply_string_operation(value: &Value, operation: &str) -> Result<Value, Error> {
    match operation {
        "upper" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::String(string_val.to_uppercase()))
        }
        "lower" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::String(string_val.to_lowercase()))
        }
        "trim" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::String(string_val.trim().to_string()))
        }
        "trim_start" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::String(string_val.trim_start().to_string()))
        }
        "trim_end" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::String(string_val.trim_end().to_string()))
        }
        "length" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::Number(serde_json::Number::from(
                string_val.chars().count(),
            )))
        }
        "reverse" => {
            let string_val = extract_string_value(value)?;
            Ok(Value::String(string_val.chars().rev().collect()))
        }
        // **拡張: OR条件対応のcontains**
        op if op.starts_with("contains(") && op.ends_with(")") => {
            let string_val = extract_string_value(value)?;
            let pattern = extract_string_argument(op)?;

            // パイプ区切りでOR条件をサポート
            if pattern.contains('|') {
                apply_contains_or_condition(string_val, &pattern)
            } else {
                Ok(Value::Bool(string_val.contains(&pattern)))
            }
        }

        // **拡張: OR条件対応のstarts_with**
        op if op.starts_with("starts_with(") && op.ends_with(")") => {
            let string_val = extract_string_value(value)?;
            let pattern = extract_string_argument(op)?;

            if pattern.contains('|') {
                apply_starts_with_or_condition(string_val, &pattern)
            } else {
                Ok(Value::Bool(string_val.starts_with(&pattern)))
            }
        }

        // **拡張: OR条件対応のends_with**
        op if op.starts_with("ends_with(") && op.ends_with(")") => {
            let string_val = extract_string_value(value)?;
            let pattern = extract_string_argument(op)?;

            if pattern.contains('|') {
                apply_ends_with_or_condition(string_val, &pattern)
            } else {
                Ok(Value::Bool(string_val.ends_with(&pattern)))
            }
        }
        //op if op.starts_with("contains(") && op.ends_with(")") => {
        //    let string_val = extract_string_value(value)?;
        //    let pattern = extract_string_argument(op)?;
        //    Ok(Value::Bool(string_val.contains(&pattern)))
        //},
        //op if op.starts_with("starts_with(") && op.ends_with(")") => {
        //    let string_val = extract_string_value(value)?;
        //    let pattern = extract_string_argument(op)?;
        //    Ok(Value::Bool(string_val.starts_with(&pattern)))
        //},
        //op if op.starts_with("ends_with(") && op.ends_with(")") => {
        //    let string_val = extract_string_value(value)?;
        //    let pattern = extract_string_argument(op)?;
        //    Ok(Value::Bool(string_val.ends_with(&pattern)))
        //},
        op if op.starts_with("replace(") && op.ends_with(")") => {
            let string_val = extract_string_value(value)?;
            let (old, new) = extract_replace_arguments(op)?;
            Ok(Value::String(string_val.replace(&old, &new)))
        }
        op if op.starts_with("substring(") && op.ends_with(")") => {
            let string_val = extract_string_value(value)?;
            let (start, length) = extract_substring_arguments(op)?;
            let result = extract_substring(string_val, start, length)?;
            Ok(Value::String(result))
        }
        op if op.starts_with("split(") && op.contains(")[") => {
            apply_split_with_slice_range(value, op)
        }
        op if op.starts_with("split(") && op.contains(")[") => {
            // **新機能: split(...)[index] 形式の処理**
            apply_split_with_index(value, op)
        }
        op if op.starts_with("split(") && op.ends_with(")") => {
            let string_val = extract_string_value(value)?;
            let delimiter = extract_string_argument(op)?;
            let parts: Vec<Value> = string_val
                .split(&delimiter)
                .map(|s| Value::String(s.to_string()))
                .collect();
            Ok(Value::Array(parts))
        }
        op if op.starts_with("join(") && op.ends_with(")") => {
            // join操作は配列に対して適用
            apply_join_operation(value, op)
        }
        _ => Err(Error::StringOperation(format!(
            "Unknown string operation: {}",
            operation
        ))),
    }
}

/// split(...)[index] 形式の処理
fn apply_split_with_index(value: &Value, operation: &str) -> Result<Value, Error> {
    // "split(\" \")[0]" のような形式を解析
    let split_end = operation
        .find(")[")
        .ok_or_else(|| Error::StringOperation("Invalid split with index format".to_string()))?;

    let bracket_start = split_end + 2; // ")[" の後
    let bracket_end = operation.len() - 1; // 最後の ']'

    if !operation.ends_with(']') {
        return Err(Error::StringOperation(
            "Missing closing bracket in split index".to_string(),
        ));
    }

    // split部分とindex部分を分離
    let split_part = &operation[..split_end + 1]; // "split(\" \")"
    let index_part = &operation[bracket_start..bracket_end]; // "0"

    // まずsplitを実行
    let string_val = extract_string_value(value)?;
    let delimiter = extract_string_argument(split_part)?;
    let parts: Vec<&str> = string_val.split(&delimiter).collect();

    // インデックスを解析
    let index = index_part
        .parse::<usize>()
        .map_err(|_| Error::StringOperation(format!("Invalid array index: {}", index_part)))?;

    // インデックスでアクセス
    if let Some(part) = parts.get(index) {
        Ok(Value::String(part.to_string()))
    } else {
        // インデックスが範囲外の場合は空文字列を返す（配列の動作に合わせる）
        Ok(Value::String("".to_string()))
    }
}

/// 文字列値を抽出（エラーハンドリングを統一）
fn extract_string_value(value: &Value) -> Result<&str, Error> {
    match value {
        Value::String(s) => Ok(s),
        _ => Err(Error::StringOperation(format!(
            "String operations can only be applied to string values, got: {}",
            get_type_name(value)
        ))),
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
                _ => Err(Error::StringOperation(
                    "Cannot join non-primitive values".to_string(),
                )),
            })
            .collect();

        let parts = string_parts?;
        Ok(Value::String(parts.join(&delimiter)))
    } else {
        Err(Error::StringOperation(
            "join can only be applied to arrays".to_string(),
        ))
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
    let start_pos = operation
        .find('(')
        .ok_or_else(|| Error::StringOperation("Missing opening parenthesis".to_string()))?
        + 1;
    let end_pos = operation
        .rfind(')')
        .ok_or_else(|| Error::StringOperation("Missing closing parenthesis".to_string()))?;

    if start_pos >= end_pos {
        return Err(Error::StringOperation(
            "Invalid argument format".to_string(),
        ));
    }

    let arg = &operation[start_pos..end_pos];

    // 引用符を除去
    let cleaned = if (arg.starts_with('"') && arg.ends_with('"'))
        || (arg.starts_with('\'') && arg.ends_with('\''))
    {
        &arg[1..arg.len() - 1]
    } else {
        arg
    };

    Ok(cleaned.to_string())
}

/// replace引数を抽出（例: replace("old", "new") → ("old", "new")）
fn extract_replace_arguments(operation: &str) -> Result<(String, String), Error> {
    let start_pos = operation
        .find('(')
        .ok_or_else(|| Error::StringOperation("Missing opening parenthesis".to_string()))?
        + 1;
    let end_pos = operation
        .rfind(')')
        .ok_or_else(|| Error::StringOperation("Missing closing parenthesis".to_string()))?;

    let args_str = &operation[start_pos..end_pos];

    // カンマで分割
    let parts: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err(Error::StringOperation(
            "replace requires exactly 2 arguments".to_string(),
        ));
    }

    let old = clean_string_argument(parts[0])?;
    let new = clean_string_argument(parts[1])?;

    Ok((old, new))
}

/// substring引数を抽出（例: substring(0, 5) → (0, 5)）
fn extract_substring_arguments(operation: &str) -> Result<(usize, Option<usize>), Error> {
    let start_pos = operation
        .find('(')
        .ok_or_else(|| Error::StringOperation("Missing opening parenthesis".to_string()))?
        + 1;
    let end_pos = operation
        .rfind(')')
        .ok_or_else(|| Error::StringOperation("Missing closing parenthesis".to_string()))?;

    let args_str = &operation[start_pos..end_pos];

    // カンマで分割
    let parts: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();

    let start = parts[0]
        .parse::<usize>()
        .map_err(|_| Error::StringOperation("Invalid start position for substring".to_string()))?;

    let length = if parts.len() > 1 {
        Some(
            parts[1]
                .parse::<usize>()
                .map_err(|_| Error::StringOperation("Invalid length for substring".to_string()))?,
        )
    } else {
        None
    };

    Ok((start, length))
}

/// 引用符をクリーニング
fn clean_string_argument(arg: &str) -> Result<String, Error> {
    let cleaned = if (arg.starts_with('"') && arg.ends_with('"'))
        || (arg.starts_with('\'') && arg.ends_with('\''))
    {
        &arg[1..arg.len() - 1]
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

/// 複数フィールドに対して同じ操作を適用（ケース1）
pub fn apply_operation_to_multiple_fields(
    item: &Value,
    field_paths: &[&str],
    operation: &str,
) -> Result<Value, Error> {
    let mut updated_item = item.clone();

    // 各フィールドに同じ操作を適用
    for field_path in field_paths {
        // フィールド値を取得
        let field_value = extract_field_value_from_item(item, field_path)?;

        // 操作を適用
        let transformed_value = apply_string_operation(&field_value, operation)?;

        // フィールドを更新
        updated_item = update_field_in_item(updated_item, field_path, transformed_value)?;
    }

    Ok(updated_item)
}

/// アイテムからフィールド値を抽出
fn extract_field_value_from_item(item: &Value, field_path: &str) -> Result<Value, Error> {
    if field_path == "." {
        return Ok(item.clone());
    }

    if !field_path.starts_with('.') {
        return Err(Error::StringOperation(format!(
            "Field path must start with '.': {}",
            field_path
        )));
    }

    let field_name = &field_path[1..]; // '.' を除去

    if let Some(value) = item.get(field_name) {
        Ok(value.clone())
    } else {
        Err(Error::StringOperation(format!(
            "Field '{}' not found",
            field_name
        )))
    }
}

/// アイテムのフィールドを更新
fn update_field_in_item(item: Value, field_path: &str, new_value: Value) -> Result<Value, Error> {
    if field_path == "." {
        // ルート値の場合は直接置き換え
        return Ok(new_value);
    }

    if !field_path.starts_with('.') {
        return Err(Error::StringOperation(format!(
            "Field path must start with '.': {}",
            field_path
        )));
    }

    let field_name = &field_path[1..]; // '.' を除去

    if let Value::Object(mut obj) = item {
        obj.insert(field_name.to_string(), new_value);
        Ok(Value::Object(obj))
    } else {
        // オブジェクトでない場合は新しいオブジェクトを作成
        let mut new_obj = serde_json::Map::new();
        new_obj.insert(field_name.to_string(), new_value);
        Ok(Value::Object(new_obj))
    }
}

fn apply_split_with_slice_range(value: &Value, operation: &str) -> Result<Value, Error> {
    // "split(\" \")[0:3]" のような形式を解析
    let split_end = operation
        .find(")[")
        .ok_or_else(|| Error::StringOperation("Invalid split with slice format".to_string()))?;

    let bracket_start = split_end + 2; // ")[" の後
    let bracket_end = operation.len() - 1; // 最後の ']'

    if !operation.ends_with(']') {
        return Err(Error::StringOperation(
            "Missing closing bracket in split slice".to_string(),
        ));
    }

    // split部分とslice部分を分離
    let split_part = &operation[..split_end + 1]; // "split(\" \")"
    let slice_part = &operation[bracket_start..bracket_end]; // "0:3"

    // まずsplitを実行
    let string_val = extract_string_value(value)?;
    let delimiter = extract_string_argument(split_part)?;
    let parts: Vec<String> = string_val
        .split(&delimiter)
        .map(|s| s.to_string())
        .collect();

    // スライス記法を解析
    if slice_part.contains(':') {
        let (start, end) = parse_slice_notation_for_split(slice_part, parts.len())?;
        let sliced_parts = apply_slice_to_string_array(&parts, start, end);

        let result: Vec<Value> = sliced_parts
            .into_iter()
            .map(|s| Value::String(s.to_string()))
            .collect();

        Ok(Value::Array(result))
    } else {
        // 単一インデックスの場合
        let index = slice_part
            .parse::<usize>()
            .map_err(|_| Error::StringOperation(format!("Invalid array index: {}", slice_part)))?;

        if let Some(part) = parts.get(index) {
            Ok(Value::String(part.to_string()))
        } else {
            Ok(Value::String("".to_string())) // 範囲外は空文字列
        }
    }
}

/// 文字列配列用のスライス記法解析
fn parse_slice_notation_for_split(
    slice_str: &str,
    array_len: usize,
) -> Result<(Option<usize>, Option<usize>), Error> {
    let parts: Vec<&str> = slice_str.split(':').collect();
    if parts.len() != 2 {
        return Err(Error::StringOperation(
            "Invalid slice format, expected start:end".to_string(),
        ));
    }

    let start =
        if parts[0].is_empty() {
            None
        } else if parts[0].starts_with('-') {
            // 負のインデックス対応
            let neg_idx = parts[0][1..].parse::<usize>().map_err(|_| {
                Error::StringOperation(format!("Invalid negative index: {}", parts[0]))
            })?;
            Some(array_len.saturating_sub(neg_idx))
        } else {
            Some(parts[0].parse::<usize>().map_err(|_| {
                Error::StringOperation(format!("Invalid start index: {}", parts[0]))
            })?)
        };

    let end = if parts[1].is_empty() {
        None
    } else if parts[1].starts_with('-') {
        // 負のインデックス対応
        let neg_idx = parts[1][1..]
            .parse::<usize>()
            .map_err(|_| Error::StringOperation(format!("Invalid negative index: {}", parts[1])))?;
        Some(array_len.saturating_sub(neg_idx))
    } else {
        Some(
            parts[1]
                .parse::<usize>()
                .map_err(|_| Error::StringOperation(format!("Invalid end index: {}", parts[1])))?,
        )
    };

    Ok((start, end))
}

/// 文字列配列にスライスを適用
fn apply_slice_to_string_array(
    array: &[String],
    start: Option<usize>,
    end: Option<usize>,
) -> Vec<String> {
    let len = array.len();

    let start_idx = start.unwrap_or(0);
    let end_idx = end.unwrap_or(len);

    // 範囲チェック
    let start_idx = start_idx.min(len);
    let end_idx = end_idx.min(len);

    if start_idx >= end_idx {
        return Vec::new();
    }

    array[start_idx..end_idx].to_vec()
}

/// contains のOR条件処理
fn apply_contains_or_condition(text: &str, pattern: &str) -> Result<Value, Error> {
    let patterns: Vec<&str> = pattern.split('|').map(|p| p.trim()).collect();
    let result = patterns.iter().any(|p| text.contains(p));
    Ok(Value::Bool(result))
}

/// starts_with のOR条件処理
fn apply_starts_with_or_condition(text: &str, pattern: &str) -> Result<Value, Error> {
    let patterns: Vec<&str> = pattern.split('|').map(|p| p.trim()).collect();
    let result = patterns.iter().any(|p| text.starts_with(p));
    Ok(Value::Bool(result))
}

/// ends_with のOR条件処理
fn apply_ends_with_or_condition(text: &str, pattern: &str) -> Result<Value, Error> {
    let patterns: Vec<&str> = pattern.split('|').map(|p| p.trim()).collect();
    let result = patterns.iter().any(|p| text.ends_with(p));
    Ok(Value::Bool(result))
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
