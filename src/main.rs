use anyhow::Result;
use hawk::{execute_query, setup, Error};

fn main() -> Result<(), Error>{
    // Load CLI arguments and file
    let (json, query) = setup()?;

    let result = execute_query(&json, &query)?;

    println!("{:?}", result);

    Ok(())
}





