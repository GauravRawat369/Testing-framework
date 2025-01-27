use anyhow::Result;
use testing_framework::types::config::{Evaluator, Status, load_config};

// Function that calls the evaluator
fn call_evaluator(config_path: &str, connector: &str) -> Result<Status> {
    // Load the configuration
    let config = load_config(config_path)?;
    println!("Loaded configuration: {:?}", config);
    // Call validate_parameters
    config.validate_parameters(connector)
}

// Example usage
fn main() -> Result<()> {
    let config_path = "input.json";

    // Call evaluator for the "stripe" connector
    match call_evaluator(config_path, "stripe")? {
        Status::Success => {
            println!("Transaction succeeded.");
            // Call recorder
            // record_transaction("stripe", Status::Success)?;
        },
        Status::Failure => {
            println!("Transaction failed.");
            // Call recorder
            // record_transaction("stripe", Status::Failure)?;
        },
    }

    Ok(())
}
