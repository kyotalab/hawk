use indexmap::IndexSet;
use serde_json::Value;

use crate::{apply_simple_filter, parse_array_segment, parse_query_segments, value_to_string, Error, OutputFormat};

pub fn execute_query(json: &Value, query: &str, format: OutputFormat) -> Result<(), Error> {
    let result_data = if query.contains('|') {
        // パイプライン処理
        let parts: Vec<&str> = query.split('|').map(|s| s.trim()).collect();
        let data_query = parts[0];
        let filter_query = parts[1];

        let data = execute_basic_query_as_json(json, data_query)?;

        apply_simple_filter(data, filter_query)?
    } else {
        execute_basic_query_as_json(json, query)?
    };

    format_output(&result_data, format)?;
    Ok(())
}

fn format_output(data: &[Value], format: OutputFormat) -> Result<(), Error> {
    if data.is_empty() {
        println!("data not found");
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            // 明示的にJSON出力
            print_as_json(data)?;
        }
        OutputFormat::Table => {
            // 明示的にテーブル出力（可能な場合）
            if is_object_array(data) {
                print_as_table(data);
            } else {
                let flattened = flatten_nested_arrays(data);
                if is_object_array(&flattened) {
                    print_as_table(&flattened);
                } else {
                    return Err(Error::InvalidQuery("Cannot display as table: data is not object array".into()));
                }
            }
        }
        OutputFormat::List => {
            // 明示的にリスト出力
            print_as_list(data);
        }
        OutputFormat::Auto => {
            // 既存のスマート判定ロジック
            match analyze_data_structure(data) {
                DataType::SimpleList => print_as_list(data),
                DataType::ObjectArray => print_as_table(data),
                DataType::NestedArray => {
                    let flattened = flatten_nested_arrays(data);
                    if is_object_array(&flattened) {
                        print_as_table(&flattened);
                    } else if is_simple_values(&flattened) {
                        print_as_list(&flattened);
                    } else {
                        print_as_json(data)?;
                    }
                }
                DataType::Mixed => print_as_json(data)?,
            }
        }
    }
    
    Ok(())
}


#[derive(Debug)]
enum DataType {
    SimpleList,    // [string, number, bool]
    ObjectArray,   // [{"name": "Alice"}, {"name": "Bob"}]
    NestedArray,   // [[{"name": "Project1"}], [{"name": "Project2"}]]
    Mixed,         // その他の複雑な構造
}

fn analyze_data_structure(data: &[Value]) -> DataType {
    if is_simple_values(data) {
        return DataType::SimpleList;
    }
    
    if is_object_array(data) {
        return DataType::ObjectArray;
    }
    
    // ネストした配列かチェック
    if data.len() == 1 && data[0].is_array() {
        return DataType::NestedArray;
    }
    
    DataType::Mixed
}

fn flatten_nested_arrays(data: &[Value]) -> Vec<Value> {
    let mut flattened = Vec::new();
    
    for item in data {
        match item {
            Value::Array(arr) => {
                // 配列の中身を展開
                flattened.extend(arr.iter().cloned());
            }
            _ => {
                flattened.push(item.clone());
            }
        }
    }
    
    flattened
}

fn is_simple_values(data: &[Value]) -> bool {
    data.iter().all(|v| matches!(v, Value::String(_) | Value::Number(_) | Value::Bool(_)))
}

fn is_object_array(data: &[Value]) -> bool {
    data.iter().all(|v| v.is_object())
}

fn print_as_list(data: &[Value]) {
    data.iter().for_each(|item| {
        println!("{}", value_to_string(item));
    });

}

fn print_as_table(data: &[Value]) {
    if data.is_empty() {
        return;
    }
    
    // 1. 全オブジェクトからフラット化されたフィールド名を収集
    let mut all_fields = IndexSet::new();
    for item in data {
        collect_flattened_fields_ordered(item, "", &mut all_fields);
    }
    
    let fields: Vec<String> = all_fields.into_iter().collect();
    
    // 2. 各列の最大幅を計算
    let mut max_widths = vec![0; fields.len()];
    
    // ヘッダーの幅
    for (i, field) in fields.iter().enumerate() {
        max_widths[i] = field.len();
    }
    
    // データの幅
    for item in data {
        for (i, field) in fields.iter().enumerate() {
            let value_str = get_flattened_value(item, field);
            max_widths[i] = max_widths[i].max(value_str.len());
        }
    }
    
    // 3. ヘッダー出力
    for (i, field) in fields.iter().enumerate() {
        print!("{:<width$}", field, width = max_widths[i]);
        if i < fields.len() - 1 {
            print!("  ");
        }
    }
    println!();
    
    // 4. データ行出力
    for item in data {
        for (i, field) in fields.iter().enumerate() {
            let value_str = get_flattened_value(item, field);
            print!("{:<width$}", value_str, width = max_widths[i]);
            if i < fields.len() - 1 {
                print!("  ");
            }
        }
        println!();
    }
}

fn collect_flattened_fields_ordered(value: &Value, prefix: &str, fields: &mut IndexSet<String>) {
    match value {
        Value::Object(obj) => {
            for (key, val) in obj {  // serde_json::Mapは順序を保持
                let field_name = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                
                match val {
                    Value::Object(_) => {
                        collect_flattened_fields_ordered(val, &field_name, fields);
                    }
                    _ => {
                        fields.insert(field_name);
                    }
                }
            }
        }
        _ => {
            if !prefix.is_empty() {
                fields.insert(prefix.to_string());
            }
        }
    }
}

// フラット化されたフィールドの値を取得
fn get_flattened_value(item: &Value, field_path: &str) -> String {
    let parts: Vec<&str> = field_path.split('.').collect();
    let mut current = item;
    
    for part in parts {
        match current.get(part) {
            Some(val) => current = val,
            None => return "".to_string(),
        }
    }
    
    match current {
        Value::Array(arr) => {
            // 配列は簡略表示
            format!("[{} items]", arr.len())
        }
        _ => value_to_string(current)
    }
}

fn print_as_json(data: &[Value]) -> Result<(), Error> {
    let json = serde_json::to_string_pretty(data).map_err(|e| Error::Json(e))?;

    println!("{}", json);
    Ok(())
}


pub fn execute_basic_query(json: &Value, query: &str) -> Result<Vec<String>, Error> {
    let (segment, fields) = parse_query_segments(query)?;
    // debug
    // println!("{}", segment);
    // println!("{}", param);

    if segment.contains('[') && segment.contains(']') {
        let (idx, ridx) = parse_array_segment(segment)?;        // debug
        // println!("{}", idx);
        // println!("{}", ridx);

        let key = segment.get(..idx).ok_or(Error::InvalidQuery("Invalid segment format".into()))?;
        let index_str = segment.get(idx + 1..ridx).ok_or(Error::InvalidQuery("Invalid bracket content".into()))?;
        
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

    if segment.contains('[') && segment.contains(']') {
        let (idx, ridx) = parse_array_segment(segment)?;
        let key = segment.get(..idx).ok_or(Error::InvalidQuery("Invalid segment format".into()))?;
        let index_str = segment.get(idx + 1..ridx).ok_or(Error::InvalidQuery("Invalid bracket content".into()))?;
        
        if index_str.is_empty() {
            // 配列全体を返す
            let result = handle_array_access_as_json(json, key, fields)?;
            Ok(result)
        } else {
            // 単一要素を返す
            let index = index_str.parse::<usize>().map_err(|e| Error::StrToInt(e))?;
            let result = handle_single_access_as_json(json, key, index, fields)?;
            Ok(vec![result]) // 単一要素も配列として返す
        }
    } else {
        let result = handle_array_access_as_json(json, segment, fields)?;
        Ok(result)
    }
}

pub fn handle_single_access_as_json(json: &Value, key: &str, index: usize, fields: Vec<&str>) -> Result<Value, Error> {
    let values = json.get(key).ok_or(Error::InvalidQuery(format!("Key '{}' not found", key)))?;
    let mut current = values.get(index).ok_or(Error::IndexOutOfBounds(index))?;

    // fieldsを順次辿る（handle_single_accessと同じロジック）
    for field in fields {
        if field.contains('[') && field.contains(']') {
            // 配列アクセスの場合
            let (idx, ridx) = parse_array_segment(field)?;
            let field_key = field.get(..idx).ok_or(Error::InvalidQuery("Invalid field".into()))?;
            let index_str = field.get(idx + 1..ridx).ok_or(Error::InvalidQuery("Invalid bracket content".into()))?;
            let field_index = index_str.parse::<usize>().map_err(|e| Error::StrToInt(e))?;
            
            // field_key でアクセスしてから、field_index でアクセス
            let array = current.get(field_key).ok_or(Error::InvalidQuery(format!("Field '{}' not found", field_key)))?;
            current = array.get(field_index).ok_or(Error::IndexOutOfBounds(field_index))?;
        } else {
            // 通常のフィールドアクセス
            current = current.get(field).ok_or(Error::InvalidQuery(format!("Field '{}' not found", field)))?;
        }
    }

    // value_to_stringではなく、Valueのまま返す
    Ok(current.clone())
}

pub fn handle_array_access_as_json(json: &Value, key: &str, fields: Vec<&str>) -> Result<Vec<Value>, Error> {
    let values = json.get(key).ok_or(Error::InvalidQuery(format!("Key '{}' not found", key)))?;
    let values_arr = values.as_array().ok_or(Error::InvalidQuery("Expected array".into()))?;

    let res: Vec<Value> = values_arr.iter()
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

pub fn handle_single_access(json: &Value, key: &str, index: usize, fields: Vec<&str>) -> Result<Vec<String>, Error> {
    // 1. 最初の配列要素を取得
    let values = json.get(key).ok_or(Error::InvalidQuery(format!("Key '{}' not found", key)))?;
    let mut current = values.get(index).ok_or(Error::IndexOutOfBounds(index))?;

    // 2. fieldsを順次辿る
    for field in fields {
        if field.contains('[') && field.contains(']') {
            // 配列アクセスの場合
            let (idx, ridx) = parse_array_segment(field)?;
            let field_key = field.get(..idx).ok_or(Error::InvalidQuery("Invalid field".into()))?;
            let index_str = field.get(idx + 1..ridx).ok_or(Error::InvalidQuery("Invalid bracket content".into()))?;
            let field_index = index_str.parse::<usize>().map_err(|e| Error::StrToInt(e))?;
            
            // field_key でアクセスしてから、field_index でアクセス
            let array = current.get(field_key).ok_or(Error::InvalidQuery(format!("Field '{}' not found", field_key)))?;
            current = array.get(field_index).ok_or(Error::IndexOutOfBounds(field_index))?;
        } else {
            // 通常のフィールドアクセス
            current = current.get(field).ok_or(Error::InvalidQuery(format!("Field '{}' not found", field)))?;
        }
    }

    Ok(vec![value_to_string(current)])
}

pub fn handle_array_access(json: &Value, key: &str, fields: Vec<&str>) -> Result<Vec<String>, Error> {
    let values = json.get(key).ok_or(Error::InvalidQuery(format!("Key '{}' not found", key)))?;
    let values_arr = values.as_array().ok_or(Error::InvalidQuery("Expected array".into()))?;

    let res: Vec<String> = values_arr.iter()
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


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

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

//    #[test]
//    fn test_execute_query_end_to_end() {
//        let json = create_test_json();
//        let result = execute_query(&json, ".users[0].name");
//        assert_eq!(result.unwrap(), vec!["Alice"]);
//    }

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

//    #[test]
//    fn test_execute_query_simple() {
//        let json = create_nested_test_json();
//        let result = execute_query(&json, ".users[0].name");
//        assert!(result.is_ok());
//        assert_eq!(result.unwrap(), vec!["Alice"]);
//    }
//
//    #[test]
//    fn test_execute_query_array_access() {
//        let json = create_nested_test_json();
//        let result = execute_query(&json, ".users.name");
//        assert!(result.is_ok());
//        assert_eq!(result.unwrap(), vec!["Alice", "Bob"]);
//    }
//
//    #[test]
//    fn test_execute_query_deep_nesting() {
//        let json = create_nested_test_json();
//        let result = execute_query(&json, ".users[0].projects[0].name");
//        assert!(result.is_ok());
//        assert_eq!(result.unwrap(), vec!["Project A"]);
//    }
//
//    #[test]
//    fn test_execute_query_nested_object_array() {
//        let json = create_nested_test_json();
//        let result = execute_query(&json, ".users.projects[0].name");
//        assert!(result.is_ok());
//        assert_eq!(result.unwrap(), vec!["Project A", "Project C"]);
//    }
}
