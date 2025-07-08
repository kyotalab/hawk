use std::{fs::File, io::BufReader};

use anyhow::Result;
use hawk::Error;
use serde_json::Value;

fn main() -> Result<()>{
    let file = File::open("users.json")?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;

    // `.users[0].name`
    let query = ".users[abc].name";
    let segments = query.split('.').collect::<Vec<&str>>();
    let segment = segments[1]; 
    let param = segments[2];

    // debug
    println!("{}", segment);
    println!("{}", param);

    let idx = segment.find('[').unwrap();
    let ridx = segment.find(']').unwrap();

    // debug
    println!("{}", idx);
    println!("{}", ridx);

    if segment.contains('[') && segment.contains(']') {
        let json_key = segment.get(..idx).unwrap();
        let index = segment.get(idx + 1..ridx).unwrap().parse::<usize>().map_err(|e| {
            Error::StrToInt(e)
        })?;


        // debug
        println!("{:?}", json_key);
        println!("{:?}", index);

        let values = json.get(json_key).unwrap();
        let res = &values[index].get(param);

        // output
        println!("{:?}", res);

    }

    Ok(())
}

