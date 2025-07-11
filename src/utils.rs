use serde_json::Value;

pub fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => value.to_string().trim_matches('"').to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_string_with_string() {
        let value = Value::String("Alice".to_string());
        assert_eq!(value_to_string(&value), "Alice");
    }

    #[test]
    fn test_value_to_string_with_number() {
        let value = Value::Number(serde_json::Number::from(42));
        assert_eq!(value_to_string(&value), "42");
    }

    #[test]
    fn test_value_to_string_with_boolean() {
        let value = Value::Bool(true);
        assert_eq!(value_to_string(&value), "true");

        let value = Value::Bool(false);
        assert_eq!(value_to_string(&value), "false");
    }

    #[test]
    fn test_value_to_string_with_null() {
        let value = Value::Null;
        assert_eq!(value_to_string(&value), "null");
    }
}
