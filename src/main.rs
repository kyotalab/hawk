use anyhow::Result;
use hawk::{Error, execute_query, setup};

fn main() -> Result<(), Error> {
    // Load CLI arguments and file
    let (json, query, format) = setup()?;

    execute_query(&json, &query, format)?;

    Ok(())
}
