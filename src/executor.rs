use serde_json::Value;

use crate::{
    Error, OutputFormat, apply_pipeline_operation, format_output, parse_array_segment,
    parse_query_segments, value_to_string,
};

pub fn execute_query(json: &Value, query: &str, format: OutputFormat) -> Result<(), Error> {
    if query.contains('|') {
        // 複数パイプライン処理
        // Multiple pipeline processing
        let parts: Vec<&str> = query.split('|').map(|s| s.trim()).collect();

        if parts.len() < 2 {
            return Err(Error::InvalidQuery("Invalid pipeline syntax".into()));
        }

        // 最初のクエリでデータを取得
        // Retrieve data with the first query
        let initial_query = parts[0];
        let mut current_data = execute_basic_query_as_json(json, initial_query)?;

        // 残りのパイプライン操作を順次実行
        // Execute the remaining pipeline operations sequentially.
        for operation in &parts[1..] {
            current_data = apply_pipeline_operation(current_data, &operation)?;
        }

        // 最終結果の出力
        // Output of final results
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
        let (idx, ridx) = parse_array_segment(segment)?; // debug

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
            let index = index_str.parse::<usize>().map_err(|e| Error::StrToInt(e))?;
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
        // JSONが配列の場合は、その要素を展開して返す
        // If JSON is an array, expand its elements and return them.
        if let Value::Array(arr) = json {
            return Ok(arr.clone());
        } else {
            // オブジェクトの場合はそのまま
            // In the case of objects, leave as is.
            return Ok(vec![json.clone()]);
        }
    }
    // ルート配列アクセス（.[0] のような場合）
    if segment.is_empty() && !fields.is_empty() {
        let first_field = fields[0];

        // [0] のような配列インデックスかチェック
        if first_field.starts_with('[') && first_field.ends_with(']') {
            let index_str = &first_field[1..first_field.len() - 1];

            // 空括弧 [] の場合は配列全体を処理
            if index_str.is_empty() {
                if let Value::Array(arr) = json {
                    // 残りのフィールドがある場合は各要素に適用
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
                        // フィールドがない場合は配列全体を返す
                        return Ok(arr.clone());
                    }
                } else {
                    return Err(Error::InvalidQuery(
                        "Cannot iterate over non-array value".into(),
                    ));
                }
            } else {
                // 通常のインデックスアクセス
                let index = index_str.parse::<usize>().map_err(|e| Error::StrToInt(e))?;

                if let Value::Array(arr) = json {
                    let item = arr.get(index).ok_or(Error::IndexOutOfBounds(index))?;

                    // 残りのフィールドがある場合
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
        }
    }

    if segment.contains('[') && segment.contains(']') {
        let (idx, ridx) = parse_array_segment(segment)?;
        let key = segment
            .get(..idx)
            .ok_or(Error::InvalidQuery("Invalid segment format".into()))?;
        let index_str = segment
            .get(idx + 1..ridx)
            .ok_or(Error::InvalidQuery("Invalid bracket content".into()))?;

        if index_str.is_empty() {
            // 配列全体を返す
            // Return the entire array
            let result = handle_array_access_as_json(json, key, fields)?;
            Ok(result)
        } else {
            // 単一要素を返す
            let index = index_str.parse::<usize>().map_err(|e| Error::StrToInt(e))?;
            let result = handle_single_access_as_json(json, key, index, fields)?;
            Ok(vec![result]) // Return single elements as arrays
        }
    } else {
        let result = handle_array_access_as_json(json, segment, fields)?;
        Ok(result)
    }
}

fn handle_nested_field_access(value: &Value, fields: Vec<&str>) -> Result<Vec<Value>, Error> {
    let mut current = value;

    for field in fields {
        if field.contains('[') && field.contains(']') {
            let (idx, ridx) = parse_array_segment(field)?;
            let field_key = field
                .get(..idx)
                .ok_or(Error::InvalidQuery("Invalid field".into()))?;
            let index_str = field
                .get(idx + 1..ridx)
                .ok_or(Error::InvalidQuery("Invalid bracket content".into()))?;
            let index = index_str.parse::<usize>().map_err(|e| Error::StrToInt(e))?;

            let array = current.get(field_key).ok_or(Error::InvalidQuery(format!(
                "Field '{}' not found",
                field_key
            )))?;
            current = array.get(index).ok_or(Error::IndexOutOfBounds(index))?;
        } else {
            current = current
                .get(field)
                .ok_or(Error::InvalidQuery(format!("Field '{}' not found", field)))?;
        }
    }

    Ok(vec![current.clone()])
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

    for field in fields {
        if field.contains('[') && field.contains(']') {
            // 配列アクセスの場合
            // In the case of array access
            let (idx, ridx) = parse_array_segment(field)?;
            let field_key = field
                .get(..idx)
                .ok_or(Error::InvalidQuery("Invalid field".into()))?;
            let index_str = field
                .get(idx + 1..ridx)
                .ok_or(Error::InvalidQuery("Invalid bracket content".into()))?;
            let field_index = index_str.parse::<usize>().map_err(|e| Error::StrToInt(e))?;

            // field_key でアクセスしてから、field_index でアクセス
            // Access with field_key, then access with field_index
            let array = current.get(field_key).ok_or(Error::InvalidQuery(format!(
                "Field '{}' not found",
                field_key
            )))?;
            current = array
                .get(field_index)
                .ok_or(Error::IndexOutOfBounds(field_index))?;
        } else {
            // 通常のフィールドアクセス
            // Normal field access
            current = current
                .get(field)
                .ok_or(Error::InvalidQuery(format!("Field '{}' not found", field)))?;
        }
    }

    // value_to_stringではなく、Valueのまま返す
    // Return Value as is, rather than value_to_string.
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
            // 各配列要素に対してフィールドパスを辿る（handle_array_accessと同じロジック）
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

            // value_to_stringではなく、Valueのまま返す
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
    // Get the first array element
    let values = json
        .get(key)
        .ok_or(Error::InvalidQuery(format!("Key '{}' not found", key)))?;
    let mut current = values.get(index).ok_or(Error::IndexOutOfBounds(index))?;

    // fieldsを順次辿る
    // Traverse fields sequentially
    for field in fields {
        if field.contains('[') && field.contains(']') {
            // 配列アクセスの場合
            // In the case of array access
            let (idx, ridx) = parse_array_segment(field)?;
            let field_key = field
                .get(..idx)
                .ok_or(Error::InvalidQuery("Invalid field".into()))?;
            let index_str = field
                .get(idx + 1..ridx)
                .ok_or(Error::InvalidQuery("Invalid bracket content".into()))?;
            let field_index = index_str.parse::<usize>().map_err(|e| Error::StrToInt(e))?;

            // field_key でアクセスしてから、field_index でアクセス
            // Access with field_key, then access with field_index
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
            // Trace the field path for each array element
            let mut current = array_item;

            for field in &fields {
                if field.contains('[') && field.contains(']') {
                    // 配列アクセスの場合
                    // In the case of array access
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
                    // Skip this element in case of error
                    return None;
                } else {
                    // 通常のフィールドアクセス
                    // Normal field access
                    if let Some(next) = current.get(field) {
                        current = next;
                    } else {
                        // フィールドが存在しない場合はこの要素をスキップ
                        // Skip this element if the field does not exist.
                        return None;
                    }
                }
            }

            Some(value_to_string(current))
        })
        .collect();

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn create_test_json() -> Value {
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

    #[test]
    fn test_handle_array_access_normal_case() {
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

    fn create_nested_test_json() -> Value {
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
}
