pub mod arg;
pub mod error;
pub mod executor;
pub mod filter;
pub mod output;
pub mod parser;
pub mod setup;
pub mod stats_opts;
pub mod string_ops;
pub mod utils;

pub use arg::*;
pub use error::*;
pub use executor::*;
pub use filter::*;
pub use output::*;
pub use parser::*;
use serde_json::Value;
pub use setup::*;
pub use stats_opts::*;
pub use string_ops::*;
pub use utils::*;

pub fn debug_json_order(json: &Value) {
    println!("=== Original JSON field order ===");

    // ルートレベル
    // Root level
    if let Value::Object(obj) = json {
        println!("Root fields:");
        for key in obj.keys() {
            println!("  {}", key);
        }

        // users配列の最初の要素のフィールド順序
        // Field order of the first element in the users array
        if let Some(Value::Array(users)) = obj.get("users") {
            if let Some(Value::Object(first_user)) = users.first() {
                println!("First user fields:");
                for key in first_user.keys() {
                    println!("  {}", key);
                }
            }
        }
    }
}
