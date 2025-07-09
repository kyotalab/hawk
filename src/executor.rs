use serde_json::Value;

use crate::{parse_array_segment, parse_query_segments, value_to_string, Error};


pub fn execute_query(json: &Value, query: &str) -> Result<Vec<String>, Error> {
    let (segment, field) = parse_query_segments(query)?;
    // debug
    // println!("{}", segment);
    // println!("{}", param);

    if segment.contains('[') && segment.contains(']') {
        let (idx, ridx) = parse_array_segment(segment)?;        // debug
        // println!("{}", idx);
        // println!("{}", ridx);

        let key = segment.get(..idx).ok_or(Error::InvalidQuery("Invalid segment format".into()))?;
        let index_str = segment.get(idx + 1..ridx).ok_or(Error::InvalidQuery("Invalid bracket content".into()))?;
        let index = index_str.parse::<usize>().map_err(|e| Error::StrToInt(e))?;


        let result = handle_single_access(json, key, index, field)?;
        // debug
        // println!("{:?}", json_key);
        // println!("{:?}", index);

        Ok(result)

    } else {
        let key = segment;

        let result = handle_array_access(json, key, field)?;

        Ok(result)
    }
}

pub fn handle_single_access(json: &Value, key: &str, index: usize, field: &str) -> Result<Vec<String>, Error> {
    let values = json.get(key).ok_or(Error::InvalidQuery(format!("Key '{}' not found", key)))?;
    let item = values.get(index).ok_or(Error::IndexOutOfBounds(index))?;
    let res = item.get(field).ok_or(Error::InvalidQuery(format!("Field '{}' not found", field)))?;

    Ok(vec![value_to_string(res)])
}

pub fn handle_array_access(json: &Value, key: &str, field: &str) -> Result<Vec<String>, Error> {
    let values = json.get(key).ok_or(Error::InvalidQuery(format!("Key '{}' not found", key)))?;
    let values_arr = values.as_array().ok_or(Error::InvalidQuery("Expected array".into()))?;

    // debug
    // println!("{:?}", values_arr);

    let res: Vec<_> = values_arr.iter()
        .filter_map(|value| value.get(field))
        .collect();
    
    let res: Vec<String> = res.into_iter()
        .map(|v| value_to_string(v))
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
        let result = handle_array_access(&json, "users", "name");
        
        assert!(result.is_ok());
        let names = result.unwrap();
        assert_eq!(names, vec!["Alice", "Bob", "Carol"]);
    }

    #[test]
    fn test_handle_array_access_with_missing_field() {
        // 正常ケース: 一部の要素にフィールドがない（filter_mapで対応）
        let json = create_test_json();
        let result = handle_array_access(&json, "users", "active");
        
        assert!(result.is_ok());
        let actives = result.unwrap();
        assert_eq!(actives, vec!["true", "false"]); // Carolにはactiveフィールドがない
    }

    #[test]
    fn test_handle_array_access_different_types() {
        // 正常ケース: 異なる型のフィールド
        let json = create_test_json();
        let result = handle_array_access(&json, "users", "age");
        
        assert!(result.is_ok());
        let ages = result.unwrap();
        assert_eq!(ages, vec!["30", "25", "35"]);
    }

    #[test]
    fn test_handle_array_access_empty_array() {
        // 正常ケース: 空の配列
        let json = create_test_json();
        let result = handle_array_access(&json, "empty_array", "name");
        
        assert!(result.is_ok());
        let names = result.unwrap();
        assert!(names.is_empty());
    }

    #[test]
    fn test_handle_array_access_key_not_found() {
        // エラーケース: 存在しないキー
        let json = create_test_json();
        let result = handle_array_access(&json, "nonexistent", "name");
        
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
        let result = handle_array_access(&json, "not_array", "name");

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
        let result = handle_array_access(&json, "users", "nonexistent_field");
        
        assert!(result.is_ok());
        let values = result.unwrap();
        assert!(values.is_empty()); // filter_mapで空になる
    }

    #[test]
    fn test_handle_single_access_normal_case() {
        // 正常ケース: 基本的な単一要素アクセス
        let json = create_test_json();
        let result = handle_single_access(&json, "users", 0, "name");
        
        assert!(result.is_ok());
        let names = result.unwrap();
        assert_eq!(names, vec!["Alice"]);
    }

    #[test]
    fn test_handle_single_access_different_index() {
        // 正常ケース: 異なるインデックス
        let json = create_test_json();
        let result = handle_single_access(&json, "users", 1, "name");
        
        assert!(result.is_ok());
        let names = result.unwrap();
        assert_eq!(names, vec!["Bob"]);
    }

    #[test]
    fn test_handle_single_access_different_field() {
        // 正常ケース: 異なるフィールド
        let json = create_test_json();
        let result = handle_single_access(&json, "users", 0, "age");
        
        assert!(result.is_ok());
        let ages = result.unwrap();
        assert_eq!(ages, vec!["30"]);
    }

    #[test]
    fn test_handle_single_access_boolean_field() {
        // 正常ケース: Boolean型のフィールド
        let json = create_test_json();
        let result = handle_single_access(&json, "users", 0, "active");
        
        assert!(result.is_ok());
        let actives = result.unwrap();
        assert_eq!(actives, vec!["true"]);
    }

    #[test]
    fn test_handle_single_access_key_not_found() {
        // エラーケース: 存在しないキー
        let json = create_test_json();
        let result = handle_single_access(&json, "nonexistent", 0, "name");
        
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
        let result = handle_single_access(&json, "users", 999, "name");
        
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
        let result = handle_single_access(&json, "users", 0, "nonexistent_field");
        
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
        let result = handle_single_access(&json, "not_array", 0, "name");
        
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
        let result = handle_single_access(&json, "empty_array", 0, "name");
        
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::IndexOutOfBounds(index) => {
                assert_eq!(index, 0);
            }
            _ => panic!("Expected IndexOutOfBounds error"),
        }
    }

    #[test]
    fn test_execute_query_end_to_end() {
        let json = create_test_json();
        let result = execute_query(&json, ".users[0].name");
        assert_eq!(result.unwrap(), vec!["Alice"]);
    }
}
