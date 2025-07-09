use serde_json::Value;

use crate::Error;


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
