use serde_json::Value;

use crate::{print_data_info, value_to_string, Error};


pub fn apply_simple_filter(data: Vec<Value>, filter: &str) -> Result<Vec<Value>, Error> {
    if filter.starts_with("select(") && filter.ends_with(")") {
        // "select(.age > 30)" から ".age > 30" を抽出
        let condition = &filter[7..filter.len()-1];

        // 条件をパース
        let (field_path, operator, value) = parse_condition(condition)?;

        // フィルタリングを実行
        let filtered: Vec<Value> = data.into_iter()
            .filter(|item| evaluate_condition(item, &field_path, &operator, &value))
            .collect();

        Ok(filtered)
    } else {
        Err(Error::InvalidQuery(format!("Unsupported filter: {}", filter)))
    }
}

pub fn apply_pipeline_operation(data: Vec<Value>, operation: &str) -> Result<Vec<Value>, Error> {
    if operation.starts_with("select(") && operation.ends_with(")") {
        // フィルタリング操作
        apply_simple_filter(data, operation)
    } else if operation == "count" {
        // カウント操作
        if is_grouped_data(&data) {
            apply_aggregation_to_groups(data, "count", "")
        } else {
            let count = data.len();
            let count_value = Value::Number(serde_json::Number::from(count));
            Ok(vec![count_value])
        }
    } else if operation == "info" {
        // info操作
        print_data_info(&data);
        Ok(vec![]) // 空のVecを返す
    } else if operation.starts_with("sum(") && operation.ends_with(")") {
        // sum(.field) の処理
        let field = &operation[4..operation.len()-1];
        let field_name = field.trim_start_matches('.');

        if is_grouped_data(&data) {
            apply_aggregation_to_groups(data, "sum", field_name)
        } else {
            let sum: f64 = data.iter()
                .filter_map(|item| item.get(field_name))
                .filter_map(|val| val.as_f64())
                .sum();

            let round_sum = if sum.fract() == 0.0 {
                sum
            } else {
                (sum * 10.0).round() / 10.0
            };
            let sum_value = Value::Number(serde_json::Number::from_f64(round_sum).unwrap());
            Ok(vec![sum_value])
        }
    } else if operation.starts_with("avg(") && operation.ends_with(")") {
        // avg(.field) の処理
        let field = &operation[4..operation.len()-1];
        let field_name = field.trim_start_matches('.');

        if is_grouped_data(&data) {
            apply_aggregation_to_groups(data, "avg", field_name)
        } else {
            let values: Vec<f64> = data.iter()
                .filter_map(|item| item.get(field_name))
                .filter_map(|val| val.as_f64())
                .collect();

            if values.is_empty() {
                Ok(vec![Value::Null])
            } else {
                let avg = values.iter().sum::<f64>() / values.len() as f64;
                let round_avg = (avg * 10.0).round() / 10.0;
                let avg_value = Value::Number(serde_json::Number::from_f64(round_avg).unwrap());
                Ok(vec![avg_value])
            }
        }
    } else if operation.starts_with("min(") && operation.ends_with(")") {
        // min(.field) の処理
        let field = &operation[4..operation.len()-1];
        let field_name = field.trim_start_matches('.');

        if is_grouped_data(&data) {
            apply_aggregation_to_groups(data, "min", field_name)
        } else {
            let min_val = data.iter()
                .filter_map(|item| item.get(field_name))
                .filter_map(|val| val.as_f64())
                .fold(f64::INFINITY, f64::min);

            if min_val == f64::INFINITY {
                Ok(vec![Value::Null])
            } else {
                let min_value = Value::Number(serde_json::Number::from_f64(min_val).unwrap());
                Ok(vec![min_value])
            }
        }
    } else if operation.starts_with("max(") && operation.ends_with(")") {
        // max(.field) の処理
        let field = &operation[4..operation.len()-1];
        let field_name = field.trim_start_matches('.');

        if is_grouped_data(&data) {
            apply_aggregation_to_groups(data, "max", field_name)
        } else {
            let max_val = data.iter()
                .filter_map(|item| item.get(field_name))
                .filter_map(|val| val.as_f64())
                .fold(f64::NEG_INFINITY, f64::max);

            if max_val == f64::NEG_INFINITY {
                Ok(vec![Value::Null])
            } else {
                let max_value = Value::Number(serde_json::Number::from_f64(max_val).unwrap());
                Ok(vec![max_value])
            }
        }
    } else if operation.starts_with("group_by(") && operation.ends_with(")") {
        // group_by(.department) の処理
        let field = &operation[9..operation.len()-1];
        let field_name = field.trim_start_matches('.');

        let grouped = group_data_by_field(data, field_name)?;
        Ok(grouped)
    } else {
        Err(Error::InvalidQuery(format!("Unsupported operation: {}", operation)))
    }
}


fn group_data_by_field(data: Vec<Value>, field_name: &str) -> Result<Vec<Value>, Error> {
    use std::collections::HashMap;

    let mut groups: HashMap<String, Vec<Value>> = HashMap::new();

    // データをフィールド値でグルーピング
    for item in data {
        if let Some(field_value) = item.get(field_name) {
            let key = value_to_string(field_value);
            groups.entry(key).or_insert_with(Vec::new).push(item);
        }
    }

    // グループを配列として返す
    let result: Vec<Value> = groups.into_iter()
        .map(|(group_name, group_items)| {
            let mut group_obj = serde_json::Map::new();
            group_obj.insert("group".to_string(), Value::String(group_name));
            group_obj.insert("items".to_string(), Value::Array(group_items));
            Value::Object(group_obj)
        })
        .collect();

    Ok(result)
}

// 条件をパースする関数
fn parse_condition(condition: &str) -> Result<(String, String, String), Error> {
    // ".age > 30" のような条件をパース
    let condition = condition.trim();

    // 演算子を検出
    if let Some(pos) = condition.find(" > ") {
        let field = condition[..pos].trim().to_string();
        let value = condition[pos + 3..].trim().to_string();
        return Ok((field, ">".to_string(), value));
    }

    if let Some(pos) = condition.find(" < ") {
        let field = condition[..pos].trim().to_string();
        let value = condition[pos + 3..].trim().to_string();
        return Ok((field, "<".to_string(), value));
    }

    if let Some(pos) = condition.find(" == ") {
        let field = condition[..pos].trim().to_string();
        let value = condition[pos + 4..].trim().to_string();
        return Ok((field, "==".to_string(), value));
    }

    if let Some(pos) = condition.find(" != ") {
        let field = condition[..pos].trim().to_string();
        let value = condition[pos + 4..].trim().to_string();
        return Ok((field, "!=".to_string(), value));
    }

    Err(Error::InvalidQuery("Invalid condition format".into()))
}

// 条件を評価する関数
fn evaluate_condition(item: &Value, field_path: &str, operator: &str, value: &str) -> bool {
    // フィールドパスから値を取得 (.age -> age)
    let field_name = if field_path.starts_with('.') {
        &field_path[1..]
    } else {
        field_path
    };

    let field_value = match item.get(field_name) {
        Some(val) => val,
        None => return false, // フィールドが存在しない場合はfalse
    };

    match operator {
        ">" => compare_greater(field_value, value),
        "<" => compare_less(field_value, value),
        "==" => compare_equal(field_value, value),
        "!=" => !compare_equal(field_value, value),
        _ => false,
    }
}

// 比較関数
fn compare_greater(field_value: &Value, target: &str) -> bool {
    match field_value {
        Value::Number(n) => {
            if let Ok(target_num) = target.parse::<f64>() {
                n.as_f64().unwrap_or(0.0) > target_num
            } else {
                false
            }
        }
        _ => false,
    }
}

fn compare_less(field_value: &Value, target: &str) -> bool {
    match field_value {
        Value::Number(n) => {
            if let Ok(target_num) = target.parse::<f64>() {
                n.as_f64().unwrap_or(0.0) < target_num
            } else {
                false
            }
        }
        _ => false,
    }
}

fn compare_equal(field_value: &Value, target: &str) -> bool {
    match field_value {
        Value::String(s) => {
            // 文字列比較（引用符を除去）
            let target_clean = target.trim_matches('"');
            s == target_clean
        }
        Value::Number(n) => {
            if let Ok(target_num) = target.parse::<f64>() {
                n.as_f64().unwrap_or(0.0) == target_num
            } else {
                false
            }
        }
        Value::Bool(b) => {
            match target {
                "true" => *b,
                "false" => !*b,
                _ => false,
            }
        }
        _ => false,
    }
}

fn is_grouped_data(data: &[Value]) -> bool {
    data.iter().all(|item| {
        if let Value::Object(obj) = item {
            obj.contains_key("group") && obj.contains_key("items")
        } else {
            false
        }
    })
}

fn apply_aggregation_to_groups(data: Vec<Value>, operation: &str, field_name: &str) -> Result<Vec<Value>, Error> {
    let mut results = Vec::new();
    
    for group_data in data {
        if let Value::Object(group_obj) = group_data {
            let group_name = group_obj.get("group").unwrap();
            let items = group_obj.get("items").and_then(|v| v.as_array()).unwrap();
            
            // 各グループのitemsに対して集約を実行
            let aggregated_value = match operation {
                "avg" => calculate_avg(items, field_name)?,
                "sum" => calculate_sum(items, field_name)?,
                "count" => Value::Number(serde_json::Number::from(items.len())),
                "min" => calculate_min(items, field_name)?,
                "max" => calculate_max(items, field_name)?,
                _ => Value::Null,
            };
            
            // 結果オブジェクトを作成
            let mut result_obj = serde_json::Map::new();
            result_obj.insert("group".to_string(), group_name.clone());
            result_obj.insert(operation.to_string(), aggregated_value);
            results.push(Value::Object(result_obj));
        }
    }
    
    Ok(results)
}

fn calculate_avg(items: &[Value], field_name: &str) -> Result<Value, Error> {
    let values: Vec<f64> = items.iter()
        .filter_map(|item| item.get(field_name))
        .filter_map(|val| val.as_f64())
        .collect();
    
    if values.is_empty() {
        Ok(Value::Null)
    } else {
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        let rounded_avg = (avg * 10.0).round() / 10.0;
        Ok(Value::Number(serde_json::Number::from_f64(rounded_avg).unwrap()))
    }
}

fn calculate_sum(items: &[Value], field_name: &str) -> Result<Value, Error> {
    let sum: f64 = items.iter()
        .filter_map(|item| item.get(field_name))
        .filter_map(|val| val.as_f64())
        .sum();
    
    let rounded_sum = if sum.fract() == 0.0 {
        sum
    } else {
        (sum * 10.0).round() / 10.0
    };
    
    Ok(Value::Number(serde_json::Number::from_f64(rounded_sum).unwrap()))
}

fn calculate_min(items: &[Value], field_name: &str) -> Result<Value, Error> {
    let min_val = items.iter()
        .filter_map(|item| item.get(field_name))
        .filter_map(|val| val.as_f64())
        .fold(f64::INFINITY, f64::min);
    
    if min_val == f64::INFINITY {
        Ok(Value::Null)
    } else {
        Ok(Value::Number(serde_json::Number::from_f64(min_val).unwrap()))
    }
}

fn calculate_max(items: &[Value], field_name: &str) -> Result<Value, Error> {
    let max_val = items.iter()
        .filter_map(|item| item.get(field_name))
        .filter_map(|val| val.as_f64())
        .fold(f64::NEG_INFINITY, f64::max);
    
    if max_val == f64::NEG_INFINITY {
        Ok(Value::Null)
    } else {
        Ok(Value::Number(serde_json::Number::from_f64(max_val).unwrap()))
    }
}
