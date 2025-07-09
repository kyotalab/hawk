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
