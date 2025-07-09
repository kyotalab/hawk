use std::{fs::File, io::{self, BufRead, BufReader}};

use anyhow::Result;
use clap::Parser;
use hawk::{Args, Error};
use serde_json::Value;

fn main() -> Result<(), Error>{
    let args = Args::parse();
    let reader: Box<dyn BufRead> = if let Some(path) = args.path {
        Box::new(BufReader::new(File::open(path)?))
    } else {
        Box::new(BufReader::new(io::stdin()))
    };

    let json: Value = serde_json::from_reader(reader)?;

    // `.users[0].name`
    let query = args.query;
    let mut segments = query.split('.');
    segments.next();
    let segment = segments.next().ok_or(Error::InvalidQuery("Missing field segment in query".into()))?; 
    let param = segments.next().ok_or(Error::InvalidQuery("Missing parameter segment in query".into()))?;

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
        let res = values[index].get(param).ok_or(Error::InvalidQuery(format!("Field '{}' not found", param)))?;

        // output
        println!("{}", res);

    } else {
        let json_key = segment;

        let values = json.get(json_key).ok_or(Error::InvalidQuery(format!("Key '{}' not found", json_key)))?;
        let values_arr = values.as_array().ok_or(Error::InvalidQuery("Expected array".into()))?;

        // debug
        // println!("{:?}", values_arr);

        // TODO シャドーイング部分を一つのチェーンにしても良いかも
        let res: Vec<_> = values_arr.iter()
            .filter_map(|value| value.get(param))
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

