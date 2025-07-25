use serde_json::{Value, json};

use crate::{
    Error, OutputFormat, apply_pipeline_operation, format_output, parse_array_segment,
    parse_query_segments, value_to_string,
};

pub fn execute_query(json: &Value, query: &str, format: OutputFormat) -> Result<(), Error> {
    if query.contains('|') {
        // 複数パイプライン処理
        let parts = split_pipeline_respecting_parentheses(query)?;

        if parts.len() < 2 {
            return Err(Error::InvalidQuery("Invalid pipeline syntax".into()));
        }

        // 最初のクエリでデータを取得
        let initial_query = parts[0].clone();
        let mut current_data = execute_basic_query_as_json(json, &initial_query)?;

        // 残りのパイプライン操作を順次実行
        for operation in &parts[1..] {
            current_data = apply_pipeline_operation(current_data, operation)?;
        }

        // 最終結果の出力
        format_output(&current_data, format)?;
    } else {
        let result_data = execute_basic_query_as_json(json, query)?;
        format_output(&result_data, format)?;
    }

    Ok(())
}

pub fn execute_basic_query(json: &Value, query: &str) -> Result<Vec<String>, Error> {
    let (segment, fields) = parse_query_segments(query)?;

    if segment.contains('[') && segment.contains(']') {
        let (idx, ridx) = parse_array_segment(segment)?;

        let key = segment
            .get(..idx)
            .ok_or(Error::InvalidQuery("Invalid segment format".into()))?;
        let index_str = segment
            .get(idx + 1..ridx)
            .ok_or(Error::InvalidQuery("Invalid bracket content".into()))?;

        if index_str.is_empty() {
            let result = handle_array_access(json, key, fields)?;
            Ok(result)
        } else {
            let index = index_str.parse::<usize>().map_err(Error::StrToInt)?;
            let result = handle_single_access(json, key, index, fields)?;
            Ok(result)
        }
    } else {
        let key = segment;
        let result = handle_array_access(json, key, fields)?;
        Ok(result)
    }
}

pub fn execute_basic_query_as_json(json: &Value, query: &str) -> Result<Vec<Value>, Error> {
    let (segment, fields) = parse_query_segments(query)?;

    if segment.is_empty() && fields.is_empty() {
        if let Value::Array(arr) = json {
            return Ok(arr.clone());
        } else {
            return Ok(vec![json.clone()]);
        }
    }

    // 単一オブジェクトへの直接フィールドアクセス
    if !segment.is_empty() && fields.is_empty() && !segment.contains('[') {
        if let Some(field_value) = json.get(segment) {
            return Ok(vec![field_value.clone()]);
        } else {
            return Err(Error::InvalidQuery(format!(
                "Field '{}' not found",
                segment
            )));
        }
    }

    // ルート配列アクセス（.[0] や .[0:5] のような場合）
    if segment.is_empty() && !fields.is_empty() {
        let first_field = fields[0];

        if first_field.starts_with('[') && first_field.ends_with(']') {
            let bracket_content = &first_field[1..first_field.len() - 1];

            // **新機能: スライス記法の処理**
            if bracket_content.contains(':') {
                if let Value::Array(arr) = json {
                    let (start, end) = parse_slice_notation(bracket_content)?;
                    let sliced = apply_array_slice(arr, start, end);

                    if fields.len() > 1 {
                        // スライス後にさらにフィールドアクセスがある場合
                        let remaining_fields = fields[1..].to_vec();
                        let mut results = Vec::new();

                        for item in sliced {
                            if let Ok(mut item_results) =
                                handle_nested_field_access(&item, remaining_fields.clone())
                            {
                                results.append(&mut item_results);
                            }
                        }
                        return Ok(results);
                    } else {
                        return Ok(sliced);
                    }
                } else {
                    return Err(Error::InvalidQuery("Cannot slice non-array value".into()));
                }
            }
            // 空括弧 [] の場合は配列全体を処理
            else if bracket_content.is_empty() {
                if let Value::Array(arr) = json {
                    if fields.len() > 1 {
                        let remaining_fields = fields[1..].to_vec();
                        let mut results = Vec::new();

                        for item in arr {
                            if let Ok(mut item_results) =
                                handle_nested_field_access(item, remaining_fields.clone())
                            {
                                results.append(&mut item_results);
                            }
                        }
                        return Ok(results);
                    } else {
                        return Ok(arr.clone());
                    }
                } else {
                    return Err(Error::InvalidQuery(
                        "Cannot iterate over non-array value".into(),
                    ));
                }
            } else {
                // 数値インデックスのみ
                let index = bracket_content.parse::<usize>().map_err(Error::StrToInt)?;

                if let Value::Array(arr) = json {
                    let item = arr.get(index).ok_or(Error::IndexOutOfBounds(index))?;

                    if fields.len() > 1 {
                        let remaining_fields = fields[1..].to_vec();
                        return handle_nested_field_access(item, remaining_fields);
                    } else {
                        return Ok(vec![item.clone()]);
                    }
                } else {
                    return Err(Error::InvalidQuery("Cannot index non-array value".into()));
                }
            }
        } else {
            // 単一オブジェクトのネストしたフィールドアクセス
            return handle_nested_field_access(json, fields);
        }
    }

    // 通常の配列アクセス（.users[0:5] のような場合）
    if segment.contains('[') && segment.contains(']') {
        let (idx, ridx) = parse_array_segment(segment)?;
        let key = segment
            .get(..idx)
            .ok_or(Error::InvalidQuery("Invalid segment format".into()))?;
        let bracket_content = segment
            .get(idx + 1..ridx)
            .ok_or(Error::InvalidQuery("Invalid bracket content".into()))?;

        // **新機能: フィールド付きスライス記法の処理**
        if bracket_content.contains(':') {
            let field_value = json
                .get(key)
                .ok_or(Error::InvalidQuery(format!("Key '{}' not found", key)))?;

            if let Value::Array(arr) = field_value {
                let (start, end) = parse_slice_notation(bracket_content)?;
                let sliced = apply_array_slice(arr, start, end);

                if !fields.is_empty() {
                    // スライス後にフィールドアクセスがある場合
                    let mut results = Vec::new();
                    for item in sliced {
                        if let Ok(mut item_results) =
                            handle_nested_field_access(&item, fields.clone())
                        {
                            results.append(&mut item_results);
                        }
                    }
                    Ok(results)
                } else {
                    Ok(sliced)
                }
            } else {
                Err(Error::InvalidQuery(format!(
                    "Field '{}' is not an array",
                    key
                )))
            }
        } else if bracket_content.is_empty() {
            let result = handle_array_access_as_json(json, key, fields)?;
            Ok(result)
        } else {
            let index = bracket_content.parse::<usize>().map_err(Error::StrToInt)?;

            let result = handle_single_access_as_json(json, key, index, fields)?;
            if let Value::Array(arr) = result {
                Ok(arr)
            } else {
                Ok(vec![result])
            }
        }
    } else {
        // 単一オブジェクトのフィールドアクセス対応
        if !fields.is_empty() {
            if let Some(first_value) = json.get(segment) {
                handle_nested_field_access(first_value, fields)
            } else {
                Err(Error::InvalidQuery(format!(
                    "Field '{}' not found",
                    segment
                )))
            }
        } else if let Some(field_value) = json.get(segment) {
            Ok(vec![field_value.clone()])
        } else {
            Err(Error::InvalidQuery(format!(
                "Field '{}' not found",
                segment
            )))
        }
    }
}

pub fn handle_nested_field_access(json: &Value, fields: Vec<&str>) -> Result<Vec<Value>, Error> {
    if fields.is_empty() {
        return Ok(vec![json.clone()]);
    }

    let field = fields[0];
    let remaining_fields = if fields.len() > 1 {
        fields[1..].to_vec()
    } else {
        vec![]
    };

    // 配列アクセス [0], [] の処理
    if field.contains('[') && field.contains(']') {
        let (idx, ridx) = parse_array_segment(field)?;
        let key = &field[..idx];
        let bracket_content = &field[idx + 1..ridx];

        if let Some(array_or_object) = json.get(key) {
            if bracket_content.is_empty() {
                if let Value::Array(arr) = array_or_object {
                    if remaining_fields.is_empty() {
                        Ok(arr.clone())
                    } else {
                        let mut all_results = Vec::new();
                        for item in arr {
                            if let Ok(mut item_results) =
                                handle_nested_field_access(item, remaining_fields.clone())
                            {
                                all_results.append(&mut item_results);
                            }
                        }
                        Ok(all_results)
                    }
                } else {
                    Err(Error::InvalidQuery(format!(
                        "Cannot iterate over non-array field '{}'",
                        key
                    )))
                }
            } else {
                // 数値インデックス [0] → 特定要素にアクセス
                let index = bracket_content.parse::<usize>().map_err(|e| {
                    Error::InvalidQuery(format!("Invalid array index '{}': {}", bracket_content, e))
                })?;

                if let Value::Array(arr) = array_or_object {
                    if let Some(item) = arr.get(index) {
                        if remaining_fields.is_empty() {
                            Ok(vec![item.clone()])
                        } else {
                            handle_nested_field_access(item, remaining_fields)
                        }
                    } else {
                        Err(Error::IndexOutOfBounds(index))
                    }
                } else {
                    Err(Error::InvalidQuery(format!(
                        "Cannot index non-array field '{}'",
                        key
                    )))
                }
            }
        } else {
            Err(Error::InvalidQuery(format!("Field '{}' not found", key)))
        }
    } else {
        // 通常のフィールドアクセス
        if let Some(value) = json.get(field) {
            if remaining_fields.is_empty() {
                Ok(vec![value.clone()])
            } else {
                handle_nested_field_access(value, remaining_fields)
            }
        } else {
            Err(Error::InvalidQuery(format!("Field '{}' not found", field)))
        }
    }
}

pub fn handle_single_access_as_json(
    json: &Value,
    key: &str,
    index: usize,
    fields: Vec<&str>,
) -> Result<Value, Error> {
    let values = json
        .get(key)
        .ok_or(Error::InvalidQuery(format!("Key '{}' not found", key)))?;
    let mut current = values.get(index).ok_or(Error::IndexOutOfBounds(index))?;

    for (field_idx, field) in fields.iter().enumerate() {
        if field.contains('[') && field.contains(']') {
            let (idx, ridx) = parse_array_segment(field)?;
            let field_key = field
                .get(..idx)
                .ok_or(Error::InvalidQuery("Invalid field".into()))?;
            let index_str = field
                .get(idx + 1..ridx)
                .ok_or(Error::InvalidQuery("Invalid bracket content".into()))?;

            // field_key でアクセス
            let array = current.get(field_key).ok_or(Error::InvalidQuery(format!(
                "Field '{}' not found",
                field_key
            )))?;

            if index_str.is_empty() {
                if let Value::Array(arr) = array {
                    // 修正: 残りのフィールドがある場合の処理
                    let remaining_fields = if field_idx + 1 < fields.len() {
                        fields[field_idx + 1..].to_vec()
                    } else {
                        vec![]
                    };

                    if remaining_fields.is_empty() {
                        // 残りフィールドなし → 配列全体を返す
                        return Ok(Value::Array(arr.clone()));
                    } else {
                        // 修正: 残りフィールドあり → 各要素に適用
                        let mut expanded_results = Vec::new();
                        for item in arr {
                            if let Ok(mut item_results) =
                                handle_nested_field_access(item, remaining_fields.clone())
                            {
                                expanded_results.append(&mut item_results);
                            }
                        }
                        return Ok(Value::Array(expanded_results));
                    }
                } else {
                    return Err(Error::InvalidQuery(format!(
                        "Field '{}' is not an array",
                        field_key
                    )));
                }
            } else {
                // 数値インデックスの場合
                let field_index = index_str.parse::<usize>().map_err(Error::StrToInt)?;
                current = array
                    .get(field_index)
                    .ok_or(Error::IndexOutOfBounds(field_index))?;
            }
        } else {
            // 通常のフィールドアクセス
            current = current
                .get(field)
                .ok_or(Error::InvalidQuery(format!("Field '{}' not found", field)))?;
        }
    }

    Ok(current.clone())
}

pub fn handle_array_access_as_json(
    json: &Value,
    key: &str,
    fields: Vec<&str>,
) -> Result<Vec<Value>, Error> {
    let values = json
        .get(key)
        .ok_or(Error::InvalidQuery(format!("Key '{}' not found", key)))?;
    let values_arr = values
        .as_array()
        .ok_or(Error::InvalidQuery("Expected array".into()))?;

    let res: Vec<Value> = values_arr
        .iter()
        .filter_map(|array_item| {
            // 各配列要素に対してフィールドパスを辿る
            let mut current = array_item;

            for field in &fields {
                if field.contains('[') && field.contains(']') {
                    // 配列アクセスの場合
                    if let Ok((idx, ridx)) = parse_array_segment(field) {
                        if let Some(field_key) = field.get(..idx) {
                            if let Some(index_str) = field.get(idx + 1..ridx) {
                                if let Ok(field_index) = index_str.parse::<usize>() {
                                    if let Some(array) = current.get(field_key) {
                                        if let Some(item) = array.get(field_index) {
                                            current = item;
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // エラーの場合はこの要素をスキップ
                    return None;
                } else {
                    // 通常のフィールドアクセス
                    if let Some(next) = current.get(field) {
                        current = next;
                    } else {
                        // フィールドが存在しない場合はこの要素をスキップ
                        return None;
                    }
                }
            }

            Some(current.clone())
        })
        .collect();

    Ok(res)
}

pub fn handle_single_access(
    json: &Value,
    key: &str,
    index: usize,
    fields: Vec<&str>,
) -> Result<Vec<String>, Error> {
    // 最初の配列要素を取得
    let values = json
        .get(key)
        .ok_or(Error::InvalidQuery(format!("Key '{}' not found", key)))?;
    let mut current = values.get(index).ok_or(Error::IndexOutOfBounds(index))?;

    // fieldsを順次辿る
    for field in fields {
        if field.contains('[') && field.contains(']') {
            // 配列アクセスの場合
            let (idx, ridx) = parse_array_segment(field)?;
            let field_key = field
                .get(..idx)
                .ok_or(Error::InvalidQuery("Invalid field".into()))?;
            let index_str = field
                .get(idx + 1..ridx)
                .ok_or(Error::InvalidQuery("Invalid bracket content".into()))?;
            let field_index = index_str.parse::<usize>().map_err(Error::StrToInt)?;

            // field_key でアクセスしてから、field_index でアクセス
            let array = current.get(field_key).ok_or(Error::InvalidQuery(format!(
                "Field '{}' not found",
                field_key
            )))?;
            current = array
                .get(field_index)
                .ok_or(Error::IndexOutOfBounds(field_index))?;
        } else {
            // Normal field access
            current = current
                .get(field)
                .ok_or(Error::InvalidQuery(format!("Field '{}' not found", field)))?;
        }
    }

    Ok(vec![value_to_string(current)])
}

pub fn handle_array_access(
    json: &Value,
    key: &str,
    fields: Vec<&str>,
) -> Result<Vec<String>, Error> {
    let values = json
        .get(key)
        .ok_or(Error::InvalidQuery(format!("Key '{}' not found", key)))?;
    let values_arr = values
        .as_array()
        .ok_or(Error::InvalidQuery("Expected array".into()))?;

    let res: Vec<String> = values_arr
        .iter()
        .filter_map(|array_item| {
            // 各配列要素に対してフィールドパスを辿る
            let mut current = array_item;

            for field in &fields {
                if field.contains('[') && field.contains(']') {
                    // 配列アクセスの場合
                    if let Ok((idx, ridx)) = parse_array_segment(field) {
                        if let Some(field_key) = field.get(..idx) {
                            if let Some(index_str) = field.get(idx + 1..ridx) {
                                if let Ok(field_index) = index_str.parse::<usize>() {
                                    if let Some(array) = current.get(field_key) {
                                        if let Some(item) = array.get(field_index) {
                                            current = item;
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // エラーの場合はこの要素をスキップ
                    return None;
                } else {
                    // 通常のフィールドアクセス
                    if let Some(next) = current.get(field) {
                        current = next;
                    } else {
                        // フィールドが存在しない場合はこの要素をスキップ
                        return None;
                    }
                }
            }

            Some(value_to_string(current))
        })
        .collect();

    Ok(res)
}

fn split_pipeline_respecting_parentheses(query: &str) -> Result<Vec<String>, Error> {
    let mut parts = Vec::new();
    let mut current_part = String::new();
    let mut paren_depth = 0;
    let chars = query.chars().peekable();

    for ch in chars {
        match ch {
            '(' => {
                paren_depth += 1;
                current_part.push(ch);
            }
            ')' => {
                paren_depth -= 1;
                current_part.push(ch);
            }
            '|' if paren_depth == 0 => {
                // 括弧の外のパイプのみで分割
                if !current_part.trim().is_empty() {
                    parts.push(current_part.trim().to_string());
                    current_part.clear();
                }
            }
            _ => {
                current_part.push(ch);
            }
        }
    }

    // 最後の部分を追加
    if !current_part.trim().is_empty() {
        parts.push(current_part.trim().to_string());
    }

    if paren_depth != 0 {
        return Err(Error::InvalidQuery(
            "Unmatched parentheses in query".to_string(),
        ));
    }

    Ok(parts)
}

/// 配列スライス記法をパース ([start:end] 形式)
fn parse_slice_notation(bracket_content: &str) -> Result<(Option<usize>, Option<usize>), Error> {
    if !bracket_content.contains(':') {
        return Err(Error::InvalidQuery("Not a slice notation".to_string()));
    }

    let parts: Vec<&str> = bracket_content.split(':').collect();
    if parts.len() != 2 {
        return Err(Error::InvalidQuery(
            "Invalid slice format, expected start:end".to_string(),
        ));
    }

    let start = if parts[0].is_empty() {
        None // 空の場合は先頭から
    } else {
        Some(
            parts[0]
                .parse::<usize>()
                .map_err(|_| Error::InvalidQuery(format!("Invalid start index: {}", parts[0])))?,
        )
    };

    let end = if parts[1].is_empty() {
        None // 空の場合は末尾まで
    } else {
        Some(
            parts[1]
                .parse::<usize>()
                .map_err(|_| Error::InvalidQuery(format!("Invalid end index: {}", parts[1])))?,
        )
    };

    Ok((start, end))
}

/// 配列に対してスライス操作を適用
fn apply_array_slice(array: &[Value], start: Option<usize>, end: Option<usize>) -> Vec<Value> {
    let len = array.len();

    let start_idx = start.unwrap_or(0);
    let end_idx = end.unwrap_or(len);

    // 範囲チェック
    let start_idx = start_idx.min(len);
    let end_idx = end_idx.min(len);

    if start_idx >= end_idx {
        return Vec::new(); // 無効な範囲の場合は空を返す
    }

    array[start_idx..end_idx].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn test_handle_single_access_simple() {
        let json = create_nested_test_json();
        let result = handle_single_access(&json, "users", 0, vec!["name"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["Alice"]);
    }

    #[test]
    fn test_handle_single_access_nested_object() {
        let json = create_nested_test_json();
        let result = handle_single_access(&json, "users", 0, vec!["address", "city"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["Tokyo"]);
    }

    #[test]
    fn test_handle_single_access_nested_array() {
        let json = create_nested_test_json();
        let result = handle_single_access(&json, "users", 0, vec!["projects[0]", "name"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["Project A"]);
    }

    #[test]
    fn test_handle_single_access_deep_nesting() {
        let json = create_nested_test_json();
        let result = handle_single_access(&json, "users", 1, vec!["projects[0]", "status"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["planning"]);
    }

    #[test]
    fn test_handle_single_access_nested_array_out_of_bounds() {
        let json = create_nested_test_json();
        let result = handle_single_access(&json, "users", 0, vec!["projects[999]", "name"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_array_access_simple() {
        let json = create_nested_test_json();
        let result = handle_array_access(&json, "users", vec!["name"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["Alice", "Bob"]);
    }

    #[test]
    fn test_handle_array_access_nested_object() {
        let json = create_nested_test_json();
        let result = handle_array_access(&json, "users", vec!["address", "city"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["Tokyo", "Osaka"]);
    }

    #[test]
    fn test_handle_array_access_nested_array() {
        let json = create_nested_test_json();
        let result = handle_array_access(&json, "users", vec!["projects[0]", "name"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["Project A", "Project C"]);
    }

    #[test]
    fn test_handle_array_access_partial_data() {
        // 一部の要素にフィールドがない場合
        let json = json!({
            "items": [
                {"details": {"name": "Item1"}},
                {"other": "data"},  // detailsフィールドなし
                {"details": {"name": "Item3"}}
            ]
        });
        let result = handle_array_access(&json, "items", vec!["details", "name"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["Item1", "Item3"]); // 中間要素はスキップ
    }

    // **新規追加: 単一オブジェクトアクセスのテスト**
    #[test]
    fn test_single_object_direct_field_access() {
        let json = json!({
            "Parameters": {
                "VpcCidr": "10.0.0.0/16",
                "SubnetCidr": "10.0.1.0/24"
            },
            "Resources": {
                "VPC": {"Type": "AWS::EC2::VPC"}
            }
        });

        // 直接フィールドアクセス
        let result = execute_basic_query_as_json(&json, ".Parameters");
        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert!(values[0].is_object());
        assert!(values[0].get("VpcCidr").is_some());
    }

    #[test]
    fn test_single_object_nested_field_access() {
        let json = json!({
            "Parameters": {
                "VpcCidr": "10.0.0.0/16",
                "SubnetCidr": "10.0.1.0/24"
            },
            "Resources": {
                "VPC": {"Type": "AWS::EC2::VPC"}
            }
        });

        // ネストしたフィールドアクセス
        let result = execute_basic_query_as_json(&json, ".Parameters.VpcCidr");
        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], Value::String("10.0.0.0/16".to_string()));
    }

    #[test]
    fn test_single_object_multiple_levels() {
        let json = json!({
            "Resources": {
                "VPC": {
                    "Type": "AWS::EC2::VPC",
                    "Properties": {
                        "CidrBlock": "10.0.0.0/16"
                    }
                }
            }
        });

        let result = execute_basic_query_as_json(&json, ".Resources.VPC.Properties.CidrBlock");
        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], Value::String("10.0.0.0/16".to_string()));
    }

    #[test]
    fn test_single_object_field_not_found() {
        let json = json!({
            "Parameters": {
                "VpcCidr": "10.0.0.0/16"
            }
        });

        let result = execute_basic_query_as_json(&json, ".NonExistent");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidQuery(msg) => {
                assert!(msg.contains("Field 'NonExistent' not found"));
            }
            _ => panic!("Expected InvalidQuery error"),
        }
    }
}

pub fn test_handle_array_access_normal_case() {
    // 正常ケース: 基本的な配列アクセス
    let json = create_test_json();
    let result = handle_array_access(&json, "users", vec!["name"]);

    assert!(result.is_ok());
    let names = result.unwrap();
    assert_eq!(names, vec!["Alice", "Bob", "Carol"]);
}

#[test]
fn test_handle_array_access_with_missing_field() {
    // 正常ケース: 一部の要素にフィールドがない（filter_mapで対応）
    let json = create_test_json();
    let result = handle_array_access(&json, "users", vec!["active"]);

    assert!(result.is_ok());
    let actives = result.unwrap();
    assert_eq!(actives, vec!["true", "false"]); // Carolにはactiveフィールドがない
}

#[test]
fn test_handle_array_access_different_types() {
    // 正常ケース: 異なる型のフィールド
    let json = create_test_json();
    let result = handle_array_access(&json, "users", vec!["age"]);

    assert!(result.is_ok());
    let ages = result.unwrap();
    assert_eq!(ages, vec!["30", "25", "35"]);
}

#[test]
fn test_handle_array_access_empty_array() {
    // 正常ケース: 空の配列
    let json = create_test_json();
    let result = handle_array_access(&json, "empty_array", vec!["name"]);

    assert!(result.is_ok());
    let names = result.unwrap();
    assert!(names.is_empty());
}

#[test]
fn test_handle_array_access_key_not_found() {
    // エラーケース: 存在しないキー
    let json = create_test_json();
    let result = handle_array_access(&json, "nonexistent", vec!["name"]);

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::InvalidQuery(msg) => {
            assert!(msg.contains("Key 'nonexistent' not found"));
        }
        _ => panic!("Expected InvalidQuery error"),
    }
}

#[test]
fn test_handle_array_access_not_array() {
    // エラーケース: 配列ではない値
    let json = create_test_json();
    let result = handle_array_access(&json, "not_array", vec!["name"]);

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::InvalidQuery(msg) => {
            assert!(msg.contains("Expected array"));
        }
        _ => panic!("Expected InvalidQuery error"),
    }
}

#[test]
fn test_handle_array_access_field_not_in_any_element() {
    // 正常ケース: どの要素にもフィールドがない
    let json = create_test_json();
    let result = handle_array_access(&json, "users", vec!["nonexistent_field"]);

    assert!(result.is_ok());
    let values = result.unwrap();
    assert!(values.is_empty()); // filter_mapで空になる
}

#[test]
fn test_handle_single_access_normal_case() {
    // 正常ケース: 基本的な単一要素アクセス
    let json = create_test_json();
    let result = handle_single_access(&json, "users", 0, vec!["name"]);

    assert!(result.is_ok());
    let names = result.unwrap();
    assert_eq!(names, vec!["Alice"]);
}

#[test]
fn test_handle_single_access_different_index() {
    // 正常ケース: 異なるインデックス
    let json = create_test_json();
    let result = handle_single_access(&json, "users", 1, vec!["name"]);

    assert!(result.is_ok());
    let names = result.unwrap();
    assert_eq!(names, vec!["Bob"]);
}

#[test]
fn test_handle_single_access_different_field() {
    // 正常ケース: 異なるフィールド
    let json = create_test_json();
    let result = handle_single_access(&json, "users", 0, vec!["age"]);

    assert!(result.is_ok());
    let ages = result.unwrap();
    assert_eq!(ages, vec!["30"]);
}

#[test]
fn test_handle_single_access_boolean_field() {
    // 正常ケース: Boolean型のフィールド
    let json = create_test_json();
    let result = handle_single_access(&json, "users", 0, vec!["active"]);

    assert!(result.is_ok());
    let actives = result.unwrap();
    assert_eq!(actives, vec!["true"]);
}

#[test]
fn test_handle_single_access_key_not_found() {
    // エラーケース: 存在しないキー
    let json = create_test_json();
    let result = handle_single_access(&json, "nonexistent", 0, vec!["name"]);

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::InvalidQuery(msg) => {
            assert!(msg.contains("Key 'nonexistent' not found"));
        }
        _ => panic!("Expected InvalidQuery error"),
    }
}

#[test]
fn test_handle_single_access_index_out_of_bounds() {
    // エラーケース: 配列の範囲外インデックス
    let json = create_test_json();
    let result = handle_single_access(&json, "users", 999, vec!["name"]);

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::IndexOutOfBounds(index) => {
            assert_eq!(index, 999);
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}

#[test]
fn test_handle_single_access_field_not_found() {
    // エラーケース: 存在しないフィールド
    let json = create_test_json();
    let result = handle_single_access(&json, "users", 0, vec!["nonexistent_field"]);

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::InvalidQuery(msg) => {
            assert!(msg.contains("Field 'nonexistent_field' not found"));
        }
        _ => panic!("Expected InvalidQuery error"),
    }
}

#[test]
fn test_handle_single_access_not_array() {
    // エラーケース: 配列ではない値へのインデックスアクセス
    let json = create_test_json();
    let result = handle_single_access(&json, "not_array", 0, vec!["name"]);

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::IndexOutOfBounds(_) => {
            // not_arrayは文字列なので、.get(0)がNoneを返してIndexOutOfBounds
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}

#[test]
fn test_handle_single_access_empty_array() {
    // エラーケース: 空配列へのアクセス
    let json = create_test_json();
    let result = handle_single_access(&json, "empty_array", 0, vec!["name"]);

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::IndexOutOfBounds(index) => {
            assert_eq!(index, 0);
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}

pub fn create_nested_test_json() -> Value {
    json!({
        "users": [
            {
                "name": "Alice",
                "age": 30,
                "address": {"city": "Tokyo", "country": "Japan"},
                "projects": [
                    {"name": "Project A", "status": "active"},
                    {"name": "Project B", "status": "completed"}
                ]
            },
            {
                "name": "Bob",
                "age": 25,
                "address": {"city": "Osaka", "country": "Japan"},
                "projects": [
                    {"name": "Project C", "status": "planning"}
                ]
            }
        ]
    })
}

pub fn create_test_json() -> Value {
    json!({
        "users": [
            {"name": "Alice", "age": 30, "active": true},
            {"name": "Bob", "age": 25, "active": false},
            {"name": "Carol", "age": 35}
        ],
        "products": [
            {"title": "Laptop", "price": 1200},
            {"title": "Mouse", "price": 25}
        ],
        "empty_array": [],
        "not_array": "string_value"
    })
}
