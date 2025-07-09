use serde_json::Value;

pub fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => value.to_string().trim_matches('"').to_string()
    }
}
