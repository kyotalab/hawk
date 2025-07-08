use std::{fs::File, io::BufReader};

use anyhow::Result;
use hawk::Error;
use serde_json::Value;

fn main() -> Result<(), Error>{
    let file = File::open("users.json")?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;

    // `.users[0].name`
    let query = ".users.name";
    let mut segments = query.split('.');
    segments.next();
    let segment = segments.next().unwrap(); 
    let param = segments.next().unwrap();

    // debug
    // println!("{}", segment);
    // println!("{}", param);



    if segment.contains('[') && segment.contains(']') {
        let idx = segment.find('[').unwrap();
        let ridx = segment.find(']').unwrap();

        if idx >= ridx {
            return Err(Error::InvalidQuery("Invalid bracket order".into()));
        }
        // debug
        // println!("{}", idx);
        // println!("{}", ridx);

        let json_key = segment.get(..idx).unwrap();
        let index = segment.get(idx + 1..ridx).unwrap().parse::<usize>().map_err(|e| {
            Error::StrToInt(e)
        })?;


        // debug
        // println!("{:?}", json_key);
        // println!("{:?}", index);

        let values = json.get(json_key).ok_or(Error::InvalidQuery(format!("Key '{}' not found", json_key)))?;
        let res = &values[index].get(param);

        // output
        println!("{:?}", res);

    } else {
        let json_key = segment;

        let values = json.get(json_key).ok_or(Error::InvalidQuery(format!("Key '{}' not found", json_key)))?;
        let values_arr = values.as_array().unwrap();

        // debug
        // println!("{:?}", values_arr);

        let mut res = Vec::new();

        for value in values_arr.iter() {
            res.push(value.get(param).unwrap());
        }

        let res: Vec<_> = values_arr.iter()
            .map(|value| value.get(param).unwrap())
            .collect();
        
        let res: Vec<String> = res.into_iter()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                _ => v.to_string().trim_matches('"').to_string()
            })
            .collect();

        // output
        println!("{:?}", res);
    }

    Ok(())
}

