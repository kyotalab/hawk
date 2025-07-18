use serde_json::Value;

use crate::{Error, apply_stats_operation, print_data_info, string_ops, value_to_string};

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
fn apply_filter_with_string_operations(
    data: Vec<Value>,
    condition: &str,
) -> Result<Vec<Value>, Error> {
    // not演算子のチェック
    let (condition, is_negated) = parse_not_condition_with_parentheses(condition)?;

    let parts: Vec<&str> = condition.split(" | ").map(|s| s.trim()).collect();

    if parts.len() < 2 {
        return Err(Error::InvalidQuery("Invalid filter condition".to_string()));
    }

    let field_access = parts[0];
    let string_operations: Vec<&str> = parts[1..].to_vec();

    // 最後の操作は比較操作である必要がある
    let last_operation = string_operations
        .last()
        .ok_or_else(|| Error::InvalidQuery("Missing comparison operation".to_string()))?;

    if !is_comparison_operation(last_operation) {
        return Err(Error::InvalidQuery(
            "Last operation must be a comparison".to_string(),
        ));
    }

    let mut results = Vec::new();

    for item in data {
        // フィールド値を取得
        let field_value = extract_field_value(&item, field_access)?;

        // 文字列操作を適用（比較操作まで）
        let final_value = string_ops::apply_string_pipeline(&field_value, &string_operations)?;

        // 比較結果を評価し、not演算子を適用
        let condition_result = matches!(final_value, Value::Bool(true));
        let final_result = if is_negated {
            !condition_result
        } else {
            condition_result
        };

        if final_result {
            results.push(item);
        }
    }

    Ok(results)
}

/// 比較操作かどうかを判定
fn is_comparison_operation(operation: &str) -> bool {
    let trimmed = operation.trim();

    trimmed.starts_with("contains(")
        || trimmed.starts_with("starts_with(")
        || trimmed.starts_with("ends_with(")
        || trimmed == "=="
        || trimmed == "!="
        || trimmed.starts_with("== ")
        || trimmed.starts_with("!= ")
}

/// 既存のシンプルフィルタ処理
fn apply_existing_simple_filter(data: Vec<Value>, condition: &str) -> Result<Vec<Value>, Error> {
    // not演算子のチェック
    let (condition, is_negated) = parse_not_condition_with_parentheses(condition)?;

    // 条件をパース
    let (field_path, operator, value) = parse_condition(&condition)?;

    // フィルタリングを実行
    let filtered: Vec<Value> = data
        .into_iter()
        .filter(|item| {
            let result = evaluate_condition(item, &field_path, &operator, &value);
            if is_negated { !result } else { result }
        })
        .collect();

    Ok(filtered)
}

fn parse_not_condition_with_parentheses(condition: &str) -> Result<(String, bool), Error> {
    let trimmed = condition.trim();

    if trimmed.starts_with("not ") {
        let rest = trimmed[4..].trim();

        // 括弧で囲まれているかチェック
        if rest.starts_with('(') && rest.ends_with(')') {
            let inner_condition = rest[1..rest.len() - 1].trim().to_string();
            Ok((inner_condition, true))
        } else {
            Err(Error::InvalidQuery(
                "not operator requires parentheses around condition: not (.condition)".to_string(),
            ))
        }
    } else {
        Ok((trimmed.to_string(), false))
    }
}

pub fn apply_pipeline_operation(data: Vec<Value>, operation: &str) -> Result<Vec<Value>, Error> {
    let trimmed_op = operation.trim();

    if operation.starts_with(".[") && operation.ends_with("]") {
        return apply_universal_slice_operation(data, operation);
    }

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
        return Err(Error::InvalidQuery(
            "Multi-field map must have format: (.field1, .field2 | operation)".to_string(),
        ));
    }

    let fields_part = parts[0].trim();
    let operation = parts[1].trim();

    // フィールド部分をパース: ".skills, .projects"
    let field_paths: Vec<&str> = fields_part.split(',').map(|s| s.trim()).collect();

    // 各フィールドパスが "." で始まることを確認
    for field_path in &field_paths {
        if !field_path.starts_with('.') {
            return Err(Error::InvalidQuery(format!(
                "Field path must start with '.': {}",
                field_path
            )));
        }
    }

    let mut results = Vec::new();

    for item in data {
        // 各フィールドに同じ操作を適用（ケース1）
        let transformed_item =
            crate::string_ops::apply_operation_to_multiple_fields(&item, &field_paths, operation)?;
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
            Err(Error::InvalidQuery(format!(
                "Field '{}' not found",
                field_name
            )))
        }
    } else {
        Err(Error::InvalidQuery(format!(
            "Invalid field access: {}",
            field_access
        )))
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
fn update_or_create_value(
    original: &Value,
    field_access: &str,
    new_value: Value,
) -> Result<Value, Error> {
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
        Err(Error::InvalidQuery(format!(
            "Invalid field access: {}",
            field_access
        )))
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
    let condition = condition.trim();

    // 演算子を検出
    if let Some(pos) = condition.find(" >= ") {
        let field = condition[..pos].trim().to_string();
        let value = condition[pos + 4..].trim().to_string();
        return Ok((field, ">=".to_string(), value));
    }

    if let Some(pos) = condition.find(" <= ") {
        let field = condition[..pos].trim().to_string();
        let value = condition[pos + 4..].trim().to_string();
        return Ok((field, "<=".to_string(), value));
    }

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
        None => return false,
    };

    match operator {
        ">" => compare_greater(field_value, value),
        "<" => compare_less(field_value, value),
        ">=" => compare_greater_equal(field_value, value),
        "<=" => compare_less_equal(field_value, value),
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

fn compare_greater_equal(field_value: &Value, target: &str) -> bool {
    match field_value {
        Value::Number(n) => {
            if let Ok(target_num) = target.parse::<f64>() {
                n.as_f64().unwrap_or(0.0) >= target_num
            } else {
                false
            }
        }
        _ => false,
    }
}

fn compare_less_equal(field_value: &Value, target: &str) -> bool {
    match field_value {
        Value::Number(n) => {
            if let Ok(target_num) = target.parse::<f64>() {
                n.as_f64().unwrap_or(0.0) <= target_num
            } else {
                false
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

/// 配列に対してスライス操作を適用（汎用関数）
pub fn apply_array_slice(array: &[Value], start: Option<usize>, end: Option<usize>) -> Vec<Value> {
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

/// グループ化されたデータに対してスライスを適用
pub fn apply_slice_to_grouped_data(
    data: Vec<Value>,
    start: Option<usize>,
    end: Option<usize>,
) -> Result<Vec<Value>, Error> {
    let mut result = Vec::new();

    for group in data {
        if let Value::Array(group_items) = group {
            // 各グループに対してスライスを適用
            let sliced_group = apply_array_slice(&group_items, start, end);

            // スライス結果を展開して結果に追加
            result.extend(sliced_group);
        } else {
            // 配列でない場合はそのまま追加（エラー回避）
            result.push(group);
        }
    }

    Ok(result)
}

/// スライス記法をパース ([start:end] 形式)
pub fn parse_slice_notation(
    bracket_content: &str,
) -> Result<(Option<usize>, Option<usize>), Error> {
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

/// 負のインデックスに対応したスライス解析
pub fn parse_slice_notation_with_negative(
    bracket_content: &str,
    data_len: usize,
) -> Result<(Option<usize>, Option<usize>), Error> {
    let parts: Vec<&str> = bracket_content.split(':').collect();
    if parts.len() != 2 {
        return Err(Error::InvalidQuery(
            "Invalid slice format, expected start:end".to_string(),
        ));
    }

    let start = if parts[0].is_empty() {
        None
    } else {
        Some(parse_index_with_negative(parts[0], data_len)?)
    };

    let end = if parts[1].is_empty() {
        None
    } else {
        Some(parse_index_with_negative(parts[1], data_len)?)
    };

    Ok((start, end))
}

/// 負のインデックス対応のインデックス解析
pub fn parse_index_with_negative(index_str: &str, data_len: usize) -> Result<usize, Error> {
    if index_str.starts_with('-') {
        let negative_index = index_str[1..]
            .parse::<usize>()
            .map_err(|_| Error::InvalidQuery(format!("Invalid negative index: {}", index_str)))?;

        if negative_index > data_len {
            Ok(0) // 範囲外の場合は0に
        } else {
            Ok(data_len - negative_index)
        }
    } else {
        index_str
            .parse::<usize>()
            .map_err(|_| Error::InvalidQuery(format!("Invalid index: {}", index_str)))
    }
}

/// データ構造の種類を判定
#[derive(Debug, PartialEq)]
pub enum DataStructure {
    GroupedData,  // group_by後：全て配列
    RegularArray, // 通常の配列：オブジェクトや値の配列
    NestedArrays, // ネストした配列：配列の配列（group_byではない）
    Mixed,        // 混合
}

pub fn detect_data_structure(data: &[Value]) -> DataStructure {
    if data.is_empty() {
        return DataStructure::RegularArray;
    }

    let all_arrays = data.iter().all(|item| item.is_array());
    let all_objects = data.iter().all(|item| item.is_object());
    let all_primitives = data.iter().all(|item| {
        matches!(
            item,
            Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null
        )
    });

    if all_arrays {
        // 全て配列の場合、group_byの結果かネストした配列かを判定
        if is_likely_grouped_data(data) {
            DataStructure::GroupedData
        } else {
            DataStructure::NestedArrays
        }
    } else if all_objects || all_primitives {
        DataStructure::RegularArray
    } else {
        DataStructure::Mixed
    }
}

/// group_byの結果らしいデータかを判定
pub fn is_likely_grouped_data(data: &[Value]) -> bool {
    // 簡単なヒューリスティック：
    // 1. 全て配列
    // 2. 各配列が空でない
    // 3. 各配列の最初の要素が同じ構造（オブジェクト）

    if data.len() < 2 {
        return false; // グループが1個だけの場合は判定困難
    }

    for item in data {
        if let Value::Array(arr) = item {
            if arr.is_empty() {
                return false;
            }
            if !arr[0].is_object() {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

/// ユニバーサルスライス操作（あらゆるデータに対応）
pub fn apply_universal_slice_operation(
    data: Vec<Value>,
    operation: &str,
) -> Result<Vec<Value>, Error> {
    let bracket_content = &operation[2..operation.len() - 1]; // ".["と"]"を除去

    // 負のインデックス対応の確認
    if bracket_content.starts_with('-') && !bracket_content.contains(':') {
        return apply_negative_index_slice(data, bracket_content);
    }

    // 通常のスライス記法かどうかチェック
    if !bracket_content.contains(':') {
        // 単一インデックスの場合
        let index = bracket_content
            .parse::<usize>()
            .map_err(|_| Error::InvalidQuery(format!("Invalid index: {}", bracket_content)))?;

        if let Some(item) = data.get(index) {
            return Ok(vec![item.clone()]);
        } else {
            return Ok(vec![]); // インデックスが範囲外の場合は空を返す
        }
    }

    // スライス記法の解析
    let (start, end) = parse_slice_notation_with_negative(bracket_content, data.len())?;

    // データ構造に応じた適切な処理
    match detect_data_structure(&data) {
        DataStructure::GroupedData => apply_slice_to_grouped_data(data, start, end),
        DataStructure::RegularArray => apply_slice_to_regular_array(data, start, end),
        DataStructure::NestedArrays => apply_slice_to_nested_arrays(data, start, end),
        DataStructure::Mixed => apply_slice_to_regular_array(data, start, end), // デフォルト
    }
}

/// 負のインデックス単体の処理
pub fn apply_negative_index_slice(data: Vec<Value>, index_str: &str) -> Result<Vec<Value>, Error> {
    let data_len = data.len();
    let negative_index = index_str[1..]
        .parse::<usize>()
        .map_err(|_| Error::InvalidQuery(format!("Invalid negative index: {}", index_str)))?;

    if negative_index > data_len || negative_index == 0 {
        return Ok(vec![]); // 範囲外または-0の場合
    }

    let actual_index = data_len - negative_index;
    if let Some(item) = data.get(actual_index) {
        Ok(vec![item.clone()])
    } else {
        Ok(vec![])
    }
}

/// 通常の配列に対するスライス
pub fn apply_slice_to_regular_array(
    data: Vec<Value>,
    start: Option<usize>,
    end: Option<usize>,
) -> Result<Vec<Value>, Error> {
    let sliced = apply_array_slice(&data, start, end);
    Ok(sliced)
}

/// ネストした配列に対するスライス
pub fn apply_slice_to_nested_arrays(
    data: Vec<Value>,
    start: Option<usize>,
    end: Option<usize>,
) -> Result<Vec<Value>, Error> {
    // ネストした配列の場合、外側の配列をスライスする（より直感的）
    let sliced = apply_array_slice(&data, start, end);
    Ok(sliced)
}

/// フィールド指定ソート操作
pub fn apply_sort_with_field_operation(
    data: Vec<Value>,
    operation: &str,
) -> Result<Vec<Value>, Error> {
    let field_path = &operation[5..operation.len() - 1]; // "sort(" と ")" を除去

    let mut sorted_data = data;
    sorted_data.sort_by(|a, b| {
        let value_a = extract_sort_key(a, field_path);
        let value_b = extract_sort_key(b, field_path);

        compare_sort_values(&value_a, &value_b)
    });

    Ok(sorted_data)
}

/// ソート用のキー値を抽出
pub fn extract_sort_key(item: &Value, field_path: &str) -> Value {
    if field_path.starts_with('.') {
        let field_name = &field_path[1..];
        item.get(field_name).cloned().unwrap_or(Value::Null)
    } else {
        item.clone()
    }
}

/// ソート用の値比較
pub fn compare_sort_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match (a, b) {
        (Value::Number(n1), Value::Number(n2)) => {
            let f1 = n1.as_f64().unwrap_or(0.0);
            let f2 = n2.as_f64().unwrap_or(0.0);
            f1.partial_cmp(&f2).unwrap_or(Ordering::Equal)
        }
        (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
        (Value::Bool(b1), Value::Bool(b2)) => b1.cmp(b2),
        (Value::Null, Value::Null) => Ordering::Equal,
        (Value::Null, _) => Ordering::Less,
        (_, Value::Null) => Ordering::Greater,
        _ => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_apply_array_slice_basic() {
        let array = vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")];

        // [0:3] -> ["a", "b", "c"]
        let result = apply_array_slice(&array, Some(0), Some(3));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], json!("a"));
        assert_eq!(result[2], json!("c"));

        // [1:4] -> ["b", "c", "d"]
        let result = apply_array_slice(&array, Some(1), Some(4));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], json!("b"));

        // [:3] -> ["a", "b", "c"]
        let result = apply_array_slice(&array, None, Some(3));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], json!("a"));

        // [2:] -> ["c", "d", "e"]
        let result = apply_array_slice(&array, Some(2), None);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], json!("c"));

        // 範囲外の場合
        let result = apply_array_slice(&array, Some(10), Some(20));
        assert_eq!(result.len(), 0);

        // 無効な範囲
        let result = apply_array_slice(&array, Some(3), Some(1));
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_apply_slice_to_grouped_data() {
        // group_by後のデータ形式をシミュレート
        let grouped_data = vec![
            json!([
                {"category": "Electronics", "name": "Laptop", "price": 1200},
                {"category": "Electronics", "name": "Phone", "price": 800},
                {"category": "Electronics", "name": "Tablet", "price": 600},
                {"category": "Electronics", "name": "Mouse", "price": 25}
            ]),
            json!([
                {"category": "Books", "name": "Fiction", "price": 20},
                {"category": "Books", "name": "Science", "price": 30},
                {"category": "Books", "name": "History", "price": 25},
                {"category": "Books", "name": "Biography", "price": 35}
            ]),
            json!([
                {"category": "Clothing", "name": "Shirt", "price": 40},
                {"category": "Clothing", "name": "Pants", "price": 60},
                {"category": "Clothing", "name": "Shoes", "price": 80}
            ]),
        ];

        // 各グループから最初の2個を取得
        let result = apply_slice_to_grouped_data(grouped_data.clone(), Some(0), Some(2)).unwrap();

        // 結果の検証：3グループ × 2個 = 6個
        assert_eq!(result.len(), 6);

        // Electronics グループの最初の2個
        assert_eq!(result[0].get("name").unwrap(), &json!("Laptop"));
        assert_eq!(result[1].get("name").unwrap(), &json!("Phone"));

        // Books グループの最初の2個
        assert_eq!(result[2].get("name").unwrap(), &json!("Fiction"));
        assert_eq!(result[3].get("name").unwrap(), &json!("Science"));

        // Clothing グループの最初の2個
        assert_eq!(result[4].get("name").unwrap(), &json!("Shirt"));
        assert_eq!(result[5].get("name").unwrap(), &json!("Pants"));
    }

    #[test]
    fn test_apply_slice_to_grouped_data_different_ranges() {
        let grouped_data = vec![
            json!([
                {"id": 1, "group": "A"},
                {"id": 2, "group": "A"},
                {"id": 3, "group": "A"},
                {"id": 4, "group": "A"},
                {"id": 5, "group": "A"}
            ]),
            json!([
                {"id": 6, "group": "B"},
                {"id": 7, "group": "B"},
                {"id": 8, "group": "B"},
                {"id": 9, "group": "B"}
            ]),
        ];

        // 各グループから2番目から4番目まで（インデックス1-3）
        let result = apply_slice_to_grouped_data(grouped_data.clone(), Some(1), Some(4)).unwrap();

        // A群：3個（id: 2,3,4）、B群：3個（id: 7,8,9）= 合計6個
        assert_eq!(result.len(), 6);

        // A群の結果確認
        assert_eq!(result[0].get("id").unwrap(), &json!(2));
        assert_eq!(result[1].get("id").unwrap(), &json!(3));
        assert_eq!(result[2].get("id").unwrap(), &json!(4));

        // B群の結果確認
        assert_eq!(result[3].get("id").unwrap(), &json!(7));
        assert_eq!(result[4].get("id").unwrap(), &json!(8));
        assert_eq!(result[5].get("id").unwrap(), &json!(9));
    }

    #[test]
    fn test_parse_slice_notation() {
        // 通常のスライス
        let (start, end) = parse_slice_notation("0:5").unwrap();
        assert_eq!(start, Some(0));
        assert_eq!(end, Some(5));

        // 開始インデックスなし
        let (start, end) = parse_slice_notation(":5").unwrap();
        assert_eq!(start, None);
        assert_eq!(end, Some(5));

        // 終了インデックスなし
        let (start, end) = parse_slice_notation("2:").unwrap();
        assert_eq!(start, Some(2));
        assert_eq!(end, None);

        // 両方なし（全体）
        let (start, end) = parse_slice_notation(":").unwrap();
        assert_eq!(start, None);
        assert_eq!(end, None);

        // エラーケース
        assert!(parse_slice_notation("abc:def").is_err());
        assert!(parse_slice_notation("0:5:10").is_err());
    }

    #[test]
    fn test_parse_index_with_negative() {
        // 正のインデックス
        assert_eq!(parse_index_with_negative("5", 10).unwrap(), 5);

        // 負のインデックス
        assert_eq!(parse_index_with_negative("-1", 10).unwrap(), 9);
        assert_eq!(parse_index_with_negative("-3", 10).unwrap(), 7);

        // 範囲外の負のインデックス
        assert_eq!(parse_index_with_negative("-15", 10).unwrap(), 0);

        // エラーケース
        assert!(parse_index_with_negative("abc", 10).is_err());
        assert!(parse_index_with_negative("-abc", 10).is_err());
    }

    #[test]
    fn test_detect_data_structure() {
        // 通常の配列
        let regular = vec![json!({"id": 1}), json!({"id": 2})];
        assert_eq!(detect_data_structure(&regular), DataStructure::RegularArray);

        // グループ化されたデータ
        let grouped = vec![
            json!([{"cat": "A", "val": 1}, {"cat": "A", "val": 2}]),
            json!([{"cat": "B", "val": 3}, {"cat": "B", "val": 4}]),
        ];
        assert_eq!(detect_data_structure(&grouped), DataStructure::GroupedData);

        // プリミティブ値の配列
        let primitives = vec![json!(1), json!(2), json!(3)];
        assert_eq!(
            detect_data_structure(&primitives),
            DataStructure::RegularArray
        );

        // 空配列
        let empty: Vec<Value> = vec![];
        assert_eq!(detect_data_structure(&empty), DataStructure::RegularArray);
    }

    #[test]
    fn test_apply_sort_with_field_operation() {
        let data = vec![
            json!({"name": "Alice", "score": 85}),
            json!({"name": "Bob", "score": 92}),
            json!({"name": "Carol", "score": 78}),
        ];

        let result = apply_sort_with_field_operation(data, "sort(.score)").unwrap();

        // スコア順にソートされているか確認
        assert_eq!(result[0].get("score").unwrap(), &json!(78)); // Carol
        assert_eq!(result[1].get("score").unwrap(), &json!(85)); // Alice
        assert_eq!(result[2].get("score").unwrap(), &json!(92)); // Bob
    }

    #[test]
    fn test_apply_negative_index_slice() {
        let data = vec![json!("a"), json!("b"), json!("c"), json!("d"), json!("e")];

        // .[-1] - 最後の要素
        let result = apply_negative_index_slice(data.clone(), "-1").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], json!("e"));

        // .[-3] - 後ろから3番目
        let result = apply_negative_index_slice(data.clone(), "-3").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], json!("c"));

        // 範囲外
        let result = apply_negative_index_slice(data.clone(), "-10").unwrap();
        assert_eq!(result.len(), 0);
    }
}
