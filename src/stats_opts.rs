use serde_json::Value;
use crate::Error;

/// 統計操作を適用する
pub fn apply_stats_operation(data: &[Value], operation: &str, field: Option<&str>) -> Result<Value, Error> {
    match operation {
        "unique" => apply_unique(data, field),
        "sort" => apply_sort(data, field),
        "median" => apply_median(data, field),
        "stddev" => apply_stddev(data, field),
        "length" => Ok(Value::Number(serde_json::Number::from(data.len()))),
        _ => Err(Error::StringOperation(format!("Unknown stats operation: {}", operation))),
    }
}

/// ユニーク値を取得
fn apply_unique(data: &[Value], field: Option<&str>) -> Result<Value, Error> {
    use std::collections::HashSet;
    
    let mut unique_values = HashSet::new();
    let mut result = Vec::new();
    
    for item in data {
        let value_to_check = if let Some(field_name) = field {
            // フィールド指定がある場合
            item.get(field_name).unwrap_or(&Value::Null).clone()
        } else {
            // フィールド指定がない場合は値そのもの
            item.clone()
        };
        
        // JSON値をハッシュ可能な文字列に変換
        let key = serde_json::to_string(&value_to_check).unwrap_or_default();
        
        if unique_values.insert(key) {
            result.push(value_to_check);
        }
    }
    
    Ok(Value::Array(result))
}

/// ソート
fn apply_sort(data: &[Value], field: Option<&str>) -> Result<Value, Error> {
    let mut sorted_data = data.to_vec();
    
    sorted_data.sort_by(|a, b| {
        let val_a = if let Some(field_name) = field {
            a.get(field_name).unwrap_or(&Value::Null)
        } else {
            a
        };
        
        let val_b = if let Some(field_name) = field {
            b.get(field_name).unwrap_or(&Value::Null)
        } else {
            b
        };
        
        compare_json_values(val_a, val_b)
    });
    
    Ok(Value::Array(sorted_data))
}

/// 中央値を計算
fn apply_median(data: &[Value], field: Option<&str>) -> Result<Value, Error> {
    let mut numbers = extract_numbers(data, field)?;
    
    if numbers.is_empty() {
        return Ok(Value::Null);
    }
    
    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    
    let len = numbers.len();
    let median = if len % 2 == 0 {
        // 偶数個の場合は中央2つの平均
        (numbers[len / 2 - 1] + numbers[len / 2]) / 2.0
    } else {
        // 奇数個の場合は中央値
        numbers[len / 2]
    };
    
    Ok(Value::Number(serde_json::Number::from_f64(median).unwrap()))
}

/// 標準偏差を計算
fn apply_stddev(data: &[Value], field: Option<&str>) -> Result<Value, Error> {
    let numbers = extract_numbers(data, field)?;
    
    if numbers.len() < 2 {
        return Ok(Value::Null);
    }
    
    let mean = numbers.iter().sum::<f64>() / numbers.len() as f64;
    let variance = numbers.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / (numbers.len() - 1) as f64; // 標本標準偏差
    
    let stddev = variance.sqrt();
    
    Ok(Value::Number(serde_json::Number::from_f64(stddev).unwrap()))
}

/// 数値を抽出
fn extract_numbers(data: &[Value], field: Option<&str>) -> Result<Vec<f64>, Error> {
    let mut numbers = Vec::new();
    
    for item in data {
        let value = if let Some(field_name) = field {
            item.get(field_name).unwrap_or(&Value::Null)
        } else {
            item
        };
        
        if let Some(num) = value.as_f64() {
            numbers.push(num);
        }
    }
    
    Ok(numbers)
}

/// JSON値の比較
fn compare_json_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    
    match (a, b) {
        (Value::Number(n1), Value::Number(n2)) => {
            n1.as_f64().unwrap_or(0.0).partial_cmp(&n2.as_f64().unwrap_or(0.0)).unwrap_or(Ordering::Equal)
        },
        (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
        (Value::Bool(b1), Value::Bool(b2)) => b1.cmp(b2),
        (Value::Null, Value::Null) => Ordering::Equal,
        (Value::Null, _) => Ordering::Less,
        (_, Value::Null) => Ordering::Greater,
        // 異なる型の場合は型名で比較
        _ => get_type_priority(a).cmp(&get_type_priority(b)),
    }
}

/// 型の優先順位
fn get_type_priority(value: &Value) -> u8 {
    match value {
        Value::Null => 0,
        Value::Bool(_) => 1,
        Value::Number(_) => 2,
        Value::String(_) => 3,
        Value::Array(_) => 4,
        Value::Object(_) => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_operation() {
        let data = vec![
            Value::String("apple".to_string()),
            Value::String("banana".to_string()),
            Value::String("apple".to_string()),
            Value::String("cherry".to_string()),
        ];
        
        let result = apply_unique(&data, None).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3); // apple, banana, cherry
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_sort_numbers() {
        let data = vec![
            Value::Number(3.into()),
            Value::Number(1.into()),
            Value::Number(4.into()),
            Value::Number(2.into()),
        ];
        
        let result = apply_sort(&data, None).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr[0], Value::Number(1.into()));
            assert_eq!(arr[1], Value::Number(2.into()));
            assert_eq!(arr[2], Value::Number(3.into()));
            assert_eq!(arr[3], Value::Number(4.into()));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_median_even() {
        let data = vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
            Value::Number(4.into()),
            Value::Number(5.into()),
        ];
        
        let result = apply_median(&data, None).unwrap();
        assert_eq!(result, Value::Number(serde_json::Number::from_f64(3.0).unwrap()));
    }

    #[test]
    fn test_stddev() {
        let data = vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
            Value::Number(3.into()),
            Value::Number(4.into()),
            Value::Number(5.into()),
        ];
        
        let result = apply_stddev(&data, None).unwrap();
        // 標本標準偏差 ≈ 1.58
        if let Value::Number(n) = result {
            let stddev = n.as_f64().unwrap();
            assert!((stddev - 1.58).abs() < 0.1);
        } else {
            panic!("Expected number result");
        }
    }

    #[test]
    fn test_unique_with_field() {
        let data = vec![
            serde_json::json!({"name": "Alice", "age": 30}),
            serde_json::json!({"name": "Bob", "age": 25}),
            serde_json::json!({"name": "Alice", "age": 35}),
        ];
        
        let result = apply_unique(&data, Some("name")).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2); // Alice, Bob
        } else {
            panic!("Expected array result");
        }
    }
}
