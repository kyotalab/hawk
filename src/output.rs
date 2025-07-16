use indexmap::IndexSet;
use serde_json::Value;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::{Error, OutputFormat, value_to_string};

#[derive(Debug)]
enum DataType {
    SimpleList,  // [string, number, bool]
    ObjectArray, // [{"name": "Alice"}, {"name": "Bob"}]
    NestedArray, // [[{"name": "Project1"}], [{"name": "Project2"}]]
    Mixed,       // Other complex structures
}

/// カラー出力設定
struct ColorScheme {
    header: ColorSpec,
    number: ColorSpec,
    string: ColorSpec,
    boolean: ColorSpec,
    null: ColorSpec,
    array_info: ColorSpec,
}

impl ColorScheme {
    fn new() -> Self {
        let mut header = ColorSpec::new();
        header.set_fg(Some(Color::Blue)).set_bold(true);
        
        let mut number = ColorSpec::new();
        number.set_fg(Some(Color::Green));
        
        let mut boolean = ColorSpec::new();
        boolean.set_fg(Some(Color::Yellow));
        
        let mut null = ColorSpec::new();
        null.set_fg(Some(Color::Black)).set_intense(true); // グレー
        
        let mut array_info = ColorSpec::new();
        array_info.set_fg(Some(Color::Cyan));
        
        Self {
            header,
            number,
            string: ColorSpec::new(), // デフォルト色
            boolean,
            null,
            array_info,
        }
    }
}

/// TTY判定とカラー出力の可否
fn should_use_colors() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stdout()) 
        && std::env::var("NO_COLOR").is_err()
}

/// 値の型に応じたカラースペックを取得
fn get_color_for_value<'a>(value: &Value, colors: &'a ColorScheme) -> &'a ColorSpec {
    match value {
        Value::Number(_) => &colors.number,
        Value::Bool(_) => &colors.boolean,
        Value::Null => &colors.null,
        _ => &colors.string,
    }
}

pub fn format_output(data: &[Value], format: OutputFormat) -> Result<(), Error> {
    if data.is_empty() {
        return Ok(());
    }

    let use_colors = should_use_colors();

    match format {
        OutputFormat::Json => {
            // 明示的にJSON出力
            print_as_json(data, use_colors)?;
        }
        OutputFormat::Table => {
            // 明示的にテーブル出力（可能な場合）
            if is_object_array(data) {
                print_as_table(data, use_colors)?;
            } else {
                let flattened = flatten_nested_arrays(data);
                if is_object_array(&flattened) {
                    print_as_table(&flattened, use_colors)?;
                } else {
                    return Err(Error::InvalidQuery(
                        "Cannot display as table: data is not object array".into(),
                    ));
                }
            }
        }
        OutputFormat::List => {
            // 明示的にリスト出力
            print_as_list(data, use_colors)?;
        }
        OutputFormat::Auto => {
            // 既存のスマート判定ロジック
            match analyze_data_structure(data) {
                DataType::SimpleList => print_as_list(data, use_colors)?,
                DataType::ObjectArray => print_as_table(data, use_colors)?,
                DataType::NestedArray => {
                    let flattened = flatten_nested_arrays(data);
                    if is_object_array(&flattened) {
                        print_as_table(&flattened, use_colors)?;
                    } else if is_simple_values(&flattened) {
                        print_as_list(&flattened, use_colors)?;
                    } else {
                        print_as_json(data, use_colors)?;
                    }
                }
                DataType::Mixed => print_as_json(data, use_colors)?,
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

fn collect_flattened_fields_ordered(value: &Value, prefix: &str, fields: &mut IndexSet<String>) {
    match value {
        Value::Object(obj) => {
            for (key, val) in obj {
                // serde_json::Mapは順序を保持
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
            format!("[{} items]", arr.len())
        }
        _ => value_to_string(current),
    }
}

fn get_field_value_for_coloring(item: &Value, field_path: &str) -> Value {
    let parts: Vec<&str> = field_path.split('.').collect();
    let mut current = item;

    for part in parts {
        match current.get(part) {
            Some(val) => current = val,
            None => return Value::Null,
        }
    }

    current.clone()
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

fn print_as_list(data: &[Value], use_colors: bool) -> Result<(), Error> {
    if use_colors {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        let colors = ColorScheme::new();
        
        for item in data {
            let color = get_color_for_value(item, &colors);
            stdout.set_color(color)?;
            print!("{}", value_to_string(item));
            stdout.reset()?;
            println!();
        }
    } else {
        data.iter().for_each(|item| {
            println!("{}", value_to_string(item));
        });
    }
    
    Ok(())
}

fn print_as_json(data: &[Value], use_colors: bool) -> Result<(), Error> {
    if use_colors {
        print_colored_json_simple(data)?;
    } else {
        let json = serde_json::to_string_pretty(data).map_err(Error::Json)?;
        println!("{}", json);
    }
    
    Ok(())
}

fn print_colored_json_simple(data: &[Value]) -> Result<(), Error> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let colors = ColorScheme::new();
    
    // シンプルなアプローチ：行ごとに処理
    let json = serde_json::to_string_pretty(data).map_err(Error::Json)?;
    
    for line in json.lines() {
        let trimmed = line.trim();
        
        // 行の内容に応じて色付け
        if trimmed.starts_with('"') && trimmed.contains(':') {
            // キー行: "key": value
            if let Some(colon_pos) = trimmed.find(':') {
                let key_part = &trimmed[..colon_pos + 1];
                let value_part = &trimmed[colon_pos + 1..].trim();
                
                // インデントを保持
                let indent = &line[..line.len() - line.trim_start().len()];
                print!("{}", indent);
                
                // キー部分（青色）
                stdout.set_color(&colors.header)?;
                print!("{}", key_part);
                stdout.reset()?;
                
                print!(" ");
                
                // 値部分（型に応じた色）
                print_colored_json_value(value_part, &colors, &mut stdout)?;
                println!();
            } else {
                println!("{}", line);
            }
        } else if trimmed.starts_with('{') || trimmed.starts_with('}') || 
                  trimmed.starts_with('[') || trimmed.starts_with(']') {
            // 構造文字（青色）
            let indent = &line[..line.len() - line.trim_start().len()];
            print!("{}", indent);
            stdout.set_color(&colors.header)?;
            print!("{}", trimmed);
            stdout.reset()?;
            println!();
        } else {
            // その他の行（配列要素など）
            let indent = &line[..line.len() - line.trim_start().len()];
            print!("{}", indent);
            print_colored_json_value(trimmed, &colors, &mut stdout)?;
            println!();
        }
    }
    
    Ok(())
}

fn print_colored_json_value(value_str: &str, colors: &ColorScheme, stdout: &mut StandardStream) -> Result<(), Error> {
    let clean_value = value_str.trim_end_matches(',');
    
    if clean_value == "null" {
        stdout.set_color(&colors.null)?;
        print!("{}", value_str);
        stdout.reset()?;
    } else if clean_value == "true" || clean_value == "false" {
        stdout.set_color(&colors.boolean)?;
        print!("{}", value_str);
        stdout.reset()?;
    } else if clean_value.starts_with('"') && clean_value.ends_with('"') {
        // 文字列値
        stdout.set_color(&colors.string)?;
        print!("{}", value_str);
        stdout.reset()?;
    } else if clean_value.parse::<f64>().is_ok() {
        // 数値
        stdout.set_color(&colors.number)?;
        print!("{}", value_str);
        stdout.reset()?;
    } else {
        // その他
        print!("{}", value_str);
    }
    
    Ok(())
}

fn print_as_table(data: &[Value], use_colors: bool) -> Result<(), Error> {
    if data.is_empty() {
        return Ok(());
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

    if use_colors {
        print_colored_table(data, &fields, &max_widths)?;
    } else {
        print_plain_table(data, &fields, &max_widths);
    }

    Ok(())
}

fn print_colored_table(data: &[Value], fields: &[String], max_widths: &[usize]) -> Result<(), Error> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let colors = ColorScheme::new();

    // 3. ヘッダー出力（色付き）
    stdout.set_color(&colors.header)?;
    for (i, field) in fields.iter().enumerate() {
        print!("{:<width$}", field, width = max_widths[i]);
        if i < fields.len() - 1 {
            print!("  ");
        }
    }
    stdout.reset()?;
    println!();

    // 4. データ行出力（色付き）
    for item in data {
        for (i, field) in fields.iter().enumerate() {
            let value_str = get_flattened_value(item, field);
            
            // 値の型に応じて色を設定
            let value = get_field_value_for_coloring(item, field);
            let color = get_color_for_value(&value, &colors);
            
            stdout.set_color(color)?;
            print!("{:<width$}", value_str, width = max_widths[i]);
            stdout.reset()?;
            
            if i < fields.len() - 1 {
                print!("  ");
            }
        }
        println!();
    }

    Ok(())
}

fn print_plain_table(data: &[Value], fields: &[String], max_widths: &[usize]) {
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

pub fn print_data_info(data: &[Value]) {
    let use_colors = should_use_colors();
    
    if use_colors {
        print_colored_data_info(data).unwrap_or_else(|_| print_plain_data_info(data));
    } else {
        print_plain_data_info(data);
    }
}

fn print_colored_data_info(data: &[Value]) -> Result<(), Error> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let colors = ColorScheme::new();

    // タイトル
    stdout.set_color(&colors.header)?;
    println!("=== Data Information ===");
    stdout.reset()?;
    
    println!("Total records: {}", data.len());

    if data.is_empty() {
        return Ok(());
    }

    // データ型の分析
    let first_item = &data[0];
    match first_item {
        Value::Object(obj) => {
            println!("Type: Object Array");
            println!("Fields: {}", obj.len());
            println!();

            // フィールド一覧と型情報
            stdout.set_color(&colors.header)?;
            println!("Field Details:");
            stdout.reset()?;
            
            for (key, value) in obj {
                let field_type = get_value_type_info(value);
                let sample_value = get_sample_value(value);
                
                // フィールド名を色付き
                stdout.set_color(&colors.string)?;
                print!("  {:<15}", key);
                stdout.reset()?;
                
                // 型情報を色付き
                let type_color = get_color_for_value(value, &colors);
                stdout.set_color(type_color)?;
                print!(" {:<10}", field_type);
                stdout.reset()?;
                
                println!(" (e.g., {})", sample_value);
            }

            // 配列フィールドの詳細
            println!();
            stdout.set_color(&colors.header)?;
            println!("Array Fields:");
            stdout.reset()?;
            
            for (key, value) in obj {
                if let Value::Array(arr) = value {
                    stdout.set_color(&colors.array_info)?;
                    print!("  {:<15}", key);
                    stdout.reset()?;
                    println!(" [{} items]", arr.len());
                    
                    if let Some(first_elem) = arr.first() {
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
        }
        _ => {
            println!("Type: Simple Values");
            // プリミティブ値の統計
        }
    }

    Ok(())
}

fn print_plain_data_info(data: &[Value]) {
    println!("=== Data Information ===");
    println!("Total records: {}", data.len());

    if data.is_empty() {
        return;
    }

    // データ型の分析
    let first_item = &data[0];
    match first_item {
        Value::Object(obj) => {
            println!("Type: Object Array");
            println!("Fields: {}", obj.len());
            println!();

            // フィールド一覧と型情報
            println!("Field Details:");
            for (key, value) in obj {
                let field_type = get_value_type_info(value);
                let sample_value = get_sample_value(value);
                println!("  {:<15} {:<10} (e.g., {})", key, field_type, sample_value);
            }

            // 配列フィールドの詳細
            println!();
            println!("Array Fields:");
            for (key, value) in obj {
                if let Value::Array(arr) = value {
                    println!("  {:<15} [{} items]", key, arr.len());
                    if let Some(first_elem) = arr.first() {
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
        }
        _ => {
            println!("Type: Simple Values");
            // プリミティブ値の統計
        }
    }
}
