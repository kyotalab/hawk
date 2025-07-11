use indexmap::IndexSet;
use serde_json::Value;

use crate::{Error, OutputFormat, value_to_string};

#[derive(Debug)]
enum DataType {
    SimpleList,  // [string, number, bool]
    ObjectArray, // [{"name": "Alice"}, {"name": "Bob"}]
    NestedArray, // [[{"name": "Project1"}], [{"name": "Project2"}]]
    Mixed,       // Other complex structures
}

pub fn format_output(data: &[Value], format: OutputFormat) -> Result<(), Error> {
    if data.is_empty() {
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            // 明示的にJSON出力
            // Explicitly output JSON
            print_as_json(data)?;
        }
        OutputFormat::Table => {
            // 明示的にテーブル出力（可能な場合）
            // Explicitly output table (if possible)
            if is_object_array(data) {
                print_as_table(data);
            } else {
                let flattened = flatten_nested_arrays(data);
                if is_object_array(&flattened) {
                    print_as_table(&flattened);
                } else {
                    return Err(Error::InvalidQuery(
                        "Cannot display as table: data is not object array".into(),
                    ));
                }
            }
        }
        OutputFormat::List => {
            // 明示的にリスト出力
            // Explicitly output list
            print_as_list(data);
        }
        OutputFormat::Auto => {
            // 既存のスマート判定ロジック
            // Existing smart judgment logic
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

fn analyze_data_structure(data: &[Value]) -> DataType {
    if is_simple_values(data) {
        return DataType::SimpleList;
    }

    if is_object_array(data) {
        return DataType::ObjectArray;
    }

    // ネストした配列かチェック
    // Check for nested arrays
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
                // Expand the contents of the array
                flattened.extend(arr.iter().cloned());
            }
            _ => {
                flattened.push(item.clone());
            }
        }
    }

    flattened
}

fn collect_flattened_fields_ordered(value: &Value, prefix: &str, fields: &mut IndexSet<String>) {
    match value {
        Value::Object(obj) => {
            for (key, val) in obj {
                // serde_json::Mapは順序を保持
                // serde_json::Map preserves order
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
            // Arrays are displayed in simplified form.
            format!("[{} items]", arr.len())
        }
        _ => value_to_string(current),
    }
}

fn get_value_type_info(value: &Value) -> &'static str {
    match value {
        Value::String(_) => "String",
        Value::Number(_) => "Number",
        Value::Bool(_) => "Boolean",
        Value::Array(_) => "Array",
        Value::Object(_) => "Object",
        Value::Null => "Null",
    }
}

fn get_sample_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s.chars().take(20).collect::<String>()),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(arr) => format!("[{} items]", arr.len()),
        Value::Object(obj) => format!("{{{}...}}", obj.keys().next().unwrap_or(&"".to_string())),
        Value::Null => "null".to_string(),
    }
}

fn is_simple_values(data: &[Value]) -> bool {
    data.iter()
        .all(|v| matches!(v, Value::String(_) | Value::Number(_) | Value::Bool(_)))
}

fn is_object_array(data: &[Value]) -> bool {
    data.iter().all(|v| v.is_object())
}

fn print_as_list(data: &[Value]) {
    data.iter().for_each(|item| {
        println!("{}", value_to_string(item));
    });
}

fn print_as_json(data: &[Value]) -> Result<(), Error> {
    let json = serde_json::to_string_pretty(data).map_err(|e| Error::Json(e))?;

    println!("{}", json);
    Ok(())
}

fn print_as_table(data: &[Value]) {
    if data.is_empty() {
        return;
    }

    // 1. 全オブジェクトからフラット化されたフィールド名を収集
    // 1. Collect flattened field names from all objects
    let mut all_fields = IndexSet::new();
    for item in data {
        collect_flattened_fields_ordered(item, "", &mut all_fields);
    }

    let fields: Vec<String> = all_fields.into_iter().collect();

    // 2. 各列の最大幅を計算
    // 2. Calculate the maximum width of each column
    let mut max_widths = vec![0; fields.len()];

    // ヘッダーの幅
    // Header width
    for (i, field) in fields.iter().enumerate() {
        max_widths[i] = field.len();
    }

    // データの幅
    // Data width
    for item in data {
        for (i, field) in fields.iter().enumerate() {
            let value_str = get_flattened_value(item, field);
            max_widths[i] = max_widths[i].max(value_str.len());
        }
    }

    // 3. ヘッダー出力
    // 3. Header output
    for (i, field) in fields.iter().enumerate() {
        print!("{:<width$}", field, width = max_widths[i]);
        if i < fields.len() - 1 {
            print!("  ");
        }
    }
    println!();

    // 4. データ行出力
    // 4. Data row output
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

pub fn print_data_info(data: &[Value]) {
    println!("=== Data Information ===");
    println!("Total records: {}", data.len());

    if data.is_empty() {
        return;
    }

    // データ型の分析
    // Analysis of data types
    let first_item = &data[0];
    match first_item {
        Value::Object(obj) => {
            println!("Type: Object Array");
            println!("Fields: {}", obj.len());
            println!();

            // フィールド一覧と型情報
            // Field list and type information
            println!("Field Details:");
            for (key, value) in obj {
                let field_type = get_value_type_info(value);
                let sample_value = get_sample_value(value);
                println!("  {:<15} {:<10} (e.g., {})", key, field_type, sample_value);
            }

            // 配列フィールドの詳細
            // Details of array fields
            println!();
            println!("Array Fields:");
            for (key, value) in obj {
                if let Value::Array(arr) = value {
                    println!("  {:<15} [{} items]", key, arr.len());
                    if let Some(first_elem) = arr.get(0) {
                        if let Value::Object(elem_obj) = first_elem {
                            print!("    └─ ");
                            let sub_fields: Vec<&String> = elem_obj.keys().collect();
                            let sub_fields: Vec<&str> =
                                sub_fields.into_iter().map(|f| f.as_str()).collect();
                            println!("{}", sub_fields.join(", "));
                        }
                    }
                }
            }
        }
        Value::Array(_) => {
            println!("Type: Nested Array");
            // ネストした配列の詳細
            // Details of nested arrays
        }
        _ => {
            println!("Type: Simple Values");
            // プリミティブ値の統計
            // Statistics on primitive values
        }
    }
}
