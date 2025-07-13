use anyhow::Result;
use hawk_data::{Error, execute_query, setup};

fn main() -> Result<(), Error> {
    let result = run();

    if let Err(ref e) = result {
        eprintln!("Error: {}", e);

        if let Error::InvalidQuery(_) = e {
            eprintln!("\nTry 'hawk --help' for usage examples.");
        }
        std::process::exit(1);
    }

    result
}

fn run() -> Result<(), Error> {
    let (json, query, format) = setup()?;
    execute_query(&json, &query, format)?;
    Ok(())
}
