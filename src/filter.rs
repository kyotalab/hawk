use serde_json::Value;

use crate::{apply_stats_operation, Error, print_data_info, value_to_string, string_ops};

pub fn apply_simple_filter(data: Vec<Value>, filter: &str) -> Result<Vec<Value>, Error> {
    if filter.starts_with("select(") && filter.ends_with(")") {
        // "select(.age > 30)" から ".age > 30" を抽出
        let condition = &filter[7..filter.len() - 1];

        // パイプラインがある場合の処理
        if condition.contains(" | ") {
            apply_filter_with_string_operations(data, condition)
        } else {
            apply_existing_simple_filter(data, condition)
        }
    } else {
        Err(Error::InvalidQuery(format!(
            "Unsupported filter: {}",
            filter
        )))
    }
}

/// 文字列操作付きフィルタの適用
fn apply_filter_with_string_operations(data: Vec<Value>, condition: &str) -> Result<Vec<Value>, Error> {
    let parts: Vec<&str> = condition.split(" | ").map(|s| s.trim()).collect();
    
    if parts.len() < 2 {
        return Err(Error::InvalidQuery("Invalid filter condition".to_string()));
    }
    
    let field_access = parts[0];
    let string_operations: Vec<&str> = parts[1..].to_vec();
    
    // 最後の操作は比較操作である必要がある
    let last_operation = string_operations.last().ok_or_else(|| {
        Error::InvalidQuery("Missing comparison operation".to_string())
    })?;
    
    if !is_comparison_operation(last_operation) {
        return Err(Error::InvalidQuery("Last operation must be a comparison".to_string()));
    }
    
    let mut results = Vec::new();
    
    for item in data {
        // フィールド値を取得
        let field_value = extract_field_value(&item, field_access)?;
        
        // 文字列操作を適用（比較操作まで）
        let final_value = string_ops::apply_string_pipeline(&field_value, &string_operations)?;
        
        // 比較結果が true の場合のみ追加
        if let Value::Bool(true) = final_value {
            results.push(item);
        }
    }
    
    Ok(results)
}

/// 比較操作かどうかを判定
fn is_comparison_operation(operation: &str) -> bool {
    operation.starts_with("contains(") ||
    operation.starts_with("starts_with(") ||
    operation.starts_with("ends_with(") ||
    operation == "==" ||
    operation == "!=" ||
    operation.starts_with("== ") ||
    operation.starts_with("!= ")
}

/// 既存のシンプルフィルタ処理
fn apply_existing_simple_filter(data: Vec<Value>, condition: &str) -> Result<Vec<Value>, Error> {
    // 条件をパース
    let (field_path, operator, value) = parse_condition(condition)?;

    // フィルタリングを実行
    let filtered: Vec<Value> = data
        .into_iter()
        .filter(|item| evaluate_condition(item, &field_path, &operator, &value))
        .collect();

    Ok(filtered)
}

pub fn apply_pipeline_operation(data: Vec<Value>, operation: &str) -> Result<Vec<Value>, Error> {
    let trimmed_op = operation.trim();
    
    if trimmed_op.starts_with("select(") && trimmed_op.ends_with(")") {
        // フィルタリング操作
        apply_simple_filter(data, trimmed_op)
    } else if trimmed_op == "count" {
        // カウント操作
        if is_grouped_data(&data) {
            apply_aggregation_to_groups(data, "count", "")
        } else {
            let count = data.len();
            let count_value = Value::Number(serde_json::Number::from(count));
            Ok(vec![count_value])
        }
    } else if trimmed_op.starts_with("map(") && trimmed_op.ends_with(")") {
        apply_map_operation(data, trimmed_op)
    } else if trimmed_op.starts_with("select_fields(") && trimmed_op.ends_with(")") {
        // 複数フィールド選択
        let fields_str = &trimmed_op[14..trimmed_op.len() - 1]; // "name,age,department"
        let field_list: Vec<String> = fields_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        apply_field_selection(data, field_list)
    } else if trimmed_op == "info" {
        // info操作
        print_data_info(&data);
        Ok(vec![]) // Return empty vector
    } else if trimmed_op.starts_with("sum(") && trimmed_op.ends_with(")") {
        // sum(.field) の処理
        let field = &trimmed_op[4..trimmed_op.len() - 1];
        let field_name = field.trim_start_matches('.');

        if is_grouped_data(&data) {
            apply_aggregation_to_groups(data, "sum", field_name)
        } else {
            let sum: f64 = data
                .iter()
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
    } else if trimmed_op.starts_with("avg(") && trimmed_op.ends_with(")") {
        // avg(.field) の処理
        let field = &trimmed_op[4..trimmed_op.len() - 1];
        let field_name = field.trim_start_matches('.');

        if is_grouped_data(&data) {
            apply_aggregation_to_groups(data, "avg", field_name)
        } else {
            let values: Vec<f64> = data
                .iter()
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
    } else if trimmed_op.starts_with("min(") && trimmed_op.ends_with(")") {
        // min(.field) の処理
        let field = &trimmed_op[4..trimmed_op.len() - 1];
        let field_name = field.trim_start_matches('.');

        if is_grouped_data(&data) {
            apply_aggregation_to_groups(data, "min", field_name)
        } else {
            let min_val = data
                .iter()
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
    } else if trimmed_op.starts_with("max(") && trimmed_op.ends_with(")") {
        // max(.field) の処理
        let field = &trimmed_op[4..trimmed_op.len() - 1];
        let field_name = field.trim_start_matches('.');

        if is_grouped_data(&data) {
            apply_aggregation_to_groups(data, "max", field_name)
        } else {
            let max_val = data
                .iter()
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
    } else if trimmed_op.starts_with("group_by(") && trimmed_op.ends_with(")") {
        // group_by(.department) の処理
        let field = &trimmed_op[9..trimmed_op.len() - 1];
        let field_name = field.trim_start_matches('.');

        let grouped = group_data_by_field(data, field_name)?;
        Ok(grouped)
    } else if trimmed_op == "unique" {
        // unique操作（重複除去）
        let result = apply_stats_operation(&data, "unique", None)?;
        if let Value::Array(arr) = result {
            Ok(arr)
        } else {
            Ok(vec![result])
        }
    } else if trimmed_op == "sort" {
        // sort操作
        let result = apply_stats_operation(&data, "sort", None)?;
        if let Value::Array(arr) = result {
            Ok(arr)
        } else {
            Ok(vec![result])
        }
    } else if trimmed_op == "length" {
        // length操作（配列の長さ）
        let result = apply_stats_operation(&data, "length", None)?;
        Ok(vec![result])
    } else if trimmed_op == "median" {
        // median操作（中央値）
        let result = apply_stats_operation(&data, "median", None)?;
        Ok(vec![result])
    } else if trimmed_op == "stddev" {
        // stddev操作（標準偏差）
        let result = apply_stats_operation(&data, "stddev", None)?;
        Ok(vec![result])
    } else if trimmed_op.starts_with("unique(") && trimmed_op.ends_with(")") {
        // unique(.field) - フィールド指定
        let field = &trimmed_op[7..trimmed_op.len() - 1];
        let field_name = field.trim_start_matches('.');
        let result = apply_stats_operation(&data, "unique", Some(field_name))?;
        if let Value::Array(arr) = result {
            Ok(arr)
        } else {
            Ok(vec![result])
        }
    } else if trimmed_op.starts_with("sort(") && trimmed_op.ends_with(")") {
        // sort(.field) - フィールド指定
        let field = &trimmed_op[5..trimmed_op.len() - 1];
        let field_name = field.trim_start_matches('.');
        let result = apply_stats_operation(&data, "sort", Some(field_name))?;
        if let Value::Array(arr) = result {
            Ok(arr)
        } else {
            Ok(vec![result])
        }
    } else if trimmed_op.starts_with("median(") && trimmed_op.ends_with(")") {
        // median(.field) - フィールド指定
        let field = &trimmed_op[7..trimmed_op.len() - 1];
        let field_name = field.trim_start_matches('.');
        let result = apply_stats_operation(&data, "median", Some(field_name))?;
        Ok(vec![result])
    } else if trimmed_op.starts_with("stddev(") && trimmed_op.ends_with(")") {
        // stddev(.field) - フィールド指定
        let field = &trimmed_op[7..trimmed_op.len() - 1];
        let field_name = field.trim_start_matches('.');
        let result = apply_stats_operation(&data, "stddev", Some(field_name))?;
        Ok(vec![result])
    } else {
        // より詳細なエラーメッセージ
        Err(Error::InvalidQuery(format!(
            "Unsupported operation: '{}' (length: {}, starts with 'map(': {}, ends with ')': {})",
            trimmed_op, 
            trimmed_op.len(),
            trimmed_op.starts_with("map("),
            trimmed_op.ends_with(")")
        )))
    }
}

/// map操作の実装
fn apply_map_operation(data: Vec<Value>, operation: &str) -> Result<Vec<Value>, Error> {
    // "map(.field | string_operation)" または "map(.field1, .field2 | operation)" の解析
    let content = &operation[4..operation.len() - 1]; // "map(" と ")" を除去
    
    // **新機能: 複数フィールド対応**
    if content.contains(',') && content.contains('|') {
        // 複数フィールドの場合: "map(.skills, .projects | join(\",\"))"
        apply_multi_field_map_operation(data, content)
    } else {
        // 単一フィールドの場合: "map(.field | operation)"
        apply_single_field_map_operation(data, content)
    }
}

/// 単一フィールドのmap操作（既存）
fn apply_single_field_map_operation(data: Vec<Value>, content: &str) -> Result<Vec<Value>, Error> {
    let (field_access, string_operations) = parse_map_content(content)?;
    
    let mut results = Vec::new();
    
    for item in data {
        // フィールドにアクセス
        let field_value = extract_field_value(&item, &field_access)?;
        
        // 文字列操作を適用
        let transformed_value = apply_string_operations(&field_value, &string_operations)?;
        
        // 元のオブジェクトを更新または新しい値を作成
        let result = update_or_create_value(&item, &field_access, transformed_value)?;
        results.push(result);
    }
    
    Ok(results)
}

/// 複数フィールドのmap操作（ケース1: 各フィールドに同じ操作）
fn apply_multi_field_map_operation(data: Vec<Value>, content: &str) -> Result<Vec<Value>, Error> {
    // "(.skills, .projects | join(\",\"))" を解析
    let parts: Vec<&str> = content.split('|').map(|s| s.trim()).collect();
    
    if parts.len() != 2 {
        return Err(Error::InvalidQuery("Multi-field map must have format: (.field1, .field2 | operation)".to_string()));
    }
    
    let fields_part = parts[0].trim();
    let operation = parts[1].trim();
    
    // フィールド部分をパース: ".skills, .projects"
    let field_paths: Vec<&str> = fields_part
        .split(',')
        .map(|s| s.trim())
        .collect();
    
    // 各フィールドパスが "." で始まることを確認
    for field_path in &field_paths {
        if !field_path.starts_with('.') {
            return Err(Error::InvalidQuery(format!("Field path must start with '.': {}", field_path)));
        }
    }
    
    let mut results = Vec::new();
    
    for item in data {
        // 各フィールドに同じ操作を適用（ケース1）
        let transformed_item = crate::string_ops::apply_operation_to_multiple_fields(&item, &field_paths, operation)?;
        results.push(transformed_item);
    }
    
    Ok(results)
}

/// map操作の内容を解析（例: ".name | upper | trim"）
fn parse_map_content(content: &str) -> Result<(String, Vec<String>), Error> {
    let parts: Vec<&str> = content.split('|').map(|s| s.trim()).collect();
    
    if parts.is_empty() {
        return Err(Error::InvalidQuery("Empty map operation".to_string()));
    }
    
    // 最初の部分はフィールドアクセス
    let field_access = parts[0].to_string();
    
    // 残りは文字列操作
    let string_operations: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
    
    Ok((field_access, string_operations))
}

/// フィールド値を抽出
fn extract_field_value(item: &Value, field_access: &str) -> Result<Value, Error> {
    if field_access == "." {
        // ルート値（Text配列の場合の各行）
        return Ok(item.clone());
    }
    
    if field_access.starts_with('.') {
        let field_name = &field_access[1..]; // '.' を除去
        
        if let Some(value) = item.get(field_name) {
            Ok(value.clone())
        } else {
            Err(Error::InvalidQuery(format!("Field '{}' not found", field_name)))
        }
    } else {
        Err(Error::InvalidQuery(format!("Invalid field access: {}", field_access)))
    }
}

/// 文字列操作を順次適用
fn apply_string_operations(value: &Value, operations: &[String]) -> Result<Value, Error> {
    if operations.is_empty() {
        return Ok(value.clone());
    }
    
    let operations_str: Vec<&str> = operations.iter().map(|s| s.as_str()).collect();
    string_ops::apply_string_pipeline(value, &operations_str)
}

/// 値を更新または新しい値を作成
fn update_or_create_value(original: &Value, field_access: &str, new_value: Value) -> Result<Value, Error> {
    if field_access == "." {
        // ルート値の場合は直接置き換え
        Ok(new_value)
    } else if field_access.starts_with('.') {
        let field_name = &field_access[1..];
        
        // オブジェクトの場合はフィールドを更新
        if let Value::Object(mut obj) = original.clone() {
            obj.insert(field_name.to_string(), new_value);
            Ok(Value::Object(obj))
        } else {
            // オブジェクトでない場合は新しいオブジェクトを作成
            let mut new_obj = serde_json::Map::new();
            new_obj.insert(field_name.to_string(), new_value);
            Ok(Value::Object(new_obj))
        }
    } else {
        Err(Error::InvalidQuery(format!("Invalid field access: {}", field_access)))
    }
}

fn apply_field_selection(data: Vec<Value>, field_list: Vec<String>) -> Result<Vec<Value>, Error> {
    let mut results = Vec::new();

    for item in data {
        if let Value::Object(obj) = item {
            let mut selected_obj = serde_json::Map::new();

            // 指定されたフィールドのみを抽出
            for field_name in &field_list {
                if let Some(value) = obj.get(field_name) {
                    selected_obj.insert(field_name.clone(), value.clone());
                }
            }

            results.push(Value::Object(selected_obj));
        } else {
            // オブジェクト以外は無視するか、エラーにする
            return Err(Error::InvalidQuery(
                "select_fields can only be applied to objects".into(),
            ));
        }
    }

    Ok(results)
}

fn group_data_by_field(data: Vec<Value>, field_name: &str) -> Result<Vec<Value>, Error> {
    use std::collections::HashMap;

    let mut groups: HashMap<String, Vec<Value>> = HashMap::new();

    // データをフィールド値でグルーピング
    for item in data {
        if let Some(field_value) = item.get(field_name) {
            let key = value_to_string(field_value);
            groups.entry(key).or_default().push(item);
        }
    }

    // グループを配列として返す
    let result: Vec<Value> = groups
        .into_iter()
        .map(|(group_name, group_items)| {
            let mut group_obj = serde_json::Map::new();
            group_obj.insert("group".to_string(), Value::String(group_name));
            group_obj.insert("items".to_string(), Value::Array(group_items));
            Value::Object(group_obj)
        })
        .collect();

    Ok(result)
}

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

fn evaluate_condition(item: &Value, field_path: &str, operator: &str, value: &str) -> bool {
    // フィールドパスから値を取得 (.age -> age)
    let field_name = if field_path.starts_with('.') {
        &field_path[1..]
    } else {
        field_path
    };

    let field_value = match item.get(field_name) {
        Some(val) => val,
        None => return false, // false if the field does not exist
    };

    match operator {
        ">" => compare_greater(field_value, value),
        "<" => compare_less(field_value, value),
        "==" => compare_equal(field_value, value),
        "!=" => !compare_equal(field_value, value),
        _ => false,
    }
}

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
        Value::Bool(b) => match target {
            "true" => *b,
            "false" => !*b,
            _ => false,
        },
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

fn apply_aggregation_to_groups(
    data: Vec<Value>,
    operation: &str,
    field_name: &str,
) -> Result<Vec<Value>, Error> {
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
    let values: Vec<f64> = items
        .iter()
        .filter_map(|item| item.get(field_name))
        .filter_map(|val| val.as_f64())
        .collect();

    if values.is_empty() {
        Ok(Value::Null)
    } else {
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        let rounded_avg = (avg * 10.0).round() / 10.0;
        Ok(Value::Number(
            serde_json::Number::from_f64(rounded_avg).unwrap(),
        ))
    }
}

fn calculate_sum(items: &[Value], field_name: &str) -> Result<Value, Error> {
    let sum: f64 = items
        .iter()
        .filter_map(|item| item.get(field_name))
        .filter_map(|val| val.as_f64())
        .sum();

    let rounded_sum = if sum.fract() == 0.0 {
        sum
    } else {
        (sum * 10.0).round() / 10.0
    };

    Ok(Value::Number(
        serde_json::Number::from_f64(rounded_sum).unwrap(),
    ))
}

fn calculate_min(items: &[Value], field_name: &str) -> Result<Value, Error> {
    let min_val = items
        .iter()
        .filter_map(|item| item.get(field_name))
        .filter_map(|val| val.as_f64())
        .fold(f64::INFINITY, f64::min);

    if min_val == f64::INFINITY {
        Ok(Value::Null)
    } else {
        Ok(Value::Number(
            serde_json::Number::from_f64(min_val).unwrap(),
        ))
    }
}

fn calculate_max(items: &[Value], field_name: &str) -> Result<Value, Error> {
    let max_val = items
        .iter()
        .filter_map(|item| item.get(field_name))
        .filter_map(|val| val.as_f64())
        .fold(f64::NEG_INFINITY, f64::max);

    if max_val == f64::NEG_INFINITY {
        Ok(Value::Null)
    } else {
        Ok(Value::Number(
            serde_json::Number::from_f64(max_val).unwrap(),
        ))
    }
}
